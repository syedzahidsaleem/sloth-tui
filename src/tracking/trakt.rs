//! Trakt.tv OAuth device-code client, scrobbler, and bidirectional history synchronizer.

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use std::time::Duration;

use crate::providers::models::{Media, MediaType};
use crate::SlothError;

/// Trakt.tv API base URL.
pub const TRAKT_API_URL: &str = "https://api.trakt.tv";
/// Trakt.tv API protocol version header.
pub const TRAKT_API_VERSION: &str = "2";
/// Default Client ID for Sloth application.
pub const DEFAULT_TRAKT_CLIENT_ID: &str =
    "a849f1f0a4dd0f5d72ef7cb337e6b772c1c6fbba0de594f8e583c84ea383a1b0";

/// Response from `/oauth/device/code` initiating the OAuth device authorization flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    /// Internal device verification code sent during polling.
    pub device_code: String,
    /// User-facing alphanumeric code to enter on the activation webpage.
    pub user_code: String,
    /// Activation URL (typically `https://trakt.tv/activate`).
    pub verification_url: String,
    /// Lifetime of code in seconds.
    pub expires_in: u64,
    /// Polling interval in seconds.
    pub interval: u64,
}

/// Token payload returned by Trakt.tv upon successful device authorization or refresh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTokenResponse {
    /// Bearer access token for authenticated API calls.
    pub access_token: String,
    /// Token type (usually "bearer").
    pub token_type: String,
    /// Token lifetime in seconds from generation.
    pub expires_in: i64,
    /// Long-lived refresh token used to renew access.
    pub refresh_token: String,
    /// Token authorization scope.
    pub scope: String,
    /// Unix timestamp when token was issued.
    pub created_at: i64,
}

/// State of device token polling attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum DeviceTokenPollStatus {
    /// User has not yet entered code on the activation webpage.
    Pending,
    /// Rate limited; polling too quickly.
    SlowDown,
    /// The user code expired.
    Expired,
    /// The user explicitly denied authorization.
    Denied,
    /// Authorization succeeded with token.
    Success,
    /// Request or server error.
    Error(String),
}

/// Local SQLite mirror of a Trakt watched entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraktEntry {
    /// Unique Trakt identifier (e.g. "movie:123456" or "show:789:s1e2").
    pub trakt_id: String,
    /// Local media identifier if matched.
    pub media_id: Option<String>,
    /// Media kind ("movie" or "episode").
    pub kind: String,
    /// Unix timestamp when item was watched.
    pub watched_at: Option<i64>,
    /// Unix timestamp when entry was mirrored locally.
    pub synced_at: i64,
    /// Flag indicating whether local state needs pushing to Trakt.
    pub dirty: bool,
}

/// A movie record in Trakt API requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktMovie {
    /// Movie title.
    pub title: String,
    /// Release year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// External identifiers.
    pub ids: TraktIds,
}

/// A television series record in Trakt API requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktShow {
    /// Show title.
    pub title: String,
    /// Premiere year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// External identifiers.
    pub ids: TraktIds,
}

/// An episode record in Trakt API requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktEpisode {
    /// Season number (0 for specials).
    pub season: u32,
    /// Episode number within season.
    pub number: u32,
    /// Optional episode title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// External identifiers.
    #[serde(default)]
    pub ids: TraktIds,
}

/// Standardized Trakt external IDs object.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraktIds {
    /// Trakt numeric identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trakt: Option<u64>,
    /// URL slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// IMDb identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb: Option<String>,
    /// TMDb numeric identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdb: Option<u64>,
    /// TheTVDB numeric identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvdb: Option<u64>,
}

