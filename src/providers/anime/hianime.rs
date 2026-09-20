//! HiAnime anime scraper and stream resolver.
//!
//! Connects to the HiAnime public JSON API to search anime catalog,
//! list episodes, and resolve HLS stream URLs with subtitle tracks.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use serde::Deserialize;

use crate::providers::models::{
    EpisodeRef, ExternalIds, Media, MediaType, ProviderError, Quality, StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities};

const DEFAULT_BASE_URL: &str = "https://hianime.to";
const USER_AGENT: &str = "Sloth-TUI/0.1.0";

/// HiAnime media provider implementation.
pub struct HiAnimeProvider {
    client: Arc<reqwest::Client>,
    base_url: String,
    is_dub: AtomicBool,
}

impl HiAnimeProvider {
    /// Creates a new `HiAnimeProvider` using the default base URL and client settings.
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

    /// Creates a new `HiAnimeProvider` with a shared HTTP client.
    pub fn with_client(client: Arc<reqwest::Client>) -> Self {
        Self {
            client,
            base_url: DEFAULT_BASE_URL.to_string(),
            is_dub: AtomicBool::new(false),
        }
    }

    /// Creates a new `HiAnimeProvider` with a custom HTTP client and base URL (useful for tests).
    pub fn with_base_url(client: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            is_dub: AtomicBool::new(false),
        }
    }

    /// Toggles or sets the audio preference to dubbed (`true`) or subbed (`false`).
    pub fn set_dub(&self, dub: bool) {
        self.is_dub.store(dub, Ordering::Relaxed);
    }

    /// Returns whether the provider is currently set to request dubbed streams.
    pub fn is_dub(&self) -> bool {
        self.is_dub.load(Ordering::Relaxed)
    }

    /// Internal helper to fetch episodes list for an anime id.
    async fn fetch_episodes_data(&self, anime_id: &str) -> Result<Vec<EpisodeItem>, ProviderError> {
        let url = format!(
            "{}/api/v2/hianime/episodes/{}",
            self.base_url.trim_end_matches('/'),
            anime_id
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
                return Err(ProviderError::NotFound);
            }
            return Err(ProviderError::Unavailable(format!(
                "HiAnime episode request failed with status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let parsed: EpisodesResponse = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse HiAnime episodes list: {e}"))
        })?;

        Ok(parsed.data.map(|d| d.episodes).unwrap_or_default())
    }
}

