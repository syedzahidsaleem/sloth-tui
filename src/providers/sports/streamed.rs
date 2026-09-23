//! Streamed provider implementation for live sports streams.
//!
//! Connects to the public streamed.su (formerly streamed.pk) JSON API
//! to retrieve live and upcoming sporting events and extract HLS/embed streams.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::providers::models::{EpisodeRef, Media, MediaType, ProviderError, Quality, StreamUrl};
use crate::providers::sports::models::{LiveMatch, MatchStream};
use crate::providers::{Provider, ProviderCapabilities};

const DEFAULT_BASE_URL: &str = "https://streamed.pk";
const USER_AGENT: &str = crate::net::DEFAULT_BROWSER_USER_AGENT;
const REFERER_HEADER: &str = "https://streamed.pk";

const MIRRORS: &[&str] = &[
    "https://streamed.pk",
    "https://streamed.top",
    "https://streamed.cc",
    "https://streamed.cx",
];

/// Live sports provider backed by the streamed.pk API.
pub struct StreamedProvider {
    client: Arc<reqwest::Client>,
    base_url: String,
}

impl StreamedProvider {
    /// Creates a new `StreamedProvider` using the default base URL and client settings.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .cookie_store(true)
            .build()
            .unwrap_or_default();
        Self {
            client: Arc::new(client),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Creates a new `StreamedProvider` with a shared HTTP client.
    pub fn with_client(client: Arc<reqwest::Client>) -> Self {
        Self {
            client,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Creates a new `StreamedProvider` with a custom HTTP client and base URL (useful for tests).
    pub fn with_base_url(client: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Internal helper to request endpoint across mirrors with failover.
    async fn fetch_endpoint(&self, path: &str) -> Result<String, ProviderError> {
        let is_local_test =
            self.base_url.contains("127.0.0.1") || self.base_url.contains("localhost");
        let mirrors: Vec<&str> = if is_local_test {
            vec![self.base_url.as_str()]
        } else {
            let mut m = vec![self.base_url.as_str()];
            for &mirror in MIRRORS {
                if !m.contains(&mirror) {
                    m.push(mirror);
                }
            }
            m
        };

        let mut last_err = None;
        for base in mirrors {
            let url = format!("{}{}", base.trim_end_matches('/'), path);
            match self
                .client
                .get(&url)
                .header("User-Agent", USER_AGENT)
                .header("Referer", base)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status() == reqwest::StatusCode::NOT_FOUND {
                        return Ok("[]".to_string());
                    }
                    if resp.status().is_success() {
                        if let Ok(body) = resp.text().await {
                            let trimmed = body.trim();
                            if trimmed.starts_with('[') || trimmed.starts_with('{') {
                                return Ok(body);
                            }
                        }
                    } else if is_local_test {
                        return Err(ProviderError::Unavailable(format!(
                            "Streamed API returned status: {}",
                            resp.status()
                        )));
                    }
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        if let Some(err) = last_err {
            Err(ProviderError::Network(err.to_string()))
        } else {
            Ok("[]".to_string())
        }
    }

    /// Fetches the currently live sporting events from `/api/matches/live`.
    pub async fn fetch_live_matches(&self) -> Result<Vec<LiveMatch>, ProviderError> {
        let body = self.fetch_endpoint("/api/matches/live").await?;
        let items: Vec<ApiMatchItem> = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse live matches JSON: {e}"))
        })?;

        let matches = items
            .into_iter()
            .map(|item| item.into_live_match(true))
            .collect();

        Ok(matches)
    }

    /// Internal helper to fetch all sporting events from `/api/matches/all`.
    async fn fetch_all_sports_matches(&self) -> Result<Vec<LiveMatch>, ProviderError> {
        let body = self.fetch_endpoint("/api/matches/all").await?;
        let items: Vec<ApiMatchItem> = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse all-sports matches JSON: {e}"))
        })?;

        let now = chrono::Utc::now();
        let matches = items
            .into_iter()
            .map(|item| {
                let starts_at = item.extract_starts_at();
                let is_live = starts_at.map_or(false, |dt| dt <= now);
                item.into_live_match(is_live)
            })
            .collect();

        Ok(matches)
    }

    /// Fetches sporting events filtered by sport category, or all events if sport is "all".
    pub async fn fetch_matches_by_sport(
        &self,
        sport: &str,
    ) -> Result<Vec<LiveMatch>, ProviderError> {
        let sport_normalized = sport.trim().to_lowercase();
        if sport_normalized == "live" {
            return self.fetch_live_matches().await;
        }

        if sport_normalized.is_empty()
            || sport_normalized == "all"
            || sport_normalized == "all-sports"
        {
            return self.fetch_all_sports_matches().await;
        }

        let cat_query = match sport_normalized.as_str() {
            "f1" | "formula 1" | "formula1" => "motor-sports",
            "boxing" | "mma" | "ufc" => "fight",
            other => other,
        };

        // Try direct sport endpoint first (e.g. /api/matches/{sport})
        if let Ok(body) = self
            .fetch_endpoint(&format!("/api/matches/{cat_query}"))
            .await
        {
            if let Ok(items) = serde_json::from_str::<Vec<ApiMatchItem>>(&body) {
                if !items.is_empty() {
                    let now = chrono::Utc::now();
                    let matches: Vec<LiveMatch> = items
                        .into_iter()
                        .map(|item| {
                            let starts_at = item.extract_starts_at();
                            let is_live = starts_at.map_or(false, |dt| dt <= now);
                            item.into_live_match(is_live)
                        })
                        .filter(|m| {
                            let title = m.title.to_lowercase();
                            if sport_normalized == "f1" {
                                title.contains("f1")
                                    || title.contains("formula")
                                    || title.contains("grand prix")
                                    || title.contains("nascar")
                                    || title.contains("rally")
                            } else if sport_normalized == "boxing" {
                                title.contains("box") || !title.contains("ufc")
                            } else if sport_normalized == "mma" {
                                title.contains("mma")
                                    || title.contains("ufc")
                                    || title.contains("pfl")
                                    || title.contains("contender")
                            } else {
                                true
                            }
                        })
                        .collect();
                    if !matches.is_empty() {
                        return Ok(matches);
                    }
                }
            }
        }

        // Fallback: Fetch all-sports and filter by category
        let all_matches = self.fetch_all_sports_matches().await?;
        let filtered = all_matches
            .into_iter()
            .filter(|m| {
                let cat = m.category.to_lowercase();
                let title = m.title.to_lowercase();
                if sport_normalized == "f1" {
                    cat == "motor-sports"
                        || title.contains("f1")
                        || title.contains("formula")
                        || title.contains("grand prix")
                        || title.contains("nascar")
                        || title.contains("rally")
                } else if sport_normalized == "boxing" {
                    cat == "fight" && (title.contains("box") || !title.contains("ufc"))
                } else if sport_normalized == "mma" {
                    cat == "fight"
                        && (title.contains("mma")
                            || title.contains("ufc")
                            || title.contains("pfl")
                            || title.contains("contender"))
                } else {
                    cat == sport_normalized || cat == cat_query
                }
            })
            .collect();

        Ok(filtered)
    }

    /// Fetches available stream sources for a specific match from `/api/stream/{source}/{match_id}`.
    pub async fn fetch_streams(
        &self,
        category: &str,
        match_id: &str,
    ) -> Result<Vec<MatchStream>, ProviderError> {
        let cat = if category.trim().is_empty() {
            "admin"
        } else {
            category.trim()
        };

        // Streamed.pk stream routes are /api/stream/{source}/{id}
        let endpoints = [
            format!("/api/stream/{cat}/{match_id}"),
            format!("/api/stream/admin/{match_id}"),
            format!("/api/stream/delta/{match_id}"),
            format!("/api/stream/golf/{match_id}"),
            format!("/api/stream/alpha/{match_id}"),
        ];

        for endpoint in &endpoints {
            if let Ok(body) = self.fetch_endpoint(endpoint).await {
                if let Ok(items) = serde_json::from_str::<Vec<ApiStreamItem>>(&body) {
                    if !items.is_empty() {
                        let streams = items
                            .into_iter()
                            .enumerate()
                            .map(|(idx, item)| {
                                let hd_url = item
                                    .hd_url
                                    .or_else(|| {
                                        item.stream_url
                                            .as_ref()
                                            .filter(|u| u.contains("hd"))
                                            .cloned()
                                    })
                                    .filter(|u| !u.trim().is_empty());
                                let sd_url = item
                                    .sd_url
                                    .or_else(|| {
                                        item.stream_url
                                            .as_ref()
                                            .filter(|u| u.contains("sd"))
                                            .cloned()
                                    })
                                    .filter(|u| !u.trim().is_empty());
                                let embed_url = item.embed_url.filter(|u| !u.trim().is_empty());

                                let quality = if item.hd == Some(true) || hd_url.is_some() {
                                    Quality::FHD1080
                                } else if sd_url.is_some() {
                                    Quality::SD480
                                } else {
                                    Quality::Auto
                                };

                                MatchStream {
                                    id: item.id.unwrap_or_else(|| {
                                        format!(
                                            "stream-{}",
                                            item.stream_no.unwrap_or((idx + 1) as u32)
                                        )
                                    }),
                                    hd_url,
                                    sd_url,
                                    embed_url,
                                    language: item.language.filter(|l| !l.trim().is_empty()),
                                    quality,
                                }
                            })
                            .collect();
                        return Ok(streams);
                    }
                }
            }
        }

        Err(ProviderError::NotFound)
    }
}

impl Default for StreamedProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for StreamedProvider {
    fn id(&self) -> &'static str {
        "streamed-pk"
    }