/// Historical watched entry returned from `/sync/history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktHistoryItem {
    /// Unique history entry identifier.
    pub id: u64,
    /// ISO-8601 watch timestamp.
    pub watched_at: String,
    /// Action type ("scrobble", "watch", "checkin").
    pub action: String,
    /// Item kind ("movie" or "episode").
    #[serde(rename = "type")]
    pub item_type: String,
    /// Movie metadata if item is a movie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movie: Option<TraktMovie>,
    /// Show metadata if item is an episode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show: Option<TraktShow>,
    /// Episode metadata if item is an episode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<TraktEpisode>,
}

/// Trakt.tv scrobble response returned by `/scrobble/*`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrobbleResponse {
    /// Scrobble activity ID.
    pub id: u64,
    /// Scrobble action state ("start", "pause", "scrobble").
    pub action: String,
    /// Playback completion percentage (0.0 - 100.0).
    pub progress: f64,
    /// Movie details if movie.
    pub movie: Option<TraktMovie>,
    /// Episode details if episode.
    pub episode: Option<TraktEpisode>,
    /// Show details if episode.
    pub show: Option<TraktShow>,
}

/// Trakt API client providing device authentication, scrobbling, and history synchronization.
#[derive(Clone)]
pub struct TraktClient {
    http: Arc<reqwest::Client>,
    client_id: String,
    client_secret: Option<String>,
    access_token: Option<String>,
}

impl TraktClient {
    /// Creates a new `TraktClient` instance.
    pub fn new(
        client_id: String,
        client_secret: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        Self {
            http: Arc::new(reqwest::Client::new()),
            client_id,
            client_secret,
            access_token,
        }
    }

