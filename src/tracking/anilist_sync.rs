//! AniList OAuth implicit grant client and watch progress synchronizer.

use std::sync::Arc;
use serde::de::DeserializeOwned;
use sqlx::{Row, SqlitePool};

use crate::SlothError;

pub const ANILIST_API: &str = "https://graphql.anilist.co";
pub const ANILIST_AUTH_URL: &str = "https://anilist.co/api/v2/oauth/authorize";
pub const DEFAULT_ANILIST_CLIENT_ID: &str = "23450";
pub const PIN_REDIRECT_URI: &str = "https://anilist.co/api/v2/oauth/pin";

/// Cached or synchronized AniList list entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AniListEntry {
    pub anilist_id: u32,
    pub media_id: Option<String>,
    pub status: Option<String>,
    pub progress: u32,
    pub score: Option<f64>,
    pub notes: Option<String>,
    pub dirty: bool,
}

/// AniList API client supporting GraphQL queries and watch progress sync.
#[derive(Clone)]
pub struct AniListClient {
    http: Arc<reqwest::Client>,
    access_token: Option<String>,
}

impl AniListClient {
    /// Creates a new AniListClient with the given optional access token.
    pub fn new(access_token: Option<String>) -> Self {
        Self {
            http: Arc::new(reqwest::Client::new()),
            access_token,
        }
    }

    /// Generates the OAuth implicit grant authorization URL for user login.
    pub fn login_url(client_id: &str) -> String {
        format!("{ANILIST_AUTH_URL}?client_id={client_id}&response_type=token")
    }

    /// Checks the database `auth_tokens` table for a valid, non-expired AniList token.
    pub async fn authenticate(pool: &SqlitePool) -> Result<Option<Self>, SlothError> {
        let row = sqlx::query(
            "SELECT access_token, expires_at FROM auth_tokens WHERE provider = 'anilist'",
        )
        .fetch_optional(pool)
        .await?;

        if let Some(r) = row {
            let token: String = r.get("access_token");
            let expires_at: Option<i64> = r.get("expires_at");
            let now = chrono::Utc::now().timestamp();

            if expires_at.map(|exp| exp > now).unwrap_or(true) {
                return Ok(Some(Self {
                    http: Arc::new(reqwest::Client::new()),
                    access_token: Some(token),
                }));
            }
        }

        Ok(None)
    }

