//! HiAnime anime scraper and stream resolver.
//!
//! Connects to the active hianime.at catalog and stream servers,
//! extracting direct 1080p HLS video playlists and subtitles via ZokoAnime XOR deobfuscation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::providers::models::{
    EpisodeRef, ExternalIds, Media, MediaType, ProviderError, Quality, StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities};

const DEFAULT_BASE_URL: &str = "https://hianime.at";
const USER_AGENT: &str = crate::net::DEFAULT_BROWSER_USER_AGENT;

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

    /// Creates a new `HiAnimeProvider` with a custom HTTP client and base URL.
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

    /// Extracts the numeric ID from an anime identifier (e.g. "solo-leveling-18718" -> "18718").
    fn extract_numeric_id(id: &str) -> &str {
        if let Some(pos) = id.rfind('-') {
            let suffix = &id[pos + 1..];
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                return suffix;
            }
        }
        id
    }

    /// Internal helper to fetch episodes list for an anime id from the theme API.
    async fn fetch_episodes_data(&self, anime_id: &str) -> Result<Vec<EpisodeInfo>, ProviderError> {
        let numeric_id = Self::extract_numeric_id(anime_id);
        let url = format!(
            "{}/api/theme/episode/list/{}",
            self.base_url.trim_end_matches('/'),
            numeric_id
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", format!("{}/watch/{}", self.base_url, anime_id))
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

        let parsed: ApiResponse = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse HiAnime episodes JSON: {e}"))
        })?;

        let html = parsed.html.unwrap_or_default();
        let document = Html::parse_document(&html);
        let item_selector =
            Selector::parse("a.ep-item").map_err(|e| ProviderError::Parsing(e.to_string()))?;

        let mut episodes = Vec::new();
        for el in document.select(&item_selector) {
            let data_id = el.value().attr("data-id").unwrap_or_default().to_string();
            let data_number = el
                .value()
                .attr("data-number")
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or((episodes.len() + 1) as u32);
            let title = el
                .value()
                .attr("title")
                .map(|t| t.to_string())
                .or_else(|| Some(format!("Episode {data_number}")));

            if !data_id.is_empty() {
                episodes.push(EpisodeInfo {
                    episode_id: data_id,
                    number: data_number,
                    title,
                });
            }
        }

        if episodes.is_empty() {
            Err(ProviderError::NotFound)
        } else {
            Ok(episodes)
        }
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
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        let encoded_query =
            percent_encoding::utf8_percent_encode(q, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let url = format!(
            "{}/search?keyword={}",
            self.base_url.trim_end_matches('/'),
            encoded_query
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "HiAnime search failed with status: {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let document = Html::parse_document(&html);
        let item_selector = Selector::parse(".flw-item, .film-detail")
            .map_err(|e| ProviderError::Parsing(e.to_string()))?;
        let name_selector =
            Selector::parse("h3.film-name a").map_err(|e| ProviderError::Parsing(e.to_string()))?;
        let img_selector = Selector::parse(".film-poster-img")
            .map_err(|e| ProviderError::Parsing(e.to_string()))?;
        let ep_sub_selector =
            Selector::parse(".tick-sub").map_err(|e| ProviderError::Parsing(e.to_string()))?;
        let ep_dub_selector =
            Selector::parse(".tick-dub").map_err(|e| ProviderError::Parsing(e.to_string()))?;

        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for item in document.select(&item_selector) {
            let Some(link) = item.select(&name_selector).next() else {
                continue;
            };
            let title = link.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let href = link.value().attr("href").unwrap_or_default();
            let id = href
                .trim_start_matches("/watch/")
                .trim_matches('/')
                .to_string();
            if id.is_empty() || !seen.insert(id.clone()) {
                continue;
            }

            let poster_url = item.select(&img_selector).next().and_then(|img| {
                img.value()
                    .attr("data-src")
                    .or_else(|| img.value().attr("src"))
                    .map(str::to_string)
            });

            let sub_count = item
                .select(&ep_sub_selector)
                .next()
                .and_then(|el| el.text().collect::<String>().trim().parse::<u32>().ok());
            let dub_count = item
                .select(&ep_dub_selector)
                .next()
                .and_then(|el| el.text().collect::<String>().trim().parse::<u32>().ok());
            let episodes_count = sub_count.or(dub_count);

            results.push(Media {
                id,
                title,
                media_type: MediaType::Anime,
                year: None,
                overview: None,
                poster_url,
                backdrop_url: None,
                genres: Vec::new(),
                rating: None,
                duration_secs: None,
                seasons_count: Some(1),
                episodes_count,
                provider_id: "hianime",
                external_ids: ExternalIds::default(),
                cast: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let ep_list = self.fetch_episodes_data(&media.id).await?;
        let target_ep_num = episode.map(|e| e.episode).unwrap_or(1);

        let target_ep = ep_list
            .iter()
            .find(|e| e.number == target_ep_num)
            .or_else(|| ep_list.first())
            .ok_or(ProviderError::NotFound)?;

        let servers_url = format!(
            "{}/api/theme/episode/servers?episodeId={}",
            self.base_url.trim_end_matches('/'),
            target_ep.episode_id
        );

        let response = self
            .client
            .get(&servers_url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", format!("{}/watch/{}", self.base_url, media.id))
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "HiAnime servers request failed with status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let parsed: ApiResponse = serde_json::from_str(&body).map_err(|e| {
            ProviderError::Parsing(format!("Failed to parse HiAnime servers JSON: {e}"))
        })?;

        let html = parsed.html.unwrap_or_default();
        let is_dub = self.is_dub.load(Ordering::Relaxed);
        let embed_urls = parse_server_embed_urls(&html, is_dub)?;

        let mut stream_urls = Vec::new();

        for embed_url in embed_urls {
            if embed_url.contains("zokoanime.") {
                if let Ok(resp) = self
                    .client
                    .get(&embed_url)
                    .header("User-Agent", USER_AGENT)
                    .header("Referer", format!("{}/", self.base_url))
                    .send()
                    .await
                {
                    if let Ok(page_html) = resp.text().await {
                        if let Some(blob) = extract_window_p_blob(&page_html) {
                            if let Some(payload) = decode_zokoanime_blob(&blob) {
                                if let Some(direct_src) = payload.src {
                                    let subtitle_url = payload
                                        .subtitles
                                        .iter()
                                        .find(|s| {
                                            s.lang.as_deref() == Some("en")
                                                || s.label
                                                    .as_deref()
                                                    .map_or(false, |l| l.contains("English"))
                                        })
                                        .and_then(|s| s.src.clone());

                                    let is_hls = direct_src.contains(".m3u8");
                                    stream_urls.push(StreamUrl {
                                        url: direct_src,
                                        quality: Quality::FHD1080,
                                        is_hls,
                                        headers: vec![
                                            ("Referer".into(), "https://zokoanime.video/".into()),
                                            ("User-Agent".into(), USER_AGENT.into()),
                                        ],
                                        subtitle_url,
                                        provider_id: "hianime",
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                // Megaplay or direct embed
                stream_urls.push(StreamUrl {
                    url: embed_url,
                    quality: Quality::Auto,
                    is_hls: false,
                    headers: vec![
                        ("Referer".into(), self.base_url.clone()),
                        ("User-Agent".into(), USER_AGENT.into()),
                    ],
                    subtitle_url: None,
                    provider_id: "hianime",
                });
            }
        }

        if stream_urls.is_empty() {
            Err(ProviderError::NotFound)
        } else {
            Ok(stream_urls)
        }
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

fn parse_server_embed_urls(html: &str, is_dub: bool) -> Result<Vec<String>, ProviderError> {
    use base64::Engine;
    let document = Html::parse_document(html);
    let server_selector =
        Selector::parse(".server-item").map_err(|e| ProviderError::Parsing(e.to_string()))?;

    let desired_type = if is_dub { "dub" } else { "sub" };
    let mut embed_urls = Vec::new();

    for el in document.select(&server_selector) {
        let server_type = el.value().attr("data-type").unwrap_or_default();
        if server_type != desired_type && !server_type.is_empty() {
            continue;
        }

        if let Some(hash) = el.value().attr("data-hash") {
            if let Ok(decoded_bytes) =
                base64::engine::general_purpose::STANDARD.decode(hash.as_bytes())
            {
                if let Ok(embed_url) = String::from_utf8(decoded_bytes) {
                    if embed_url.starts_with("https://") {
                        embed_urls.push(embed_url);
                    }
                }
            }
        }
    }

    if embed_urls.is_empty() {
        for el in document.select(&server_selector) {
            if let Some(hash) = el.value().attr("data-hash") {
                if let Ok(decoded_bytes) =
                    base64::engine::general_purpose::STANDARD.decode(hash.as_bytes())
                {
                    if let Ok(embed_url) = String::from_utf8(decoded_bytes) {
                        if embed_url.starts_with("https://") {
                            embed_urls.push(embed_url);
                        }
                    }
                }
            }
        }
    }

    Ok(embed_urls)
}

fn extract_window_p_blob(html: &str) -> Option<String> {
    for needle in &[
        "window.__P=\"",
        "window.__P = \"",
        "window.__P='",
        "window.__P = '",
    ] {
        if let Some(idx) = html.find(needle) {
            let start = idx + needle.len();
            let end_char = if needle.ends_with('\'') { '\'' } else { '"' };
            if let Some(end) = html[start..].find(end_char) {
                return Some(html[start..start + end].to_string());
            }
        }
    }
    None
}

fn decode_zokoanime_blob(blob: &str) -> Option<ZokoAnimePayload> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(blob.as_bytes())
        .ok()?;
    let key = b"otaku-embed-v1";
    let mut decrypted = vec![0u8; bytes.len()];
    for (i, b) in bytes.iter().enumerate() {
        decrypted[i] = b ^ key[i % key.len()];
    }
    let json_str = String::from_utf8(decrypted).ok()?;
    serde_json::from_str(&json_str).ok()
}

// ---------------------------------------------------------------------------
// Private response and decryption models
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<bool>,
    #[serde(default)]
    html: Option<String>,
}

#[derive(Debug, Clone)]
struct EpisodeInfo {
    episode_id: String,
    number: u32,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZokoAnimePayload {
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    subtitles: Vec<ZokoSubtitle>,
}

#[derive(Debug, Deserialize)]
struct ZokoSubtitle {
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    src: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numeric_id() {
        assert_eq!(
            HiAnimeProvider::extract_numeric_id("solo-leveling-18718"),
            "18718"
        );
        assert_eq!(HiAnimeProvider::extract_numeric_id("one-piece-100"), "100");
        assert_eq!(HiAnimeProvider::extract_numeric_id("52299"), "52299");
        assert_eq!(HiAnimeProvider::extract_numeric_id("naruto"), "naruto");
    }

    #[test]
    fn test_zokoanime_decryption() {
        use base64::Engine;
        let original_json = r#"{"src":"https://example.com/master.m3u8","subtitles":[{"lang":"en","label":"English","src":"https://example.com/sub.vtt"}]}"#;
        let key = b"otaku-embed-v1";
        let mut xor_bytes = vec![0u8; original_json.len()];
        for (i, b) in original_json.bytes().enumerate() {
            xor_bytes[i] = b ^ key[i % key.len()];
        }
        let blob = base64::engine::general_purpose::STANDARD.encode(&xor_bytes);

        let payload = decode_zokoanime_blob(&blob).expect("successful decryption");
        assert_eq!(
            payload.src.as_deref(),
            Some("https://example.com/master.m3u8")
        );
        assert_eq!(payload.subtitles.len(), 1);
        assert_eq!(payload.subtitles[0].label.as_deref(), Some("English"));
    }
}
