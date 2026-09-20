use sloth_tui::metadata::tmdb::{TmdbClient, TmdbMovie, TmdbTv};
use sloth_tui::providers::models::ProviderKind;
use sloth_tui::providers::models::{
    CastMember, ExternalIds, Media, MediaDetails, MediaType, ProviderMediaId,
};
use sloth_tui::tui::screens::details;
use sloth_tui::tui::state::AppState;
use sloth_tui::tui::theme::Theme;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn setup_test_db() -> sqlx::SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to connect to in-memory sqlite db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[test]
fn test_poster_url() {
    let url1 = TmdbClient::poster_url("/abc1234.jpg");
    assert_eq!(url1, "https://image.tmdb.org/t/p/w500/abc1234.jpg");

    let url2 = TmdbClient::poster_url("def5678.jpg");
    assert_eq!(url2, "https://image.tmdb.org/t/p/w500/def5678.jpg");
}

#[tokio::test]
async fn test_graceful_skip_without_key() {
    let client = TmdbClient::new(None);
    assert_eq!(client.api_key(), None);

    assert!(
        client
            .search_movie("Inception", Some(2010))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        client
            .search_tv("Breaking Bad", Some(2008))
            .await
            .unwrap()
            .is_none()
    );
    assert!(client.movie_details(27205).await.unwrap().is_none());
    assert!(client.tv_details(1396).await.unwrap().is_none());
    assert!(client.tv_season_episodes(1396, 1).await.unwrap().is_empty());

    let mut media = Media {
        id: "moviebox:123".to_string(),
        title: "Dune: Part Two".to_string(),
        media_type: MediaType::Movie,
        year: Some(2024),
        overview: None,
        poster_url: None,
        backdrop_url: None,
        genres: vec![],
        rating: None,
        duration_secs: None,
        seasons_count: None,
        episodes_count: None,
        provider_id: "moviebox",
        external_ids: ExternalIds::default(),
        cast: vec![],
    };

    let result = client.enrich_media(&mut media).await;
    assert!(result.is_ok());
    assert_eq!(media.external_ids.tmdb, None);
}

#[test]
fn test_tmdb_movie_and_tv_deserialization() {
    let movie_json = r#"{
        "id": 27205,
        "title": "Inception",
        "overview": "Cobb steals information from targets...",
        "poster_path": "/edv5CZvWj09upOsy2Y6IwDhK8bt.jpg",
        "backdrop_path": "/s3TBrRGB1iav7gFOCNx3H31MoES.jpg",
        "release_date": "2010-07-15",
        "vote_average": 8.364,
        "runtime": 148,
        "genres": [{"id": 28, "name": "Action"}, {"id": 878, "name": "Science Fiction"}],
        "credits": {
            "cast": [
                {"id": 6193, "name": "Leonardo DiCaprio", "character": "Dom Cobb", "order": 0},
                {"id": 24045, "name": "Joseph Gordon-Levitt", "character": "Arthur", "order": 1}
            ]
        }
    }"#;

    let movie: TmdbMovie = serde_json::from_str(movie_json).expect("Failed to deserialize movie");
    assert_eq!(movie.id, 27205);
    assert_eq!(movie.title.as_deref(), Some("Inception"));
    assert_eq!(movie.genres.as_ref().unwrap().len(), 2);
    assert_eq!(
        movie.credits.as_ref().unwrap().cast.as_ref().unwrap().len(),
        2
    );

    let tv_json = r#"{
        "id": 1396,
        "name": "Breaking Bad",
        "overview": "Walter White, a New Mexico chemistry teacher...",
        "poster_path": "/ztkUQFLlC19CCMYHW9o1zWhJAGq.jpg",
        "backdrop_path": "/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg",
        "first_air_date": "2008-01-20",
        "vote_average": 8.9,
        "number_of_seasons": 5,
        "number_of_episodes": 62,
        "genres": [{"id": 18, "name": "Drama"}, {"id": 80, "name": "Crime"}],
        "credits": {
            "cast": [
                {"id": 17419, "name": "Bryan Cranston", "character": "Walter White", "order": 0}
            ]
        }
    }"#;

    let tv: TmdbTv = serde_json::from_str(tv_json).expect("Failed to deserialize tv show");
    assert_eq!(tv.id, 1396);
    assert_eq!(tv.name.as_deref(), Some("Breaking Bad"));
    assert_eq!(tv.number_of_seasons, Some(5));
}

