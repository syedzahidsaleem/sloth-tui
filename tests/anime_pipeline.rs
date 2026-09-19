use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use sloth_tui::config::Config;
use sloth_tui::providers::anime::{AllAnimeProvider, HiAnimeProvider};
use sloth_tui::providers::models::{EpisodeRef, ExternalIds, Media, MediaType, Quality};
use sloth_tui::providers::{Provider, ProviderRegistry};

const HIANIME_SEARCH_JSON: &str = include_str!("fixtures/hianime/search_naruto.json");
const HIANIME_EPISODES_JSON: &str = include_str!("fixtures/hianime/episodes.json");
const HIANIME_SOURCES_JSON: &str = include_str!("fixtures/hianime/sources_sub.json");
const ALLANIME_SEARCH_JSON: &str = include_str!("fixtures/allanime/search_one_piece.json");
const ALLANIME_SOURCES_JSON: &str = include_str!("fixtures/allanime/episode_sources.json");

/// Spawns a lightweight local mock server returning hardcoded test fixture JSON.
async fn spawn_mock_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind mock server listener");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    let base_url = format!("http://127.0.0.1:{port}");

    let handle = tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };

            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");

                let (status, body) = if first_line.starts_with("HEAD /") || first_line.starts_with("GET / ") {
                    ("200 OK", "")
                } else if first_line.contains("/api/v2/hianime/search") {
                    if first_line.contains("q=Naruto") {
                        ("200 OK", HIANIME_SEARCH_JSON)
                    } else {
                        // Return empty array for queries other than Naruto
                        ("200 OK", r#"{"data":{"animes":[]}}"#)
                    }
                } else if first_line.contains("/api/v2/hianime/episodes/") {
                    ("200 OK", HIANIME_EPISODES_JSON)
                } else if first_line.contains("/api/v2/hianime/episode/sources") {
                    ("200 OK", HIANIME_SOURCES_JSON)
                } else if first_line.starts_with("POST /api") || first_line.starts_with("POST / ") {
                    if req.contains("sourceUrls") || req.contains("episodeString") {
                        ("200 OK", ALLANIME_SOURCES_JSON)
                    } else if req.contains("SearchInput") || req.contains("shows") {
                        ("200 OK", ALLANIME_SEARCH_JSON)
                    } else {
                        ("200 OK", "{}")
                    }
                } else if first_line.starts_with("GET /api") {
                    ("200 OK", "")
                } else {
                    ("404 Not Found", "{}")
                };

                let content_len = body.len();
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {content_len}\r\nConnection: close\r\n\r\n{body}"
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    (base_url, handle)
}

fn create_test_media(id: &str, title: &str, provider_id: &'static str) -> Media {
    Media {
        id: id.to_string(),
        title: title.to_string(),
        media_type: MediaType::Anime,
        year: Some(2002),
        overview: Some("Test anime description".to_string()),
        poster_url: None,
        backdrop_url: None,
        genres: vec!["Action".to_string(), "Adventure".to_string()],
        rating: Some(8.0),
        duration_secs: None,
        seasons_count: Some(1),
        episodes_count: Some(220),
        provider_id,
        external_ids: ExternalIds::default(),
        cast: Vec::new(),
    }
}

#[tokio::test]
async fn test_search_naruto_returns_results() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = HiAnimeProvider::with_base_url(client, base_url);

    let results = provider
        .search("Naruto", MediaType::Anime)
        .await
        .expect("Search should succeed");

    assert!(!results.is_empty(), "Search should return results");
    assert_eq!(results[0].id, "naruto-677");
    assert_eq!(results[0].title, "Naruto");
    assert_eq!(results[0].media_type, MediaType::Anime);
    assert_eq!(results[0].provider_id, "hianime");
    assert_eq!(results[0].rating, Some(8.0));
    assert_eq!(results[0].episodes_count, Some(220));

    assert_eq!(results[1].id, "naruto-shippuden-355");
    assert_eq!(results[1].title, "Naruto: Shippuden");
}