    /// Stores an access token in the `auth_tokens` table.
    pub async fn store_token(
        pool: &SqlitePool,
        token: &str,
        expires_at: Option<i64>,
    ) -> Result<(), SlothError> {
        sqlx::query(
            r#"
            INSERT INTO auth_tokens (provider, access_token, expires_at, created_at)
            VALUES ('anilist', ?1, ?2, unixepoch())
            ON CONFLICT(provider) DO UPDATE SET
                access_token = excluded.access_token,
                expires_at = excluded.expires_at,
                created_at = unixepoch()
            "#,
        )
        .bind(token)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Removes any stored AniList access token from `auth_tokens`.
    pub async fn logout(pool: &SqlitePool) -> Result<(), SlothError> {
        sqlx::query("DELETE FROM auth_tokens WHERE provider = 'anilist'")
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Initiates auth flow: generates URL, attempts to open the browser, and prints instructions.
    pub async fn start_auth_flow(pool: &SqlitePool, client_id: &str) -> Result<String, SlothError> {
        let url = Self::login_url(client_id);
        println!("Please open the following URL in your browser to authorize Sloth with AniList:");
        println!("{url}\n");
        let _ = open::that(&url);

        // If a token was already stored, return it
        if let Some(client) = Self::authenticate(pool).await? {
            if let Some(t) = client.access_token {
                return Ok(t);
            }
        }

        Ok(url)
    }

    /// Executes an authenticated GraphQL query or mutation against the AniList API.
    pub async fn gql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, SlothError> {
        let mut req = self.http.post(ANILIST_API).json(&serde_json::json!({
            "query": query,
            "variables": variables,
        }));

        if let Some(token) = &self.access_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if let Some(errors) = body.get("errors") {
            let msg = errors.to_string();
            return Err(SlothError::Io(std::io::Error::other(format!(
                "AniList GraphQL error: {msg}"
            ))));
        }

        let data = body
            .get("data")
            .ok_or_else(|| SlothError::Io(std::io::Error::other("Missing 'data' in GraphQL response")))?;

        serde_json::from_value(data.clone()).map_err(|e| SlothError::Io(std::io::Error::other(e)))
    }

    /// Fetches currently logged in user's username via `Viewer` query.
    pub async fn get_current_username(&self) -> Result<Option<String>, SlothError> {
        #[derive(serde::Deserialize)]
        struct ViewerData {
            #[serde(rename = "Viewer")]
            viewer: Option<ViewerInfo>,
        }
        #[derive(serde::Deserialize)]
        struct ViewerInfo {
            name: String,
        }

        let query = r#"
            query {
                Viewer {
                    name
                }
            }
        "#;

        let res: ViewerData = self.gql(query, serde_json::json!({})).await?;
        Ok(res.viewer.map(|v| v.name))
    }

    /// Alias for get_current_username.
    pub async fn get_authenticated_user(&self) -> Result<Option<String>, SlothError> {
        self.get_current_username().await
    }

    /// Marks an episode as watched on AniList and updates local `anilist_entries` cache.
    pub async fn mark_episode_watched(
        &self,
        anilist_id: u32,
        episode: u32,
    ) -> Result<(), SlothError> {
        let db_path = crate::config::db_path();
        let pool = crate::db::open(&db_path).await.ok();
        self.mark_episode_watched_with_pool(pool.as_ref(), anilist_id, episode)
            .await
    }

    /// Marks an episode as watched on AniList, updating local `anilist_entries` in the specified pool.
    pub async fn mark_episode_watched_with_pool(
        &self,
        pool: Option<&SqlitePool>,
        anilist_id: u32,
        episode: u32,
    ) -> Result<(), SlothError> {
        let query = r#"
            mutation ($mediaId: Int, $progress: Int) {
                SaveMediaListEntry (mediaId: $mediaId, progress: $progress) {
                    id
                    progress
                    status
                }
            }
        "#;

        let variables = serde_json::json!({
            "mediaId": anilist_id,
            "progress": episode,
        });

        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct SaveEntryData {
            #[serde(rename = "SaveMediaListEntry")]
            entry: Option<serde_json::Value>,
        }

        // Send GraphQL mutation to AniList
        match self.gql::<SaveEntryData>(query, variables).await {
            Ok(_) => {
                // Update local SQLite cache: progress=episode, dirty=0
                if let Some(p) = pool {
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO anilist_entries (anilist_id, progress, dirty, synced_at)
                        VALUES (?1, ?2, 0, unixepoch())
                        ON CONFLICT(anilist_id) DO UPDATE SET
                            progress = excluded.progress,
                            dirty = 0,
                            synced_at = unixepoch()
                        "#,
                    )
                    .bind(anilist_id as i64)
                    .bind(episode as i64)
                    .execute(p)
                    .await;
                }
                Ok(())
            }
            Err(err) => {
                // Queue as dirty if sync fails (retry later)
                if let Some(p) = pool {
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO anilist_entries (anilist_id, progress, dirty, synced_at)
                        VALUES (?1, ?2, 1, unixepoch())
                        ON CONFLICT(anilist_id) DO UPDATE SET
                            progress = excluded.progress,
                            dirty = 1,
                            synced_at = unixepoch()
                        "#,
                    )
                    .bind(anilist_id as i64)
                    .bind(episode as i64)
                    .execute(p)
                    .await;
                }
                Err(err)
            }
        }
    }

    /// Fetches the user's anime list from AniList and stores it in `anilist_entries`.
    pub async fn fetch_user_list(&self) -> Result<Vec<AniListEntry>, SlothError> {
        let db_path = crate::config::db_path();
        let pool = crate::db::open(&db_path).await.ok();
        self.fetch_user_list_with_pool(pool.as_ref()).await
    }

