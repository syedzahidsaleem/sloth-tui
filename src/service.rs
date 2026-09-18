//! Central backend service coordinating providers, cache, and HTTP calls.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::models::BrowseMetrics;
use crate::providers::Provider;
use crate::providers::bdix::circleftp::CircleFtpClient;
use crate::providers::bdix::dhakaflix::client::DhakaFlixClient;
use crate::providers::fourkhdhub::FourKHdHubClient;
use crate::providers::models::{CatalogItem, MediaDetails, ProviderError, ProviderKind};
use crate::providers::moviebox::client::MovieBoxClient;

/// Central streaming and metadata service.
#[derive(Clone)]
pub struct MovieBoxService {
    /// MovieBox provider client.
    pub client: MovieBoxClient,
    /// 4KHDHub provider client.
    pub fourk_client: Option<FourKHdHubClient>,
    /// BDIX CircleFTP provider client.
    pub circleftp_client: CircleFtpClient,
    /// BDIX DhakaFlix provider client.
    pub dhakaflix_client: DhakaFlixClient,
    /// Stremio Addons provider client.
    pub addon_client: crate::providers::addons::AddonClient,
    /// Reusable HTTP client with pooled connections.
    pub http_client: reqwest::Client,
}

/// Type alias for application service.
pub type SlothService = MovieBoxService;

impl Default for MovieBoxService {
    fn default() -> Self {
        Self::new()
    }
}