    /// Returns the active access token if set.
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Authenticates against SQLite storage, refreshing expired tokens if a refresh token is present.
    pub async fn authenticate(
        pool: &SqlitePool,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<Option<Self>, SlothError> {
        let row = sqlx::query(
            "SELECT access_token, refresh_token, expires_at FROM auth_tokens WHERE provider = 'trakt'",
        )
        .fetch_optional(pool)
        .await?;

        let Some(r) = row else {
            return Ok(None);
        };

        let access_token: String = r.get("access_token");
        let refresh_token: Option<String> = r.get("refresh_token");
        let expires_at: Option<i64> = r.get("expires_at");
        let now = chrono::Utc::now().timestamp();

        // Check if token is still valid (with 60-second grace window)
        if expires_at.map(|exp| exp > (now + 60)).unwrap_or(true) {
            return Ok(Some(Self::new(
                client_id.to_string(),
                client_secret.map(|s| s.to_string()),
                Some(access_token),
            )));
        }

        // Token expired; try refreshing if refresh_token and secret are available
        if let (Some(refresh), Some(secret)) = (&refresh_token, client_secret) {
            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "refresh_token": refresh,
                "client_id": client_id,
                "client_secret": secret,
                "redirect_uri": "urn:ietf:wg:oauth:2.0:oob",
                "grant_type": "refresh_token"
            });

            let res = client
                .post(format!("{TRAKT_API_URL}/oauth/token"))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(token_resp) = response.json::<DeviceTokenResponse>().await {
                        let _ = Self::store_tokens(
                            pool,
                            &token_resp.access_token,
                            Some(&token_resp.refresh_token),
                            token_resp.expires_in,
                        )
                        .await;

                        return Ok(Some(Self::new(
                            client_id.to_string(),
                            client_secret.map(|s| s.to_string()),
                            Some(token_resp.access_token),
                        )));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Stores access and refresh tokens in SQLite `auth_tokens` table.
    pub async fn store_tokens(
        pool: &SqlitePool,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: i64,
    ) -> Result<(), SlothError> {
        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        sqlx::query(
            r#"
            INSERT INTO auth_tokens (provider, access_token, refresh_token, expires_at, created_at)
            VALUES ('trakt', ?1, ?2, ?3, unixepoch())
            ON CONFLICT(provider) DO UPDATE SET
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                expires_at = excluded.expires_at,
                created_at = unixepoch()
            "#,
        )
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Removes stored Trakt tokens from `auth_tokens`.
    pub async fn logout(pool: &SqlitePool) -> Result<(), SlothError> {
        sqlx::query("DELETE FROM auth_tokens WHERE provider = 'trakt'")
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Requests a new device code to begin user authorization on `https://trakt.tv/activate`.
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, SlothError> {
        let body = serde_json::json!({
            "client_id": self.client_id
        });

        let resp = self
            .http
            .post(format!("{TRAKT_API_URL}/oauth/device/code"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SlothError::Provider(crate::providers::models::ProviderError::Network(e.to_string())))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SlothError::Provider(
                crate::providers::models::ProviderError::Parsing(format!(
                    "Trakt device code request failed ({status}): {text}"
                )),
            ));
        }

        let device_code: DeviceCodeResponse = resp.json().await.map_err(|e| {
            SlothError::Provider(crate::providers::models::ProviderError::Parsing(e.to_string()))
        })?;

        Ok(device_code)
    }

    /// Polls `/oauth/device/token` once to check if the user has approved authorization.
    pub async fn poll_device_token(
        &self,
        device_code: &str,
    ) -> Result<(DeviceTokenPollStatus, Option<DeviceTokenResponse>), SlothError> {
        let mut body = serde_json::json!({
            "code": device_code,
            "client_id": self.client_id
        });

        if let Some(ref secret) = self.client_secret {
            body["client_secret"] = serde_json::Value::String(secret.clone());
        }

        let resp = self
            .http
            .post(format!("{TRAKT_API_URL}/oauth/device/token"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SlothError::Provider(crate::providers::models::ProviderError::Network(e.to_string())))?;

        match resp.status().as_u16() {
            200 => {
                let token: DeviceTokenResponse = resp.json().await.map_err(|e| {
                    SlothError::Provider(crate::providers::models::ProviderError::Parsing(
                        e.to_string(),
                    ))
                })?;
                Ok((DeviceTokenPollStatus::Success, Some(token)))
            }
            400 => Ok((DeviceTokenPollStatus::Pending, None)),
            404 => Ok((
                DeviceTokenPollStatus::Error("Device code not found".into()),
                None,
            )),
            409 => Ok((
                DeviceTokenPollStatus::Error("Device code already used".into()),
                None,
            )),
            410 => Ok((DeviceTokenPollStatus::Expired, None)),
            418 => Ok((DeviceTokenPollStatus::Denied, None)),
            429 => Ok((DeviceTokenPollStatus::SlowDown, None)),
            code => {
                let err_text = resp.text().await.unwrap_or_default();
                Ok((
                    DeviceTokenPollStatus::Error(format!("Trakt error ({code}): {err_text}")),
                    None,
                ))
            }
        }
    }

    /// Full interactive device authentication flow: generates code, opens browser, and polls for token.
    pub async fn start_device_auth_flow(
        &mut self,
        pool: &SqlitePool,
    ) -> Result<DeviceTokenResponse, SlothError> {
        let device_code = self.request_device_code().await?;

        println!("\n=======================================================");
        println!("Trakt.tv Authorization Required");
        println!("1. Open URL in your browser: {}", device_code.verification_url);
        println!("2. Enter verification code:  {}", device_code.user_code);
        println!("=======================================================\n");

        let _ = open::that(&device_code.verification_url);

        let poll_interval = Duration::from_secs(device_code.interval.max(5));
        let timeout_at =
            std::time::Instant::now() + Duration::from_secs(device_code.expires_in.max(300));

        while std::time::Instant::now() < timeout_at {
            tokio::time::sleep(poll_interval).await;

            let (status, token_opt) = self.poll_device_token(&device_code.device_code).await?;

            match status {
                DeviceTokenPollStatus::Success => {
                    if let Some(token) = token_opt {
                        Self::store_tokens(
                            pool,
                            &token.access_token,
                            Some(&token.refresh_token),
                            token.expires_in,
                        )
                        .await?;

                        self.access_token = Some(token.access_token.clone());
                        println!("Trakt.tv authorization successful!");
                        return Ok(token);
                    }
                }
                DeviceTokenPollStatus::Pending => {
                    // Continue polling
                }
                DeviceTokenPollStatus::SlowDown => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                DeviceTokenPollStatus::Expired => {
                    return Err(SlothError::Provider(
                        crate::providers::models::ProviderError::Parsing(
                            "Device code expired".into(),
                        ),
                    ));
                }
                DeviceTokenPollStatus::Denied => {
                    return Err(SlothError::Provider(
                        crate::providers::models::ProviderError::Parsing(
                            "User denied authorization".into(),
                        ),
                    ));
                }
                DeviceTokenPollStatus::Error(err) => {
                    return Err(SlothError::Provider(
                        crate::providers::models::ProviderError::Parsing(err),
                    ));
                }
            }
        }

        Err(SlothError::Provider(
            crate::providers::models::ProviderError::Parsing("Device code polling timed out".into()),
        ))
    }

    /// Internal helper to build an authenticated request with Trakt headers.
    fn authed_request(&self, method: reqwest::Method, url: &str) -> Result<reqwest::RequestBuilder, SlothError> {
        let Some(ref token) = self.access_token else {
            return Err(SlothError::Provider(
                crate::providers::models::ProviderError::AuthRequired,
            ));
        };

        Ok(self
            .http
            .request(method, url)
            .header("Content-Type", "application/json")
            .header("trakt-api-version", TRAKT_API_VERSION)
            .header("trakt-api-key", &self.client_id)
            .header("Authorization", format!("Bearer {token}")))
    }

    /// Sends a scrobble playback start notification to Trakt.
    pub async fn scrobble_start(
        &self,
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
        progress: f64,
    ) -> Result<ScrobbleResponse, SlothError> {
        self.send_scrobble("start", media, season, episode, progress)
            .await
    }

    /// Sends a scrobble playback pause notification to Trakt.
    pub async fn scrobble_pause(
        &self,
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
        progress: f64,
    ) -> Result<ScrobbleResponse, SlothError> {
        self.send_scrobble("pause", media, season, episode, progress)
            .await
    }

    /// Sends a scrobble playback stop notification to Trakt.
    pub async fn scrobble_stop(
        &self,
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
        progress: f64,
    ) -> Result<ScrobbleResponse, SlothError> {
        self.send_scrobble("stop", media, season, episode, progress)
            .await
    }

    async fn send_scrobble(
        &self,
        action: &str,
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
        progress: f64,
    ) -> Result<ScrobbleResponse, SlothError> {
        let url = format!("{TRAKT_API_URL}/scrobble/{action}");
        let payload = Self::build_media_payload(media, season, episode, progress);

        let req = self.authed_request(reqwest::Method::POST, &url)?;
        let resp = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| SlothError::Provider(crate::providers::models::ProviderError::Network(e.to_string())))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SlothError::Provider(
                crate::providers::models::ProviderError::Parsing(format!(
                    "Trakt scrobble {action} failed ({status}): {text}"
                )),
            ));
        }

        let scrobble: ScrobbleResponse = resp.json().await.map_err(|e| {
            SlothError::Provider(crate::providers::models::ProviderError::Parsing(e.to_string()))
        })?;

        Ok(scrobble)
    }

    /// Fetches user watched history from Trakt `/sync/history`.
    pub async fn fetch_history(
        &self,
        page: u32,
        limit: u32,
    ) -> Result<Vec<TraktHistoryItem>, SlothError> {
        let url = format!("{TRAKT_API_URL}/sync/history?page={page}&limit={limit}");
        let req = self.authed_request(reqwest::Method::GET, &url)?;

        let resp = req
            .send()
            .await
            .map_err(|e| SlothError::Provider(crate::providers::models::ProviderError::Network(e.to_string())))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SlothError::Provider(
                crate::providers::models::ProviderError::Parsing(format!(
                    "Trakt history fetch failed ({status}): {text}"
                )),
            ));
        }