    fn name(&self) -> &'static str {
        "Streamed"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            movies: false,
            tv_series: false,
            anime: false,
            live_sports: true,
            iptv: false,
            f1: false,
            subtitles: false,
            download: false,
            quality_selection: true,
            dub_sub_toggle: false,
            supports_search: true,
            supports_pagination: false,
            supports_series: false,
            supports_subtitles: false,
            supports_homepage: false,
        }
    }

    async fn search(&self, query: &str, _kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        // Search live matches first
        let live_matches = self.fetch_live_matches().await.unwrap_or_default();
        let mut matches: Vec<LiveMatch> = live_matches
            .into_iter()
            .filter(|m| {
                m.title.to_lowercase().contains(&q)
                    || m.home_team
                        .as_ref()
                        .map_or(false, |t| t.to_lowercase().contains(&q))
                    || m.away_team
                        .as_ref()
                        .map_or(false, |t| t.to_lowercase().contains(&q))
                    || m.competition
                        .as_ref()
                        .map_or(false, |c| c.to_lowercase().contains(&q))
            })
            .collect();

        // If no matches found in live, check upcoming / all-sports
        if matches.is_empty() {
            if let Ok(all) = self.fetch_matches_by_sport("all-sports").await {
                matches = all
                    .into_iter()
                    .filter(|m| {
                        m.title.to_lowercase().contains(&q)
                            || m.home_team
                                .as_ref()
                                .map_or(false, |t| t.to_lowercase().contains(&q))
                            || m.away_team
                                .as_ref()
                                .map_or(false, |t| t.to_lowercase().contains(&q))
                            || m.competition
                                .as_ref()
                                .map_or(false, |c| c.to_lowercase().contains(&q))
                    })
                    .collect();
            }
        }

        let media_items = matches.into_iter().map(Media::from).collect();
        Ok(media_items)
    }

    async fn resolve(
        &self,
        media: &Media,
        _episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let category = media
            .genres
            .first()
            .filter(|g| !g.trim().is_empty())
            .map(|g| g.as_str())
            .unwrap_or("all");

        let mut streams = self
            .fetch_streams(category, &media.id)
            .await
            .unwrap_or_default();

        if streams.is_empty() {
            // Find match in live or all-sports to check its direct sources
            let matches = self.fetch_all_sports_matches().await.unwrap_or_default();
            if let Some(m) = matches.iter().find(|m| m.id == media.id) {
                for s in &m.streams {
                    if let Ok(strms) = self.fetch_streams(&m.category, &s.id).await {
                        if !strms.is_empty() {
                            streams = strms;
                            break;
                        }
                    }
                    if let Ok(strms) = self.fetch_streams("admin", &s.id).await {
                        if !strms.is_empty() {
                            streams = strms;
                            break;
                        }
                    }
                }
                if streams.is_empty() && !m.streams.is_empty() {
                    streams = m.streams.clone();
                }
            }
        }

        if streams.is_empty() {
            return Err(ProviderError::NotFound);
        }

        let mut results = Vec::new();
        for stream in streams {
            let mut added = false;
            if let Some(hd) = stream.hd_url {
                let is_hls = hd.contains(".m3u8");
                results.push(StreamUrl {
                    url: hd,
                    quality: Quality::FHD1080,
                    is_hls,
                    headers: vec![
                        ("Referer".into(), REFERER_HEADER.into()),
                        ("User-Agent".into(), USER_AGENT.into()),
                    ],
                    subtitle_url: None,
                    provider_id: "streamed-pk",
                });
                added = true;
            }

            if let Some(sd) = stream.sd_url {
                let is_hls = sd.contains(".m3u8");
                results.push(StreamUrl {
                    url: sd,
                    quality: Quality::SD480,
                    is_hls,
                    headers: vec![
                        ("Referer".into(), REFERER_HEADER.into()),
                        ("User-Agent".into(), USER_AGENT.into()),
                    ],
                    subtitle_url: None,
                    provider_id: "streamed-pk",
                });
                added = true;
            }

            if !added {
                if let Some(embed) = stream.embed_url {
                    let is_hls = embed.contains(".m3u8");
                    results.push(StreamUrl {
                        url: embed,
                        quality: Quality::Auto,
                        is_hls,
                        headers: vec![
                            ("Referer".into(), REFERER_HEADER.into()),
                            ("User-Agent".into(), USER_AGENT.into()),
                        ],
                        subtitle_url: None,
                        provider_id: "streamed-pk",
                    });
                }
            }
        }

        if results.is_empty() {
            return Err(ProviderError::NotFound);
        }

        Ok(results)
    }

    async fn health(&self) -> bool {
        if let Ok(body) = self.fetch_endpoint("/api/matches/live").await {
            serde_json::from_str::<serde_json::Value>(&body)
                .map(|v| v.is_array())
                .unwrap_or(false)
        } else {
            false
        }
    }
}

