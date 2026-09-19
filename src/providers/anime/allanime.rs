//! AllAnime provider implementation using GraphQL API.
//!
//! Connects to the AllAnime GraphQL endpoint to search anime titles,
//! list episodes, and resolve media playback stream URLs.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::providers::models::{
    EpisodeRef, ExternalIds, Media, MediaType, ProviderError, Quality, StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities};

const DEFAULT_BASE_URL: &str = "https://api.allanime.day";
const USER_AGENT: &str = "Sloth-TUI/0.1.0";
const REFERER: &str = "https://allanime.to";

/// AllAnime GraphQL media provider implementation.
pub struct AllAnimeProvider {
    client: Arc<reqwest::Client>,
    base_url: String,
    is_dub: AtomicBool,
}

impl AllAnimeProvider {
    /// Creates a new `AllAnimeProvider` using default client and base URL.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self {
            client: Arc::new(client),
            base_url: DEFAULT_BASE_URL.to_string(),
            is_dub: AtomicBool::new(false),
        }
    }

    /// Creates a new `AllAnimeProvider` with a shared HTTP client.
    pub fn with_client(client: Arc<reqwest::Client>) -> Self {
        Self {
            client,
            base_url: DEFAULT_BASE_URL.to_string(),
            is_dub: AtomicBool::new(false),
        }
    }

    /// Creates a new `AllAnimeProvider` with custom client and base URL (useful for test mocks).
    pub fn with_base_url(client: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            is_dub: AtomicBool::new(false),
        }
    }

    /// Toggles or sets dubbed (`true`) or subbed (`false`) stream preference.
    pub fn set_dub(&self, dub: bool) {
        self.is_dub.store(dub, Ordering::Relaxed);
    }

    /// Returns whether the provider is currently set to request dubbed streams.
    pub fn is_dub(&self) -> bool {
        self.is_dub.load(Ordering::Relaxed)
    }

    /// Private helper method to execute GraphQL POST queries against the API endpoint.
    async fn gql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, ProviderError> {
        let endpoint = if self.base_url.ends_with("/api") {
            self.base_url.clone()
        } else {
            format!("{}/api", self.base_url.trim_end_matches('/'))
        };

        let body = json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .client
            .post(&endpoint)
            .header("User-Agent", USER_AGENT)
            .header("Referer", REFERER)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "AllAnime GraphQL query failed with HTTP status: {}",
                response.status()
            )));
        }

        let resp_text = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let gql_response: GraphQLResponse<T> = serde_json::from_str(&resp_text).map_err(|e| {
            ProviderError::Parsing(format!("Failed to deserialize AllAnime GraphQL response: {e}"))
        })?;

        if let Some(data) = gql_response.data {
            Ok(data)
        } else if let Some(errors) = gql_response.errors {
            let error_msgs = errors
                .into_iter()
                .map(|err| err.message)
                .collect::<Vec<_>>()
                .join("; ");
            Err(ProviderError::Parsing(format!("AllAnime GraphQL error: {error_msgs}")))
        } else {
            Err(ProviderError::NotFound)
        }
    }
}

