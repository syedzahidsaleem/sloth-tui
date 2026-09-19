//! Streamed provider implementation for live sports streams.
//!
//! Connects to the public streamed.su (formerly streamed.pk) JSON API
//! to retrieve live and upcoming sporting events and extract HLS/embed streams.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::providers::models::{
    EpisodeRef, Media, MediaType, ProviderError, Quality, StreamUrl,
};
use crate::providers::sports::models::{LiveMatch, MatchStream};
use crate::providers::{Provider, ProviderCapabilities};

const DEFAULT_BASE_URL: &str = "https://streamed.su";
const USER_AGENT: &str = "Sloth-TUI/0.1.0";
const REFERER_HEADER: &str = "https://streamed.su";

/// Live sports provider backed by the streamed.su API.
pub struct StreamedProvider {
    client: Arc<reqwest::Client>,
    base_url: String,
}

impl StreamedProvider {
    /// Creates a new `StreamedProvider` using the default base URL and client settings.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
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

    /// Fetches the currently live sporting events from `/api/matches/live`.
    pub async fn fetch_live_matches(&self) -> Result<Vec<LiveMatch>, ProviderError> {
        let url = format!("{}/api/matches/live", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(Vec::new());
            }
            return Err(ProviderError::Unavailable(format!(
                "Streamed API matches/live returned status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let items: Vec<ApiMatchItem> = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse live matches JSON: {e}"))
        })?;

        let matches = items
            .into_iter()
            .map(|item| item.into_live_match(true))
            .collect();

        Ok(matches)
    }

    /// Internal helper to fetch all sporting events from `/api/matches/all-sports`.
    async fn fetch_all_sports_matches(&self) -> Result<Vec<LiveMatch>, ProviderError> {
        let url = format!(
            "{}/api/matches/all-sports",
            self.base_url.trim_end_matches('/')
        );
        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(Vec::new());
            }
            return Err(ProviderError::Unavailable(format!(
                "Streamed API matches/all-sports returned status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

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
    pub async fn fetch_matches_by_sport(&self, sport: &str) -> Result<Vec<LiveMatch>, ProviderError> {
        let sport_normalized = sport.trim().to_lowercase();
        if sport_normalized == "live" {
            return self.fetch_live_matches().await;
        }

        if sport_normalized.is_empty() || sport_normalized == "all" || sport_normalized == "all-sports" {
            return self.fetch_all_sports_matches().await;
        }

        // Try direct sport endpoint first (e.g. /api/matches/{sport})
        let url = format!(
            "{}/api/matches/{}",
            self.base_url.trim_end_matches('/'),
            sport_normalized
        );
        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| ProviderError::Network(e.to_string()))?;

            if let Ok(items) = serde_json::from_str::<Vec<ApiMatchItem>>(&body) {
                let now = chrono::Utc::now();
                let matches = items
                    .into_iter()
                    .map(|item| {
                        let starts_at = item.extract_starts_at();
                        let is_live = starts_at.map_or(false, |dt| dt <= now);
                        item.into_live_match(is_live)
                    })
                    .collect();
                return Ok(matches);
            }
        }

        // Fallback: Fetch all-sports and filter by category
        let all_matches = self.fetch_all_sports_matches().await?;
        let filtered = all_matches
            .into_iter()
            .filter(|m| m.category.eq_ignore_ascii_case(&sport_normalized))
            .collect();

        Ok(filtered)
    }

    /// Fetches available stream sources for a specific match from `/api/stream/{category}/{match_id}`.
    pub async fn fetch_streams(
        &self,
        category: &str,
        match_id: &str,
    ) -> Result<Vec<MatchStream>, ProviderError> {
        let cat = if category.trim().is_empty() {
            "all"
        } else {
            category.trim()
        };
        let url = format!(
            "{}/api/stream/{}/{}",
            self.base_url.trim_end_matches('/'),
            cat,
            match_id
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(Vec::new());
            }
            return Err(ProviderError::Unavailable(format!(
                "Streamed API stream request returned status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let items: Vec<ApiStreamItem> = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse match streams JSON: {e}"))
        })?;

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

                let quality = if hd_url.is_some() {
                    Quality::FHD1080
                } else if sd_url.is_some() {
                    Quality::SD480
                } else {
                    Quality::Auto
                };

                MatchStream {
                    id: item
                        .id
                        .unwrap_or_else(|| format!("stream-{}", item.stream_no.unwrap_or((idx + 1) as u32))),
                    hd_url,
                    sd_url,
                    embed_url,
                    language: item.language.filter(|l| !l.trim().is_empty()),
                    quality,
                }
            })
            .collect();

        Ok(streams)
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
            .cloned();

        let streams = if let Some(cat) = category {
            self.fetch_streams(&cat, &media.id).await?
        } else {
            let live = self.fetch_live_matches().await.unwrap_or_default();
            if let Some(m) = live.iter().find(|m| m.id == media.id) {
                self.fetch_streams(&m.category, &media.id).await?
            } else {
                self.fetch_streams("all", &media.id).await?
            }
        };

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
        let url = format!("{}/api/matches/live", self.base_url.trim_end_matches('/'));
        let Ok(response) = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        else {
            return false;
        };

        if !response.status().is_success() {
            return false;
        }

        let Ok(body) = response.text().await else {
            return false;
        };

        serde_json::from_str::<serde_json::Value>(&body)
            .map(|v| v.is_array())
            .unwrap_or(false)
    }
}

/// Internal representation of match items returned by the streamed.su API.
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
            _ => None,
        };
        let away_team = match &self.teams {
            Some(ApiTeams::Obj { away, .. }) => away.clone(),
            Some(ApiTeams::Names(_, a)) => Some(a.clone()),
            _ => None,
        };
        let starts_at = self.extract_starts_at();
        let poster_url = self.poster_url.or(self.poster);
        let is_popular = self.popular.unwrap_or(false);

        let streams = self
            .sources
            .into_iter()
            .enumerate()
            .map(|(idx, s)| MatchStream {
                id: s
                    .id
                    .unwrap_or_else(|| format!("stream-{}", s.stream_no.unwrap_or((idx + 1) as u32))),
                hd_url: None,
                sd_url: None,
                embed_url: None,
                language: None,
                quality: Quality::Auto,
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
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ApiTimestamp {
    Millis(i64),
    Iso(String),
}

/// Brief stream source reference embedded in match listings.
#[derive(Debug, Clone, Deserialize)]
struct ApiSourceItem {
    id: Option<String>,
    #[serde(rename = "streamNo")]
    stream_no: Option<u32>,
}

/// Direct stream metadata returned by `/api/stream/{category}/{match_id}`.
#[derive(Debug, Clone, Deserialize)]
struct ApiStreamItem {
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "streamNo", default)]
    stream_no: Option<u32>,
    #[serde(default)]
    language: Option<String>,
    #[serde(rename = "hdUrl", alias = "hd_url", default)]
    hd_url: Option<String>,
    #[serde(rename = "sdUrl", alias = "sd_url", default)]
    sd_url: Option<String>,
    #[serde(rename = "embedUrl", alias = "embed_url", default)]
    embed_url: Option<String>,
    #[serde(rename = "streamUrl", alias = "stream_url", alias = "url", default)]
    stream_url: Option<String>,
}