impl MovieBoxService {
    /// Creates a new service instance initializing all client connections.
    pub fn new() -> Self {
        let http_client = crate::net::http_client_builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            client: MovieBoxClient::new(),
            fourk_client: FourKHdHubClient::new().ok(),
            circleftp_client: CircleFtpClient::new(),
            dhakaflix_client: DhakaFlixClient::new(),
            addon_client: crate::providers::addons::AddonClient::new(),
            http_client,
        }
    }

    /// Returns a reference to the shared HTTP client.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Returns capabilities of a specific provider.
    pub fn capabilities(&self, provider: ProviderKind) -> crate::providers::ProviderCapabilities {
        match provider {
            ProviderKind::MovieBox => Provider::capabilities(&self.client),
            ProviderKind::FourKHdHub => self
                .fourk_client
                .as_ref()
                .map(Provider::capabilities)
                .unwrap_or_default(),
            ProviderKind::BdixCircleFtp => Provider::capabilities(&self.circleftp_client),
            ProviderKind::BdixDhakaFlix => Provider::capabilities(&self.dhakaflix_client),
            ProviderKind::Addons => Provider::capabilities(&self.addon_client),
        }
    }

    /// Queries search suggestions for a query string.
    pub async fn suggest(&self, query: &str) -> Result<Vec<String>, String> {
        let payload = self
            .client
            .suggest(query)
            .await
            .map_err(|e| e.to_string())?;
        Ok(crate::providers::moviebox::adapt::moviebox_suggest_json_to_strings(&payload))
    }

    /// Searches the specified provider, returning parsed catalog items.
    pub async fn search_typed(
        &self,
        provider: ProviderKind,
        query: &str,
        page: usize,
    ) -> Result<Vec<CatalogItem>, ProviderError> {
        match provider {
            ProviderKind::MovieBox => self.client.search_catalog(query, page).await,
            ProviderKind::FourKHdHub => {
                let fourk = self.fourk_client.as_ref().ok_or_else(|| {
                    ProviderError::Unavailable("4KHDHub is unavailable".to_string())
                })?;
                fourk.search_catalog(query, page).await
            }
            ProviderKind::BdixCircleFtp => {
                self.circleftp_client.search_catalog(query, page).await
            }
            ProviderKind::BdixDhakaFlix => {
                self.dhakaflix_client.search_catalog(query, page).await
            }
            ProviderKind::Addons => self.addon_client.search_catalog(query, page).await,
        }
    }

    /// Fetches catalog items from a specific Stremio addon manifest and catalog.
    pub async fn fetch_addon_catalog(
        &self,
        manifest_url: &str,
        r#type: &str,
        catalog_id: &str,
    ) -> Result<Vec<CatalogItem>, String> {
        let manifest_clone = manifest_url.to_string();
        let type_clone = r#type.to_string();
        let cat_id_clone = catalog_id.to_string();
        if let Ok(Some(cached)) = tokio::task::spawn_blocking(move || {
            crate::cache::get_addon_catalog_cache_typed(&manifest_clone, &type_clone, &cat_id_clone)
        })
        .await
        {
            return Ok(cached);
        }

        let base_url = crate::providers::addons::AddonClient::base_addon_url(manifest_url);
        let metas = self
            .addon_client
            .fetch_catalog(&base_url, r#type, catalog_id, None)
            .await
            .map_err(|e| e.to_string())?;

        if metas.is_empty() {
            return Err("No catalog items found".to_string());
        }

        let items: Vec<CatalogItem> = metas
            .iter()
            .map(crate::providers::addons::adapter::meta_to_catalog_item)
            .collect();

        let manifest_clone = manifest_url.to_string();
        let type_clone = r#type.to_string();
        let cat_id_clone = catalog_id.to_string();
        let items_clone = items.clone();
        tokio::task::spawn_blocking(move || {
            crate::cache::set_addon_catalog_cache_typed(
                &manifest_clone,
                &type_clone,
                &cat_id_clone,
                &items_clone,
            );
        });
        Ok(items)
    }

    /// Queries detailed metadata for a subject on the given provider.
    pub async fn details_typed(
        &self,
        provider: ProviderKind,
        subject_id: &str,
    ) -> Result<MediaDetails, ProviderError> {
        match provider {
            ProviderKind::MovieBox => self.client.details(subject_id).await,
            ProviderKind::FourKHdHub => {
                let fourk = self.fourk_client.as_ref().ok_or_else(|| {
                    ProviderError::Unavailable("4KHDHub is unavailable".to_string())
                })?;
                fourk.details(subject_id).await
            }
            ProviderKind::BdixCircleFtp => {
                self.circleftp_client.details(subject_id).await
            }
            ProviderKind::BdixDhakaFlix => {
                self.dhakaflix_client.details(subject_id).await
            }
            ProviderKind::Addons => self.addon_client.details(subject_id).await,
        }
    }

    /// Fetches homepage catalog items and metrics for a specific tab ID.
    pub async fn homepage(
        &self,
        tab_id: &str,
        page: usize,
    ) -> Result<
        (
            Vec<CatalogItem>,
            std::collections::HashMap<String, BrowseMetrics>,
        ),
        String,
    > {
        let payload = self
            .client
            .get_homepage(tab_id, page)
            .await
            .map_err(|e| e.to_string())?;
        Ok(crate::providers::moviebox::adapt::moviebox_homepage_json_to_catalog(&payload))
    }

    /// Queries available video resolutions for a subject collection.
    pub async fn fetch_collection_resolutions(&self, subject_id: &str) -> Result<Vec<u32>, String> {
        self.client
            .fetch_collection_resolutions(subject_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Queries external subtitles with deduplication and language sorting.
    pub async fn get_ext_captions(
        &self,
        subject_id: &str,
        resource_id: &str,
        sibling_ids: &[String],
        season: usize,
        episode: usize,
    ) -> Result<Vec<crate::providers::models::SubtitleOption>, String> {
        let mut all_captions = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        if let Ok(payload) = self.client.get_ext_captions(subject_id, resource_id).await {
            Self::append_unique_captions(
                &mut all_captions,
                &mut seen_urls,
                crate::providers::moviebox::adapt::captions_json_to_options(&payload),
            );
        }

        if all_captions.len() < 5 && !sibling_ids.is_empty() {
            let mut sibling_futs = Vec::new();
            for sib in sibling_ids.iter().take(3) {
                if !sib.is_empty() && sib != subject_id {
                    let client = &self.client;
                    sibling_futs.push(async move {
                        let page = if episode > 0 { (episode - 1) / 20 + 1 } else { 1 };
                        if let Ok((items, _)) = client.fetch_resource_page(sib, season, episode, 0, page).await {
                            let matched_item = find_matching_resource_item(&items, season, episode);
                            if let Some(item) = matched_item {
                                let item_rid = item
                                    .get("resourceId")
                                    .or_else(|| item.get("id"))
                                    .and_then(|v| {
                                        if let Some(n) = v.as_i64() {
                                            Some(n.to_string())
                                        } else if let Some(n) = v.as_u64() {
                                            Some(n.to_string())
                                        } else {
                                            v.as_str().map(|s| s.to_string())
                                        }
                                    });
                                if let Some(rid) = item_rid {
                                    if let Ok(res_payload) = client.get_ext_captions(sib, &rid).await {
                                        return crate::providers::moviebox::adapt::captions_json_to_options(
                                            &res_payload,
                                        );
                                    }
                                }
                            }
                        }
                        Vec::new()
                    });
                }
            }

            if !sibling_futs.is_empty() {
                let sibling_results = futures::future::join_all(sibling_futs).await;
                for res in sibling_results {
                    Self::append_unique_captions(&mut all_captions, &mut seen_urls, res);
                }
            }
        }
        let mut deduplicated: Vec<crate::providers::models::SubtitleOption> = Vec::new();
        let mut seen_languages = std::collections::HashSet::new();
        for sub in all_captions {
            let sanitized_lang = crate::tui::text::sanitize_language_label(&sub.name);
            if seen_languages.insert(sanitized_lang) {
                deduplicated.push(sub);
            }
        }

        deduplicated.sort_by(|a, b| {
            let clean_a = crate::tui::text::sanitize_language_label(&a.name);
            let clean_b = crate::tui::text::sanitize_language_label(&b.name);
            if clean_a.eq_ignore_ascii_case("english") {
                std::cmp::Ordering::Less
            } else if clean_b.eq_ignore_ascii_case("english") {
                std::cmp::Ordering::Greater
            } else {
                clean_a.cmp(&clean_b)
            }
        });

        Ok(deduplicated)
    }

    fn append_unique_captions(
        target: &mut Vec<crate::providers::models::SubtitleOption>,
        seen_urls: &mut std::collections::HashSet<String>,
        opts: Vec<crate::providers::models::SubtitleOption>,
    ) {
        for opt in opts {
            if seen_urls.insert(opt.url.clone()) {
                target.push(opt);
            }
        }
    }

    /// Fetches poster raw bytes from remote URL.
    pub async fn fetch_poster_bytes(&self, url: &str) -> Option<Vec<u8>> {
        let response = self
            .http_client
            .get(url)
            .header("User-Agent", crate::net::APP_HTTP_USER_AGENT)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        Some(response.bytes().await.ok()?.to_vec())
    }

    /// Downloads a subtitle file to a temporary file on disk.
    pub async fn download_subtitle_file(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<PathBuf, String> {
        let mut request = self.http_client.get(url);
        for (name, value) in headers {
            request = request.header(name.as_str(), value.as_str());
        }

        let response = tokio::time::timeout(std::time::Duration::from_secs(8), request.send())
            .await
            .map_err(|_| "Subtitle download timed out".to_string())?
            .map_err(|e| format!("Failed to request subtitle: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Subtitle response status error: {e}"))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read subtitle bytes: {e}"))?;

        let extension = url
            .rsplit('.')
            .next()
            .map(|e| e.to_ascii_lowercase())
            .filter(|e| matches!(e.as_str(), "srt" | "vtt" | "ass" | "ssa" | "sub"))
            .unwrap_or_else(|| "srt".to_string());

        let base_dir = resolve_subtitle_dir();
        let _ = std::fs::create_dir_all(&base_dir);

        let path = base_dir.join(format!(
            "{}_{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            extension
        ));

        tokio::fs::write(&path, bytes)
            .await
            .map_err(|e| format!("Failed to write subtitle file: {e}"))?;

        Ok(path)
    }
}

/// Decodes and downscales raw image bytes to an in-memory DynamicImage.
pub async fn decode_poster(bytes: Vec<u8>) -> Option<Arc<image::DynamicImage>> {
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).ok()?;
        const MAX_DIM: u32 = 512;
        let downscaled = if img.width().max(img.height()) <= MAX_DIM {
            img
        } else {
            img.resize(MAX_DIM, MAX_DIM, image::imageops::FilterType::Triangle)
        };
        Some(Arc::new(downscaled))
    })
    .await
    .ok()?
}

