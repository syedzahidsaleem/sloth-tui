use sloth_tui::tracking::anilist_sync::AniListClient;
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
fn test_login_url_format() {
    let url = AniListClient::login_url("12345");
    assert_eq!(
        url,
        "https://anilist.co/api/v2/oauth/authorize?client_id=12345&response_type=token"
    );
}

#[tokio::test]
async fn test_token_lifecycle() {
    let pool = setup_test_db().await;

    // 1. Initial state: unauthenticated
    let client = AniListClient::authenticate(&pool).await.unwrap();
    assert!(client.is_none());

    // 2. Store valid token
    AniListClient::store_token(&pool, "test_token_secret", None)
        .await
        .unwrap();

    let client = AniListClient::authenticate(&pool).await.unwrap();
    assert!(client.is_some());
    let client = client.unwrap();
    assert_eq!(client.token().unwrap(), "test_token_secret");

    // 3. Logout
    AniListClient::logout(&pool).await.unwrap();
    let client = AniListClient::authenticate(&pool).await.unwrap();
    assert!(client.is_none());
}

#[tokio::test]
async fn test_expired_token_ignored() {
    let pool = setup_test_db().await;

    // Store token that expired in the past (unix timestamp 1000)
    AniListClient::store_token(&pool, "old_expired_token", Some(1000))
        .await
        .unwrap();

    let client = AniListClient::authenticate(&pool).await.unwrap();
    assert!(client.is_none(), "Expired token must not be authenticated");
}

#[tokio::test]
async fn test_dirty_entry_on_sync_failure() {
    let pool = setup_test_db().await;

    AniListClient::store_token(&pool, "mock_token", None)
        .await
        .unwrap();
    let client = AniListClient::authenticate(&pool).await.unwrap().unwrap();

    // Call mark_episode_watched with in-memory pool
    // Since GraphQL server is unreachable or responds with error, it should record dirty=1 in DB
    let result = client
        .mark_episode_watched_with_pool(Some(&pool), 99999, 12)
        .await;
    assert!(result.is_err(), "Network call to fake endpoint should fail");

    // Verify entry is queued in local SQLite with dirty = 1
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT anilist_id, progress, dirty FROM anilist_entries WHERE anilist_id = 99999",
    )
    .fetch_one(&pool)
    .await
    .expect("Entry should be persisted in anilist_entries");

    assert_eq!(row.0, 99999);
    assert_eq!(row.1, 12);
    assert_eq!(row.2, 1, "Failed sync must be marked dirty=1");
}
