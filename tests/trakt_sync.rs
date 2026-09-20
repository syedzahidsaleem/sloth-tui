use sloth_tui::providers::models::{ExternalIds, Media, MediaType};
use sloth_tui::tracking::trakt::{DEFAULT_TRAKT_CLIENT_ID, TraktClient};
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

#[tokio::test]
async fn test_trakt_token_lifecycle() {
    let pool = setup_test_db().await;

    // 1. Initial state: unauthenticated
    let client = TraktClient::authenticate(&pool, DEFAULT_TRAKT_CLIENT_ID, None)
        .await
        .unwrap();
    assert!(client.is_none());

    // 2. Store valid tokens (expires in 2 hours)
    TraktClient::store_tokens(
        &pool,
        "mock_trakt_access_token",
        Some("mock_trakt_refresh_token"),
        7200,
    )
    .await
    .unwrap();

    let client = TraktClient::authenticate(&pool, DEFAULT_TRAKT_CLIENT_ID, None)
        .await
        .unwrap();
    assert!(client.is_some());
    let client = client.unwrap();
    assert_eq!(client.access_token(), Some("mock_trakt_access_token"));

    // 3. Logout
    TraktClient::logout(&pool).await.unwrap();
    let client = TraktClient::authenticate(&pool, DEFAULT_TRAKT_CLIENT_ID, None)
        .await
        .unwrap();
    assert!(client.is_none());
}

#[tokio::test]
async fn test_trakt_expired_token_without_secret() {
    let pool = setup_test_db().await;

    // Store token that expired in the past (-3600s)
    TraktClient::store_tokens(
        &pool,
        "old_expired_token",
        Some("refresh_token"),
        -3600,
    )
    .await
    .unwrap();

    // Without secret provided, cannot refresh -> should return None
    let client = TraktClient::authenticate(&pool, DEFAULT_TRAKT_CLIENT_ID, None)
        .await
        .unwrap();
    assert!(client.is_none(), "Expired token without secret must return None");
}

#[tokio::test]
async fn test_trakt_offline_movie_mark_watched_queued_as_dirty() {
    let pool = setup_test_db().await;

    let client = TraktClient::new(
        DEFAULT_TRAKT_CLIENT_ID.to_string(),
        None,
        Some("dummy_token".to_string()),
    );

    let media = Media {
        id: "mock_movie_42".to_string(),
        provider_id: "test",
        title: "Test Movie 42".to_string(),
        media_type: MediaType::Movie,
        year: Some(2024),
        poster_url: None,
        backdrop_url: None,
        rating: Some(8.5),
        duration_secs: Some(7200.0),
        overview: None,
        genres: vec!["Sci-Fi".to_string()],
        episodes_count: None,
        seasons_count: None,
        external_ids: ExternalIds {
            imdb: Some("tt9999999".to_string()),
            tmdb: Some(424242),
            ..Default::default()
        },
        cast: vec![],
    };

    // Call mark_watched with in-memory pool; network call will fail or be unreachable,
    // so it should fallback to recording a dirty=1 entry locally in SQLite trakt_entries.
    let res = client.mark_watched(Some(&pool), &media, None, None).await;
    assert!(res.is_err(), "Network call to dummy Trakt endpoint should fail");

    let row: (String, Option<String>, String, i64) = sqlx::query_as(
        "SELECT trakt_id, media_id, kind, dirty FROM trakt_entries WHERE trakt_id = 'movie:mock_movie_42'",
    )
    .fetch_one(&pool)
    .await
    .expect("Dirty record should exist in trakt_entries");

    assert_eq!(row.0, "movie:mock_movie_42");
    assert_eq!(row.1, Some("mock_movie_42".to_string()));
    assert_eq!(row.2, "movie");
    assert_eq!(row.3, 1, "Failed Trakt scrobble must be recorded as dirty = 1");
}

#[tokio::test]
async fn test_trakt_offline_episode_mark_watched_queued_as_dirty() {
    let pool = setup_test_db().await;

    let client = TraktClient::new(
        DEFAULT_TRAKT_CLIENT_ID.to_string(),
        None,
        Some("dummy_token".to_string()),
    );

    let media = Media {
        id: "mock_show_7".to_string(),
        provider_id: "test",
        title: "Test Show 7".to_string(),
        media_type: MediaType::Series,
        year: Some(2023),
        poster_url: None,
        backdrop_url: None,
        rating: Some(9.0),
        duration_secs: Some(3000.0),
        overview: None,
        genres: vec!["Drama".to_string()],
        episodes_count: Some(10),
        seasons_count: Some(2),
        external_ids: ExternalIds {
            imdb: Some("tt8888888".to_string()),
            tmdb: Some(77777),
            ..Default::default()
        },
        cast: vec![],
    };

    let res = client
        .mark_watched(Some(&pool), &media, Some(2), Some(4))
        .await;
    assert!(res.is_err(), "Network call should fail");

    let row: (String, Option<String>, String, i64) = sqlx::query_as(
        "SELECT trakt_id, media_id, kind, dirty FROM trakt_entries WHERE trakt_id = 'show:mock_show_7:s2e4'",
    )
    .fetch_one(&pool)
    .await
    .expect("Dirty episode record should exist in trakt_entries");

    assert_eq!(row.0, "show:mock_show_7:s2e4");
    assert_eq!(row.1, Some("mock_show_7".to_string()));
    assert_eq!(row.2, "episode");
    assert_eq!(row.3, 1, "Failed Trakt episode scrobble must be dirty = 1");
}

#[tokio::test]
async fn test_sync_trakt_function_safe_unauthenticated() {
    // Calling sync_trakt with no tokens stored should safely succeed without panic
    let result = sloth_tui::tracking::sync_trakt("test_unauth_item", 0, 0, 100.0).await;
    assert!(result.is_ok());
}