impl Default for AllAnimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AllAnimeProvider {
    fn id(&self) -> &'static str {
        "allanime"
    }

    fn name(&self) -> &'static str {
        "AllAnime"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            movies: false,
            tv_series: false,
            anime: true,
            live_sports: false,
            iptv: false,
            f1: false,
            subtitles: true,
            download: true,
            quality_selection: true,
            dub_sub_toggle: true,
            supports_search: true,
            supports_pagination: false,
            supports_series: true,
            supports_subtitles: true,
            supports_homepage: false,
        }
    }

    async fn search(&self, query: &str, _kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let search_query = r#"
            query ($search: SearchInput, $limit: Int, $page: Int) {
              shows(
                search: $search
                limit: $limit
                page: $page
                translationType: sub
              ) {
                edges {
                  _id
                  name
                  englishName
                  thumbnail
                  score
                  availableEpisodes { sub dub }
                }
              }
            }
        "#;

        let variables = json!({
            "search": { "query": query },
            "limit": 20,
            "page": 1,
        });

        let data: ShowsData = self.gql(search_query, variables).await?;
        let edges = data.shows.map(|s| s.edges).unwrap_or_default();

        let media_items = edges
            .into_iter()
            .map(|edge| {
                let title = edge
                    .english_name
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(edge.name);
                let episodes_count = edge
                    .available_episodes
                    .as_ref()
                    .and_then(|e| e.sub.or(e.dub));

                Media {
                    id: edge.id,
                    title,
                    media_type: MediaType::Anime,
                    year: None,
                    overview: None,
                    poster_url: edge.thumbnail,
                    backdrop_url: None,
                    genres: Vec::new(),
                    rating: edge.score,
                    duration_secs: None,
                    seasons_count: Some(1),
                    episodes_count,
                    provider_id: "allanime",
                    external_ids: ExternalIds::default(),
                    cast: Vec::new(),
                }
            })
            .collect();

        Ok(media_items)
    }

    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let ep_num = episode.map(|e| e.episode).unwrap_or(1);
        let translation_type = if self.is_dub.load(Ordering::Relaxed) {
            "dub"
        } else {
            "sub"
        };

        let episode_query = r#"
            query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
              episode(
                showId: $showId
                translationType: $translationType
                episodeString: $episodeString
              ) {
                sourceUrls
              }
            }
        "#;

        let variables = json!({
            "showId": media.id,
            "translationType": translation_type,
            "episodeString": ep_num.to_string(),
        });

        let data: EpisodeData = self.gql(episode_query, variables).await?;
        let episode_info = data.episode.ok_or(ProviderError::NotFound)?;
        let source_urls = episode_info.source_urls.unwrap_or_default();

        if source_urls.is_empty() {
            return Err(ProviderError::NotFound);
        }

        let mut streams = Vec::new();
        for item in source_urls {
            let raw_url = match item {
                SourceUrlEntry::Object { source_url, .. } => source_url,
                SourceUrlEntry::String(url) => url,
            };

            if let Some(decoded) = decode_source_url(&raw_url) {
                let is_hls = decoded.contains(".m3u8");
                streams.push(StreamUrl {
                    url: decoded,
                    quality: Quality::Auto,
                    is_hls,
                    headers: vec![
                        ("Referer".into(), REFERER.into()),
                        ("User-Agent".into(), USER_AGENT.into()),
                    ],
                    subtitle_url: None,
                    provider_id: "allanime",
                });
            }
        }

        if streams.is_empty() {
            return Err(ProviderError::NotFound);
        }

        Ok(streams)
    }

    async fn episodes(
        &self,
        media: &Media,
        season: u32,
    ) -> Result<Vec<EpisodeRef>, ProviderError> {
        let count = media.episodes_count.unwrap_or(1);
        let s = if season == 0 { 1 } else { season };
        let list = (1..=count)
            .map(|ep| EpisodeRef {
                season: s,
                episode: ep,
                title: Some(format!("Episode {ep}")),
                duration_secs: None,
            })
            .collect();
        Ok(list)
    }

    async fn health(&self) -> bool {
        match self
            .client
            .get(&self.base_url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", REFERER)
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success() || resp.status().as_u16() < 500,
            Err(_) => false,
        }
    }
}

/// Helper function to decode and normalize AllAnime source URLs (base64, hex/xor, clock, or plain).
fn decode_source_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }

    // Attempt base64 decoding
    if let Ok(decoded_bytes) = STANDARD.decode(trimmed) {
        if let Ok(s) = String::from_utf8(decoded_bytes) {
            if s.starts_with("http://") || s.starts_with("https://") {
                return Some(s);
            }
        }
    }

    // Attempt hex XOR decoding (AllAnime hex obfuscation with "--" prefix)
    if let Some(hex_part) = trimmed.strip_prefix("--") {
        if let Ok(bytes) = hex_to_bytes(hex_part) {
            let xor_bytes: Vec<u8> = bytes.iter().map(|b| b ^ 56).collect();
            if let Ok(s) = String::from_utf8(xor_bytes) {
                if s.starts_with("http://") || s.starts_with("https://") {
                    return Some(s);
                }
            }
            if let Ok(s) = String::from_utf8(bytes) {
                if s.starts_with("http://") || s.starts_with("https://") {
                    return Some(s);
                }
            }
        }
    }

    // Clock prefix: e.g. "/clock?id=..." or "clock?id=..."
    if trimmed.starts_with("/clock") || trimmed.starts_with("clock") {
        return Some(format!(
            "https://allanime.day/{}",
            trimmed.trim_start_matches('/')
        ));
    }

    Some(trimmed.to_string())
}

/// Converts a hex string into raw byte vector.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ()> {
    if hex.len() % 2 != 0 {
        return Err(());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

// ---------------------------------------------------------------------------
// Private GraphQL Serde structures
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    errors: Option<Vec<GraphQLErrorItem>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLErrorItem {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ShowsData {
    #[serde(default)]
    shows: Option<ShowsContainer>,
}

#[derive(Debug, Deserialize)]
struct ShowsContainer {
    #[serde(default)]
    edges: Vec<ShowEdge>,
}

#[derive(Debug, Deserialize)]
struct ShowEdge {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    #[serde(rename = "englishName", default)]
    english_name: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    score: Option<f32>,
    #[serde(rename = "availableEpisodes", default)]
    available_episodes: Option<AvailableEpisodes>,
}

#[derive(Debug, Deserialize)]
struct AvailableEpisodes {
    #[serde(default)]
    sub: Option<u32>,
    #[serde(default)]
    dub: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct EpisodeData {
    #[serde(default)]
    episode: Option<EpisodeContainer>,
}

#[derive(Debug, Deserialize)]
struct EpisodeContainer {
    #[serde(rename = "sourceUrls", default)]
    source_urls: Option<Vec<SourceUrlEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SourceUrlEntry {
    Object {
        #[serde(rename = "sourceUrl")]
        source_url: String,
        #[serde(default)]
        _priority: Option<f64>,
    },
    String(String),
}