#[tokio::test]
async fn test_sqlite_media_upsert() {
    let pool = setup_test_db().await;

    let media = Media {
        id: "moviebox:9999".to_string(),
        title: "Interstellar".to_string(),
        media_type: MediaType::Movie,
        year: Some(2014),
        overview: Some("A team of explorers travel through a wormhole in space...".to_string()),
        poster_url: Some(
            "https://image.tmdb.org/t/p/w500/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg".to_string(),
        ),
        backdrop_url: None,
        genres: vec![
            "Adventure".to_string(),
            "Drama".to_string(),
            "Science Fiction".to_string(),
        ],
        rating: Some(8.6),
        duration_secs: Some(169.0 * 60.0),
        seasons_count: None,
        episodes_count: None,
        provider_id: "moviebox",
        external_ids: ExternalIds {
            imdb: Some("tt0816692".to_string()),
            tmdb: Some(157336),
            mal: None,
            anilist: None,
            tvdb: None,
            trakt: None,
        },
        cast: vec![CastMember {
            name: "Matthew McConaughey".to_string(),
            character: Some("Joseph Cooper".to_string()),
        }],
    };

    let result = TmdbClient::upsert_media(&pool, &media).await;
    assert!(result.is_ok());

    let (title, tmdb_id, rating): (String, Option<i64>, Option<f64>) =
        sqlx::query_as("SELECT title, tmdb_id, rating FROM media WHERE id = 'moviebox:9999'")
            .fetch_one(&pool)
            .await
            .expect("Row should be present in media table");

    assert_eq!(title, "Interstellar");
    assert_eq!(tmdb_id, Some(157336));
    assert!((rating.unwrap() - 8.6).abs() < 0.01);
}

#[test]
fn test_details_screen_shows_tmdb_rating_and_attribution() {
    let backend = ratatui::backend::TestBackend::new(120, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let theme = Theme::mocha();

    let mut state = AppState {
        selected_details: Some(MediaDetails {
            id: ProviderMediaId {
                provider: ProviderKind::MovieBox,
                value: "interstellar".to_string(),
            },
            title: "Interstellar".to_string(),
            media_type: MediaType::Movie,
            year: Some("2014".to_string()),
            description: Some("Space exploration journey".to_string()),
            tagline: None,
            imdb_rating: Some("8.7".to_string()),
            director: Some("Christopher Nolan".to_string()),
            stars: None,
            prints: None,
            audios: None,
            poster_url: None,
            duration: Some("169m".to_string()),
            genres: vec!["Sci-Fi".to_string(), "Adventure".to_string()],
            seasons: vec![],
            dubs: vec![],
        }),
        selected_media: Some(Media {
            id: "interstellar".to_string(),
            title: "Interstellar".to_string(),
            media_type: MediaType::Movie,
            year: Some(2014),
            overview: Some("Space exploration journey".to_string()),
            poster_url: None,
            backdrop_url: None,
            genres: vec!["Sci-Fi".to_string(), "Adventure".to_string()],
            rating: Some(8.6),
            duration_secs: None,
            seasons_count: None,
            episodes_count: None,
            provider_id: "moviebox",
            external_ids: ExternalIds {
                imdb: Some("tt0816692".to_string()),
                tmdb: Some(157336),
                mal: None,
                anilist: None,
                tvdb: None,
                trakt: None,
            },
            cast: vec![
                CastMember {
                    name: "Matthew McConaughey".to_string(),
                    character: Some("Cooper".to_string()),
                },
                CastMember {
                    name: "Anne Hathaway".to_string(),
                    character: Some("Brand".to_string()),
                },
            ],
        }),
        ..Default::default()
    };

    terminal
        .draw(|frame| {
            let area = frame.area();
            details::draw(frame, area, &mut state, &theme);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let rendered = (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Check TMDB rating is displayed
    assert!(
        rendered.contains("TMDB"),
        "Details screen should display TMDB label"
    );
    assert!(
        rendered.contains("8.6"),
        "Details screen should display TMDB rating 8.6"
    );

    // Check genres are rendered as badges
    assert!(
        rendered.contains("[Sci-Fi]") || rendered.contains("Sci-Fi"),
        "Genres should be displayed"
    );

    // Check cast is rendered
    assert!(
        rendered.contains("Matthew McConaughey"),
        "Cast member should be displayed"
    );

    // Check TMDB attribution in bottom area
    assert!(
        rendered.contains("Powered by TMDB"),
        "TMDB attribution must be displayed"
    );
}