/// Extracts a float metric from nested JSON metadata.
pub fn metric_value(item: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    let mut containers = vec![item];
    if let Some(metadata) = item.get("metadata") {
        containers.push(metadata);
    }
    if let Some(meta) = item.get("meta") {
        containers.push(meta);
    }

    containers.into_iter().find_map(|container| {
        keys.iter().find_map(|key| {
            let value = container.get(*key)?;
            value
                .as_f64()
                .or_else(|| value.as_i64().map(|number| number as f64))
                .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
        })
    })
}

/// Extracts all browse sorting metrics from a JSON catalog record.
pub fn extract_browse_metrics(item: &serde_json::Value) -> BrowseMetrics {
    BrowseMetrics {
        trending: metric_value(
            item,
            &["__browse_rank", "imdb_trending", "imdbTrending", "trending"],
        ),
        rating: metric_value(
            item,
            &["imdbRatingValue", "imdbRate", "imdb_rating", "imdbRating"],
        ),
        recent_rating: metric_value(
            item,
            &[
                "imdb_rating_30d",
                "imdbRating30Days",
                "imdbRatingLast30Days",
                "imdb_rating_recent",
                "imdbRatingValue",
                "imdbRate",
            ],
        ),
        popularity: metric_value(
            item,
            &[
                "__browse_rank",
                "imdb_popularity",
                "imdbPopularity",
                "popularity",
                "viewers",
            ],
        ),
    }
}