impl Default for HiAnimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for HiAnimeProvider {
    fn id(&self) -> &'static str {
        "hianime"
    }

    fn name(&self) -> &'static str {
        "HiAnime"
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

        let url = format!(
            "{}/api/v2/hianime/search",
            self.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .query(&[("q", query), ("page", "1")])
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "HiAnime search failed with status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let parsed: SearchResponse = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse HiAnime search response: {e}"))
        })?;

        let animes = parsed.data.map(|d| d.animes).unwrap_or_default();

        let media_items = animes
            .into_iter()
            .map(|anime| {
                let rating = anime.rating.as_deref().and_then(|r| r.parse::<f32>().ok());
                let episodes_count = anime.episodes.as_ref().and_then(|e| e.sub.or(e.dub));

                Media {
                    id: anime.id,
                    title: anime.name,
                    media_type: MediaType::Anime,
                    year: None,
                    overview: None,
                    poster_url: anime.poster,
                    backdrop_url: None,
                    genres: Vec::new(),
                    rating,
                    duration_secs: None,
                    seasons_count: Some(1),
                    episodes_count,
                    provider_id: "hianime",
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
        let ep_list = self.fetch_episodes_data(&media.id).await?;
        if ep_list.is_empty() {
            return Err(ProviderError::NotFound);
        }

        let target_episode = match episode {
            Some(ep) => ep_list
                .iter()
                .find(|e| e.number == ep.episode)
                .or_else(|| ep_list.first())
                .ok_or(ProviderError::NotFound)?,
            None => ep_list
                .iter()
                .find(|e| e.number == 1)
                .or_else(|| ep_list.first())
                .ok_or(ProviderError::NotFound)?,
        };

        let category = if self.is_dub.load(Ordering::Relaxed) {
            "dub"
        } else {
            "sub"
        };

        let url = format!(
            "{}/api/v2/hianime/episode/sources",
            self.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .query(&[
                ("animeEpisodeId", target_episode.episode_id.as_str()),
                ("server", "hd-1"),
                ("category", category),
            ])
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(ProviderError::NotFound);
            }
            return Err(ProviderError::Unavailable(format!(
                "HiAnime sources request failed with status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let parsed: SourcesResponse = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse HiAnime sources response: {e}"))
        })?;

        let sources_data = parsed.data.ok_or(ProviderError::NotFound)?;
        if sources_data.sources.is_empty() {
            return Err(ProviderError::NotFound);
        }

        let subtitle_url = sources_data
            .tracks
            .into_iter()
            .find(|t| {
                t.kind.as_deref() == Some("captions")
                    && t.label
                        .as_deref()
                        .map_or(false, |l| l.to_lowercase().contains("english"))
            })
            .map(|t| t.file);

        let stream_urls = sources_data
            .sources
            .into_iter()
            .map(|s| {
                let is_hls = s.source_type.as_deref() == Some("hls") || s.url.contains(".m3u8");
                StreamUrl {
                    url: s.url,
                    quality: Quality::Auto,
                    is_hls,
                    headers: vec![
                        ("Referer".into(), "https://hianime.to".into()),
                        ("User-Agent".into(), USER_AGENT.into()),
                    ],
                    subtitle_url: subtitle_url.clone(),
                    provider_id: "hianime",
                }
            })
            .collect();

        Ok(stream_urls)
    }

    async fn episodes(&self, media: &Media, season: u32) -> Result<Vec<EpisodeRef>, ProviderError> {
        let ep_list = self.fetch_episodes_data(&media.id).await?;
        let episodes = ep_list
            .into_iter()
            .map(|ep| EpisodeRef {
                season: if season == 0 { 1 } else { season },
                episode: ep.number,
                title: ep.title,
                duration_secs: None,
            })
            .collect();

        Ok(episodes)
    }

    async fn health(&self) -> bool {
        let url = format!("{}/", self.base_url.trim_end_matches('/'));
        match self
            .client
            .head(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        {
            Ok(resp) => resp.status() == reqwest::StatusCode::OK || resp.status().is_success(),
            Err(_) => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Private serde API response structures
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Option<SearchData>,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    #[serde(default)]
    animes: Vec<AnimeItem>,
}

#[derive(Debug, Deserialize)]
struct AnimeItem {
    id: String,
    name: String,
    #[serde(default)]
    poster: Option<String>,
    #[serde(default)]
    rating: Option<String>,
    #[serde(default)]
    episodes: Option<EpisodesSummary>,
}

#[derive(Debug, Deserialize)]
struct EpisodesSummary {
    #[serde(default)]
    sub: Option<u32>,
    #[serde(default)]
    dub: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct EpisodesResponse {
    data: Option<EpisodesData>,
}

#[derive(Debug, Deserialize)]
struct EpisodesData {
    #[serde(default)]
    episodes: Vec<EpisodeItem>,
}

#[derive(Debug, Deserialize)]
struct EpisodeItem {
    #[serde(rename = "episodeId")]
    episode_id: String,
    #[serde(default)]
    number: u32,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SourcesResponse {
    data: Option<SourcesData>,
}

#[derive(Debug, Deserialize)]
struct SourcesData {
    #[serde(default)]
    sources: Vec<SourceItem>,
    #[serde(default)]
    tracks: Vec<TrackItem>,
}

#[derive(Debug, Deserialize)]
struct SourceItem {
    url: String,
    #[serde(rename = "type", default)]
    source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TrackItem {
    file: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    label: Option<String>,
}