/// Internal representation of match items returned by the streamed.su API.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ApiMatchItem {
    id: String,
    title: String,
    #[serde(default)]
    category: String,
    teams: Option<ApiTeams>,
    competition: Option<String>,
    #[serde(default)]
    popular: Option<bool>,
    poster: Option<String>,
    #[serde(rename = "posterUrl", default)]
    poster_url: Option<String>,
    #[serde(default)]
    date: Option<ApiTimestamp>,
    #[serde(default, rename = "startsAt")]
    starts_at: Option<ApiTimestamp>,
    #[serde(default)]
    sources: Vec<ApiSourceItem>,
}

impl ApiMatchItem {
    fn extract_starts_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.starts_at
            .as_ref()
            .or(self.date.as_ref())
            .and_then(|ts| match ts {
                ApiTimestamp::Millis(ms) => chrono::DateTime::from_timestamp_millis(*ms),
                ApiTimestamp::Iso(s) => chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok(),
            })
    }

    fn into_live_match(self, is_live: bool) -> LiveMatch {
        let home_team = match &self.teams {
            Some(ApiTeams::Obj { home, .. }) => home.clone(),
            Some(ApiTeams::Names(h, _)) => Some(h.clone()),
            Some(ApiTeams::Single(t)) => Some(t.clone()),
            _ => None,
        };
        let away_team = match &self.teams {
            Some(ApiTeams::Obj { away, .. }) => away.clone(),
            Some(ApiTeams::Names(_, a)) => Some(a.clone()),
            _ => None,
        };
        let starts_at = self.extract_starts_at();
        let poster_url = self.poster_url.or(self.poster).map(|p| {
            if p.starts_with('/') {
                format!("https://streamed.pk{}", p)
            } else {
                p
            }
        });
        let is_popular = self.popular.unwrap_or(false);

        let streams = self
            .sources
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                let stream_id = s.id.clone().unwrap_or_else(|| {
                    format!("stream-{}", s.stream_no.unwrap_or((idx + 1) as u32))
                });
                let embed_url = match (&s.source, &s.id) {
                    (Some(src), Some(sid)) => Some(format!("https://embed.st/embed/{src}/{sid}/1")),
                    _ => None,
                };
                MatchStream {
                    id: stream_id,
                    hd_url: None,
                    sd_url: None,
                    embed_url,
                    language: None,
                    quality: Quality::Auto,
                }
            })
            .collect();

        LiveMatch {
            id: self.id,
            title: self.title,
            category: self.category,
            home_team,
            away_team,
            competition: self.competition,
            starts_at,
            is_live,
            is_popular,
            poster_url,
            streams,
        }
    }
}

/// Flexible deserialization for team representations.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ApiTeams {
    Obj {
        home: Option<String>,
        away: Option<String>,
    },
    Names(String, String),
    Single(String),
}

/// Flexible timestamp format handling epoch milliseconds or ISO strings.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ApiTimestamp {
    Millis(i64),
    Iso(String),
}

/// Brief stream source reference embedded in match listings.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ApiSourceItem {
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "streamNo", default)]
    stream_no: Option<u32>,
}

/// Direct stream metadata returned by `/api/stream/{category}/{match_id}`.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ApiStreamItem {
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "streamNo", default)]
    stream_no: Option<u32>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    hd: Option<bool>,
    #[serde(rename = "hdUrl", alias = "hd_url", default)]
    hd_url: Option<String>,
    #[serde(rename = "sdUrl", alias = "sd_url", default)]
    sd_url: Option<String>,
    #[serde(rename = "embedUrl", alias = "embed_url", default)]
    embed_url: Option<String>,
    #[serde(rename = "streamUrl", alias = "stream_url", alias = "url", default)]
    stream_url: Option<String>,
    #[serde(default)]
    source: Option<String>,
}