fn is_termux_environment() -> bool {
    std::env::var("TERMUX_VERSION").is_ok()
        || std::env::var("PREFIX")
            .map(|p| p.contains("com.termux"))
            .unwrap_or(false)
}

/// Returns appropriate directory for cached subtitle files.
pub fn resolve_subtitle_dir() -> PathBuf {
    if is_termux_environment() {
        if let Some(home) = dirs::home_dir() {
            let storage = home.join(format!("storage/downloads/{}_subs", crate::config::APP_NAME));
            if home.join("storage/downloads").exists() {
                let _ = std::fs::create_dir_all(&storage);
                return storage;
            }
        }
    }
    crate::config::cache_dir().join("subs")
}

/// Ensures path ends with application subdirectory.
pub fn ensure_moviebox_subdir(path: &Path) -> PathBuf {
    let is_already_app = path
        .file_name()
        .map(|name| {
            let s = name.to_string_lossy();
            s.eq_ignore_ascii_case("Sloth-TUI")
                || s.eq_ignore_ascii_case("Sloth")
                || s.eq_ignore_ascii_case("MovieBox-TUI")
                || s.eq_ignore_ascii_case("MovieBox")
        })
        .unwrap_or(false);

    if is_already_app {
        path.to_path_buf()
    } else {
        path.join("Sloth-TUI")
    }
}

/// Resolves default download directory for media files.
pub fn resolve_download_dir(custom_dir: Option<&Path>) -> PathBuf {
    if let Some(custom) = custom_dir {
        if std::fs::create_dir_all(custom).is_ok() {
            let probe = custom.join(format!(".sloth_probe_{}", std::process::id()));
            if std::fs::write(&probe, b"ok").is_ok() {
                let _ = std::fs::remove_file(&probe);
                return ensure_moviebox_subdir(custom);
            }
        }
    }

    let base_dir = dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."));

    if is_termux_environment() {
        if let Some(home) = dirs::home_dir() {
            let android_storage = home.join("storage/downloads");
            if android_storage.exists() {
                return ensure_moviebox_subdir(&android_storage);
            }
        }
    }

    ensure_moviebox_subdir(&base_dir)
}

/// Finds the resource item corresponding to specific season and episode.
pub fn find_matching_resource_item(
    items: &[serde_json::Value],
    season: usize,
    episode: usize,
) -> Option<&serde_json::Value> {
    items.iter().find(|item| {
        let parse_num = |k: &str| -> Option<usize> {
            item.get(k).and_then(|v| {
                v.as_u64()
                    .map(|n| n as usize)
                    .or_else(|| v.as_i64().map(|n| n as usize))
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
        };
        let se = parse_num("se");
        let ep_num = parse_num("ep");
        (season == 0 && episode == 0) || (se == Some(season) && ep_num == Some(episode))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decode_poster_caps_dimensions() {
        let mut buffer = std::io::Cursor::new(Vec::new());
        let img = image::DynamicImage::new_rgb8(800, 600);
        img.write_to(&mut buffer, image::ImageFormat::Png).unwrap();
        let bytes = buffer.into_inner();

        let decoded = decode_poster(bytes).await.unwrap();
        assert!(decoded.width() <= 512);
        assert!(decoded.height() <= 512);
        assert_eq!(decoded.width(), 512);
        assert_eq!(decoded.height(), 384);
    }

    #[test]
    fn test_find_matching_resource_item_integer_and_string_values() {
        let items = vec![
            serde_json::json!({
                "id": "1001",
                "se": 1,
                "ep": 1
            }),
            serde_json::json!({
                "id": "1002",
                "se": "2",
                "ep": "5"
            }),
        ];

        let matched = find_matching_resource_item(&items, 2, 5);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().get("id").unwrap().as_str(), Some("1002"));
    }

    #[test]
    fn test_find_matching_resource_item_series_does_not_fall_back_to_episode_one() {
        let items = vec![
            serde_json::json!({
                "id": "1001",
                "se": 1,
                "ep": 1
            }),
            serde_json::json!({
                "id": "1002",
                "se": 1,
                "ep": 2
            }),
        ];

        let matched = find_matching_resource_item(&items, 2, 3);
        assert!(matched.is_none());
    }

    #[test]
    fn test_find_matching_resource_item_movie_matches_first() {
        let items = vec![
            serde_json::json!({
                "id": "movie_res_1"
            }),
            serde_json::json!({
                "id": "movie_res_2"
            }),
        ];

        let matched = find_matching_resource_item(&items, 0, 0);
        assert!(matched.is_some());
        assert_eq!(
            matched.unwrap().get("id").unwrap().as_str(),
            Some("movie_res_1")
        );
    }
}