        let items: Vec<TraktHistoryItem> = resp.json().await.map_err(|e| {
            SlothError::Provider(crate::providers::models::ProviderError::Parsing(e.to_string()))
        })?;

        Ok(items)
    }

    /// Fetches remote history and mirrors it into local SQLite `trakt_entries` table.
    pub async fn sync_history_to_db(
        &self,
        pool: &SqlitePool,
        page: u32,
        limit: u32,
    ) -> Result<usize, SlothError> {
        let items = self.fetch_history(page, limit).await?;
        let count = items.len();

        for item in items {
            let trakt_id = match item.item_type.as_str() {
                "movie" => {
                    let id = item.movie.as_ref().and_then(|m| m.ids.trakt);
                    match id {
                        Some(tid) => format!("movie:{tid}"),
                        None => format!("movie_hist_{}", item.id),
                    }
                }
                "episode" => {
                    let ep_id = item.episode.as_ref().and_then(|e| e.ids.trakt);
                    let season = item.episode.as_ref().map(|e| e.season).unwrap_or(1);
                    let number = item.episode.as_ref().map(|e| e.number).unwrap_or(1);
                    let show_id = item.show.as_ref().and_then(|s| s.ids.trakt).unwrap_or(0);
                    match ep_id {
                        Some(eid) => format!("show:{show_id}:s{season}e{number}:{eid}"),
                        None => format!("episode_hist_{}", item.id),
                    }
                }
                _ => format!("hist_{}", item.id),
            };

            let watched_ts = chrono::DateTime::parse_from_rfc3339(&item.watched_at)
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|_| chrono::Utc::now().timestamp());

            let _ = sqlx::query(
                r#"
                INSERT INTO trakt_entries (trakt_id, media_id, kind, watched_at, synced_at, dirty)
                VALUES (?1, NULL, ?2, ?3, unixepoch(), 0)
                ON CONFLICT(trakt_id) DO UPDATE SET
                    watched_at = excluded.watched_at,
                    synced_at = unixepoch(),
                    dirty = 0
                "#,
            )
            .bind(&trakt_id)
            .bind(&item.item_type)
            .bind(watched_ts)
            .execute(pool)
            .await;
        }

        Ok(count)
    }

    /// Marks media as watched on Trakt.tv `/sync/history` and local SQLite table.
    pub async fn mark_watched(
        &self,
        pool: Option<&SqlitePool>,
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> Result<(), SlothError> {
        let is_movie = media.media_type == MediaType::Movie;
        let kind_str = if is_movie { "movie" } else { "episode" };

        let trakt_id = if is_movie {
            format!("movie:{}", media.id)
        } else {
            let s = season.unwrap_or(1);
            let e = episode.unwrap_or(1);
            format!("show:{}:s{}e{}", media.id, s, e)
        };

        let payload = Self::build_sync_history_payload(media, season, episode);

        let res = self.authed_request(reqwest::Method::POST, &format!("{TRAKT_API_URL}/sync/history"));

        let mut success = false;
        if let Ok(req) = res {
            if let Ok(resp) = req.json(&payload).send().await {
                if resp.status().is_success() {
                    success = true;
                }
            }
        }

        if let Some(pool) = pool {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO media (id, provider_id, title, kind) VALUES (?1, 'unknown', ?2, ?3)",
            )
            .bind(&media.id)
            .bind(&media.title)
            .bind(kind_str)
            .execute(pool)
            .await;

            let dirty = if success { 0 } else { 1 };
            let _ = sqlx::query(
                r#"
                INSERT INTO trakt_entries (trakt_id, media_id, kind, watched_at, synced_at, dirty)
                VALUES (?1, ?2, ?3, unixepoch(), unixepoch(), ?4)
                ON CONFLICT(trakt_id) DO UPDATE SET
                    watched_at = excluded.watched_at,
                    synced_at = unixepoch(),
                    dirty = ?4
                "#,
            )
            .bind(&trakt_id)
            .bind(&media.id)
            .bind(kind_str)
            .bind(dirty)
            .execute(pool)
            .await;
        }

        if !success {
            return Err(SlothError::Provider(
                crate::providers::models::ProviderError::Unavailable(
                    "Trakt.tv API request failed".into(),
                ),
            ));
        }

        Ok(())
    }

    /// Synchronizes locally pending dirty watch records to Trakt `/sync/history`.
    pub async fn sync_dirty_entries(&self, pool: &SqlitePool) -> Result<usize, SlothError> {
        let rows = sqlx::query(
            "SELECT trakt_id, media_id, kind, watched_at FROM trakt_entries WHERE dirty = 1",
        )
        .fetch_all(pool)
        .await?;

        if rows.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for row in rows {
            let trakt_id: String = row.get("trakt_id");
            let media_id: Option<String> = row.get("media_id");
            let kind: Option<String> = row.get("kind");
            let watched_at: Option<i64> = row.get("watched_at");

            let payload = serde_json::json!({
                "movies": if kind.as_deref() == Some("movie") {
                    vec![serde_json::json!({
                        "ids": {
                            "tmdb": media_id.as_deref().and_then(|id| id.strip_prefix("tmdb:").or(Some(id))).and_then(|s| s.parse::<u64>().ok())
                        },
                        "watched_at": watched_at.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default().to_rfc3339())
                    })]
                } else {
                    vec![]
                }
            });

            if let Ok(req) = self.authed_request(reqwest::Method::POST, &format!("{TRAKT_API_URL}/sync/history")) {
                if let Ok(resp) = req.json(&payload).send().await {
                    if resp.status().is_success() {
                        let _ = sqlx::query("UPDATE trakt_entries SET dirty = 0 WHERE trakt_id = ?1")
                            .bind(&trakt_id)
                            .execute(pool)
                            .await;
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    fn build_media_payload(
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
        progress: f64,
    ) -> serde_json::Value {
        let progress_clamped = progress.clamp(0.0, 100.0);

        if media.media_type == MediaType::Movie {
            serde_json::json!({
                "movie": {
                    "title": media.title,
                    "year": media.year,
                    "ids": {
                        "tmdb": media.external_ids.tmdb,
                        "imdb": media.external_ids.imdb,
                        "trakt": media.external_ids.trakt.as_deref().and_then(|s| s.parse::<u64>().ok())
                    }
                },
                "progress": progress_clamped
            })
        } else {
            serde_json::json!({
                "show": {
                    "title": media.title,
                    "year": media.year,
                    "ids": {
                        "tmdb": media.external_ids.tmdb,
                        "imdb": media.external_ids.imdb,
                        "trakt": media.external_ids.trakt.as_deref().and_then(|s| s.parse::<u64>().ok())
                    }
                },
                "episode": {
                    "season": season.unwrap_or(1),
                    "number": episode.unwrap_or(1)
                },
                "progress": progress_clamped
            })
        }
    }

    fn build_sync_history_payload(
        media: &Media,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> serde_json::Value {
        let now_iso = chrono::Utc::now().to_rfc3339();

        if media.media_type == MediaType::Movie {
            serde_json::json!({
                "movies": [{
                    "title": media.title,
                    "year": media.year,
                    "ids": {
                        "tmdb": media.external_ids.tmdb,
                        "imdb": media.external_ids.imdb,
                        "trakt": media.external_ids.trakt.as_deref().and_then(|s| s.parse::<u64>().ok())
                    },
                    "watched_at": now_iso
                }]
            })
        } else {
            serde_json::json!({
                "shows": [{
                    "title": media.title,
                    "year": media.year,
                    "seasons": [{
                        "number": season.unwrap_or(1),
                        "episodes": [{
                            "number": episode.unwrap_or(1),
                            "watched_at": now_iso
                        }]
                    }]
                }]
            })
        }
    }
}