    /// Fetches user anime list from AniList and caches it in SQLite if pool is provided.
    pub async fn fetch_user_list_with_pool(
        &self,
        pool: Option<&SqlitePool>,
    ) -> Result<Vec<AniListEntry>, SlothError> {
        #[derive(serde::Deserialize)]
        struct CollectionData {
            #[serde(rename = "MediaListCollection")]
            collection: Option<MediaListCollection>,
        }
        #[derive(serde::Deserialize)]
        struct MediaListCollection {
            lists: Vec<MediaListGroup>,
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct MediaListGroup {
            name: Option<String>,
            entries: Vec<RawAniListEntry>,
        }
        #[derive(serde::Deserialize)]
        struct RawAniListEntry {
            #[serde(rename = "mediaId")]
            media_id: u32,
            status: Option<String>,
            progress: Option<u32>,
            score: Option<f64>,
            notes: Option<String>,
        }

        let query = r#"
            query {
                Viewer {
                    id
                }
            }
        "#;
        #[derive(serde::Deserialize)]
        struct ViewerIdData {
            #[serde(rename = "Viewer")]
            viewer: Option<ViewerId>,
        }
        #[derive(serde::Deserialize)]
        struct ViewerId {
            id: u32,
        }

        let viewer_data: ViewerIdData = self.gql(query, serde_json::json!({})).await?;
        let user_id = viewer_data
            .viewer
            .map(|v| v.id)
            .ok_or_else(|| SlothError::Io(std::io::Error::other("Failed to get Viewer ID")))?;

        let list_query = r#"
            query ($userId: Int) {
                MediaListCollection (userId: $userId, type: ANIME) {
                    lists {
                        name
                        entries {
                            mediaId
                            status
                            progress
                            score
                            notes
                        }
                    }
                }
            }
        "#;

        let collection_res: CollectionData = self
            .gql(list_query, serde_json::json!({ "userId": user_id }))
            .await?;

        let mut results = Vec::new();

        if let Some(col) = collection_res.collection {
            for group in col.lists {
                for raw in group.entries {
                    let entry = AniListEntry {
                        anilist_id: raw.media_id,
                        media_id: None,
                        status: raw.status,
                        progress: raw.progress.unwrap_or(0),
                        score: raw.score,
                        notes: raw.notes,
                        dirty: false,
                    };
                    results.push(entry);
                }
            }
        }

        if let Some(p) = pool {
            for entry in &results {
                let _ = sqlx::query(
                    r#"
                    INSERT INTO anilist_entries (anilist_id, status, progress, score, notes, dirty, synced_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, 0, unixepoch())
                    ON CONFLICT(anilist_id) DO UPDATE SET
                        status = excluded.status,
                        progress = excluded.progress,
                        score = excluded.score,
                        notes = excluded.notes,
                        dirty = 0,
                        synced_at = unixepoch()
                    "#,
                )
                .bind(entry.anilist_id as i64)
                .bind(entry.status.as_deref())
                .bind(entry.progress as i64)
                .bind(entry.score)
                .bind(entry.notes.as_deref())
                .execute(p)
                .await;
            }
        }

        Ok(results)
    }

    /// Synchronizes all locally queued entries marked as `dirty = 1` to AniList.
    pub async fn sync_dirty_entries(&self, pool: &SqlitePool) -> Result<(), SlothError> {
        let rows = sqlx::query(
            "SELECT anilist_id, progress FROM anilist_entries WHERE dirty = 1",
        )
        .fetch_all(pool)
        .await?;

        for row in rows {
            let anilist_id: i64 = row.get("anilist_id");
            let progress: i64 = row.get("progress");

            let mutation = r#"
                mutation ($mediaId: Int, $progress: Int) {
                    SaveMediaListEntry (mediaId: $mediaId, progress: $progress) {
                        id
                        progress
                    }
                }
            "#;

            let vars = serde_json::json!({
                "mediaId": anilist_id,
                "progress": progress,
            });

            #[derive(serde::Deserialize)]
            #[allow(dead_code)]
            struct MutationResp {
                #[serde(rename = "SaveMediaListEntry")]
                entry: Option<serde_json::Value>,
            }

            if self.gql::<MutationResp>(mutation, vars).await.is_ok() {
                let _ = sqlx::query(
                    "UPDATE anilist_entries SET dirty = 0, synced_at = unixepoch() WHERE anilist_id = ?1",
                )
                .bind(anilist_id)
                .execute(pool)
                .await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_url_format() {
        let url = AniListClient::login_url("12345");
        assert_eq!(
            url,
            "https://anilist.co/api/v2/oauth/authorize?client_id=12345&response_type=token"
        );
    }

    #[tokio::test]
    async fn test_authenticate_and_store_token() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory db");

        sqlx::query(
            r#"
            CREATE TABLE auth_tokens (
                provider TEXT PRIMARY KEY,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                expires_at INTEGER,
                scope TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );
            CREATE TABLE anilist_entries (
                anilist_id INTEGER PRIMARY KEY,
                media_id TEXT,
                status TEXT,
                progress INTEGER DEFAULT 0,
                score REAL,
                notes TEXT,
                synced_at INTEGER NOT NULL DEFAULT (unixepoch()),
                dirty INTEGER DEFAULT 0
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("create tables");

        // Initially no token
        let client = AniListClient::authenticate(&pool).await.expect("query");
        assert!(client.is_none());

        // Store token
        AniListClient::store_token(&pool, "test_token_xyz", None)
            .await
            .expect("store");

        // Authenticate succeeds
        let client = AniListClient::authenticate(&pool).await.expect("query");
        assert!(client.is_some());
        assert_eq!(client.unwrap().access_token.as_deref(), Some("test_token_xyz"));

        // Logout
        AniListClient::logout(&pool).await.expect("logout");
        let client = AniListClient::authenticate(&pool).await.expect("query");
        assert!(client.is_none());
    }
}