#[tokio::test]
async fn test_resolve_episode_1_returns_stream_url() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = HiAnimeProvider::with_base_url(client, base_url);

    let media = create_test_media("naruto-677", "Naruto", "hianime");

    // Case a: episode is None, should resolve episode 1
    let streams = provider
        .resolve(&media, None)
        .await
        .expect("Resolve should succeed");

    assert!(!streams.is_empty(), "Should return playable stream URLs");
    let stream = &streams[0];
    assert!(
        stream.url.contains("master.m3u8"),
        "URL should contain master.m3u8"
    );
    assert!(stream.is_hls, "Stream should be flagged as HLS");
    assert_eq!(stream.provider_id, "hianime");
    assert_eq!(stream.quality, Quality::Auto);

    // Verify Referer header
    let has_referer = stream
        .headers
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("Referer") && v == "https://hianime.to");
    assert!(has_referer, "Headers must include Referer: https://hianime.to");

    // Verify English subtitle URL
    assert!(stream.subtitle_url.is_some(), "Subtitle URL should be present");
    assert!(
        stream
            .subtitle_url
            .as_ref()
            .unwrap()
            .contains("english.vtt"),
        "Subtitle should point to english.vtt"
    );

    // Case b: explicit EpisodeRef for episode 1
    let ep1 = EpisodeRef {
        season: 1,
        episode: 1,
        title: Some("Enter: Naruto Uzumaki!".to_string()),
        duration_secs: None,
    };
    let streams_ep1 = provider
        .resolve(&media, Some(&ep1))
        .await
        .expect("Resolve with explicit episode 1 should succeed");
    assert!(!streams_ep1.is_empty());
}

#[tokio::test]
async fn test_health_returns_true_when_server_responds_200() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = HiAnimeProvider::with_base_url(client, base_url);

    let healthy = provider.health().await;
    assert!(healthy, "health() should return true when server responds 200");
}

#[tokio::test]
async fn test_episodes_listing() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = HiAnimeProvider::with_base_url(client, base_url);

    let media = create_test_media("naruto-677", "Naruto", "hianime");
    let episodes = provider
        .episodes(&media, 1)
        .await
        .expect("Episodes fetch should succeed");

    assert_eq!(episodes.len(), 3);
    assert_eq!(episodes[0].episode, 1);
    assert_eq!(
        episodes[0].title,
        Some("Enter: Naruto Uzumaki!".to_string())
    );
    assert_eq!(episodes[1].episode, 2);
    assert_eq!(episodes[2].episode, 3);
}

#[tokio::test]
async fn test_allanime_search_returns_results_for_one_piece() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = AllAnimeProvider::with_base_url(client, base_url);

    let results = provider
        .search("One Piece", MediaType::Anime)
        .await
        .expect("Search should succeed");

    assert!(!results.is_empty(), "AllAnime search should return results");
    assert_eq!(results[0].id, "ReiveDemon64");
    assert_eq!(results[0].title, "One Piece");
    assert_eq!(results[0].media_type, MediaType::Anime);
    assert_eq!(results[0].provider_id, "allanime");
    assert_eq!(results[0].rating, Some(8.8));
    assert_eq!(results[0].episodes_count, Some(1120));

    assert_eq!(results[1].id, "FilmRed123");
    assert_eq!(results[1].title, "One Piece Film: Red");
}

#[tokio::test]
async fn test_allanime_resolve_episode_stream() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = AllAnimeProvider::with_base_url(client, base_url);

    let media = create_test_media("ReiveDemon64", "One Piece", "allanime");
    let streams = provider
        .resolve(&media, None)
        .await
        .expect("AllAnime resolve should succeed");

    assert!(!streams.is_empty(), "Streams should not be empty");
    let stream = &streams[0];
    assert!(stream.url.contains("onepiece-ep1.m3u8"));
    assert!(stream.is_hls);
    assert_eq!(stream.provider_id, "allanime");
}

#[tokio::test]
async fn test_fallback_to_allanime_when_hianime_returns_empty_results() {
    let (base_url, _server) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());

    let hianime = Arc::new(HiAnimeProvider::with_base_url(
        Arc::clone(&client),
        base_url.clone(),
    ));
    let allanime = Arc::new(AllAnimeProvider::with_base_url(client, base_url));

    let mut registry = ProviderRegistry::new(&Config::default());
    registry.anime_chain = vec![hianime, allanime];

    // Query "One Piece" — HiAnime returns empty result, AllAnime returns One Piece
    let search_results = registry.search("One Piece", MediaType::Anime).await;

    assert!(
        !search_results.is_empty(),
        "Fallback should return AllAnime results"
    );
    assert_eq!(search_results[0].title, "One Piece");
    assert_eq!(search_results[0].provider_id, "allanime");
}
