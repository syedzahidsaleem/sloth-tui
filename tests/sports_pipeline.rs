use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use sloth_tui::config::Config;
use sloth_tui::providers::models::{Media, MediaType, Quality};
use sloth_tui::providers::sports::StreamedProvider;
use sloth_tui::providers::{Provider, ProviderRegistry};

const LIVE_MATCHES_JSON: &str = include_str!("fixtures/sports/live_matches.json");
const STREAM_SOURCES_JSON: &str = include_str!("fixtures/sports/stream_sources.json");

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

                let (status, body) =
                    if first_line.starts_with("HEAD /") || first_line.starts_with("GET / ") {
                        ("200 OK", "")
                    } else if first_line.contains("/api/matches/live") {
                        ("200 OK", LIVE_MATCHES_JSON)
                    } else if first_line.contains("/api/matches/all-sports") {
                        ("200 OK", LIVE_MATCHES_JSON)
                    } else if first_line.contains("/api/matches/football") {
                        ("200 OK", LIVE_MATCHES_JSON)
                    } else if first_line.contains("/api/stream/") {
                        ("200 OK", STREAM_SOURCES_JSON)
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

#[tokio::test]
async fn test_fetch_live_matches_returns_populated_list() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let matches = provider
        .fetch_live_matches()
        .await
        .expect("fetch_live_matches should succeed");

    assert_eq!(matches.len(), 2);

    let m1 = &matches[0];
    assert_eq!(m1.id, "arsenal-vs-chelsea-2026-09-19");
    assert_eq!(m1.title, "Arsenal vs Chelsea");
    assert_eq!(m1.category, "football");
    assert_eq!(m1.home_team.as_deref(), Some("Arsenal"));
    assert_eq!(m1.away_team.as_deref(), Some("Chelsea"));
    assert_eq!(m1.competition.as_deref(), Some("Premier League"));
    assert!(m1.is_live);
    assert!(m1.is_popular);

    let m2 = &matches[1];
    assert_eq!(m2.id, "lakers-vs-warriors-2026-09-19");
    assert_eq!(m2.title, "LA Lakers vs Golden State Warriors");
    assert_eq!(m2.category, "basketball");
    assert_eq!(m2.home_team.as_deref(), Some("LA Lakers"));
    assert_eq!(m2.away_team.as_deref(), Some("Golden State Warriors"));
    assert!(m2.is_live);
}

#[tokio::test]
async fn test_resolve_match_returns_stream_url() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let matches = provider
        .fetch_live_matches()
        .await
        .expect("fetch_live_matches should succeed");
    let first_match = &matches[0];
    let media = Media::from(first_match.clone());

    let streams = provider
        .resolve(&media, None)
        .await
        .expect("resolve should succeed");

    assert!(!streams.is_empty(), "Stream URLs should not be empty");

    let hd_stream = streams
        .iter()
        .find(|s| s.quality == Quality::FHD1080)
        .expect("Should have at least one FHD stream");

    assert!(
        hd_stream.url.contains(".m3u8"),
        "URL should be an HLS stream"
    );
    assert!(hd_stream.is_hls, "is_hls should be true for m3u8 stream");
    assert_eq!(hd_stream.provider_id, "streamed-pk");

    // Verify required Referer header
    let referer = hd_stream
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Referer"))
        .map(|(_, v)| v.as_str());
    assert_eq!(
        referer,
        Some("https://streamed.su"),
        "Referer header must be https://streamed.su"
    );

    // Verify User-Agent header
    let user_agent = hd_stream
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("User-Agent"))
        .map(|(_, v)| v.as_str());
    assert_eq!(
        user_agent,
        Some("Sloth-TUI/0.1.0"),
        "User-Agent header must match Sloth-TUI"
    );
}

#[tokio::test]
async fn test_fetch_streams_direct() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let streams = provider
        .fetch_streams("football", "arsenal-vs-chelsea-2026-09-19")
        .await
        .expect("fetch_streams should succeed");

    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "stream-1");
    assert_eq!(streams[0].language.as_deref(), Some("English"));
    assert!(streams[0].hd_url.is_some());

    assert_eq!(streams[1].id, "stream-2");
    assert_eq!(streams[1].language.as_deref(), Some("Spanish"));
    assert!(streams[1].hd_url.is_some());
}

#[tokio::test]
async fn test_fetch_matches_by_sport() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let matches = provider
        .fetch_matches_by_sport("football")
        .await
        .expect("fetch_matches_by_sport should succeed");

    assert!(!matches.is_empty());
    assert!(matches.iter().any(|m| m.category == "football"));

    let live_matches = provider
        .fetch_matches_by_sport("live")
        .await
        .expect("fetch_matches_by_sport('live') should succeed");

    assert_eq!(live_matches.len(), 2);
}

#[tokio::test]
async fn test_search_live_sports_filtering() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let results = provider
        .search("Arsenal", MediaType::LiveSport)
        .await
        .expect("search should succeed");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Arsenal vs Chelsea");

    let results_lower = provider
        .search("chelsea", MediaType::LiveSport)
        .await
        .expect("case-insensitive search should succeed");

    assert_eq!(results_lower.len(), 1);
    assert_eq!(results_lower[0].title, "Arsenal vs Chelsea");

    let empty = provider
        .search("NonExistentSportEvent123", MediaType::LiveSport)
        .await
        .expect("search should succeed");

    assert!(empty.is_empty());
}

#[tokio::test]
async fn test_sports_health_check() {
    let (base_url, _handle) = spawn_mock_server().await;
    let client = Arc::new(reqwest::Client::new());
    let provider = StreamedProvider::with_base_url(client, base_url);

    let is_healthy = provider.health().await;
    assert!(is_healthy, "StreamedProvider should be healthy");
}

#[tokio::test]
async fn test_provider_registry_sports_chain() {
    let registry = ProviderRegistry::new(&Config::default());
    let chain = registry.chain_for(&MediaType::LiveSport);

    assert_eq!(
        chain.len(),
        1,
        "sports_chain must have 1 provider registered"
    );
    let provider = &chain[0];
    assert_eq!(provider.id(), "streamed-pk");
    assert_eq!(provider.name(), "Streamed");
    assert!(
        provider.capabilities().live_sports,
        "capabilities.live_sports must be true"
    );
}
