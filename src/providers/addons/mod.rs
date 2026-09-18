//! Stremio Addons provider and aggregator module.

pub mod adapter;
pub mod aggregator;
pub mod client;
pub mod models;

pub use adapter::{
    meta_detail_to_media_details, meta_to_catalog_item, meta_to_search_result,
    release_to_playback_source, stream_item_to_release,
};
pub use aggregator::aggregate_streams;
pub use client::AddonClient;
pub use models::{AddonManifest, InstalledAddon, MetaDetail, MetaItem, StreamItem};

use async_trait::async_trait;
use crate::providers::models::{
    CatalogItem, EpisodeRef, Media, MediaDetails, MediaType, ProviderError, ProviderKind, Quality,
    StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities};

#[async_trait]
impl Provider for AddonClient {
    fn id(&self) -> &'static str {
        "addons"
    }

    fn name(&self) -> &'static str {
        "Addons"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            movies: true,
            tv_series: true,
            anime: true,
            live_sports: false,
            iptv: false,
            f1: false,
            subtitles: true,
            download: false,
            quality_selection: true,
            dub_sub_toggle: false,
            supports_search: true,
            supports_pagination: false,
            supports_series: true,
            supports_subtitles: true,
            supports_homepage: false,
        }
    }

    async fn search(&self, query: &str, _kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        let catalog = self.search_catalog(query, 1).await?;
        Ok(catalog.into_iter().map(Media::from).collect())
    }

    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let addons = crate::config::load_addons();
        let (season, ep_num) = match episode {
            Some(e) => (e.season as usize, e.episode as usize),
            None => (0, 0),
        };
        let is_series = media.media_type == MediaType::Series;
        let (releases, _) =
            aggregate_streams(self, &addons, &media.id, season, ep_num, is_series).await;
        let mut urls = Vec::new();
        for rel in releases {
            let quality = Quality::from_resolution(rel.resolution_u64());
            for m in rel.mirrors {
                let is_hls = m.resolver_url.contains(".m3u8");
                urls.push(StreamUrl {
                    url: m.resolver_url,
                    quality,
                    is_hls,
                    headers: m.headers,
                    subtitle_url: None,
                    provider_id: "addons",
                });
            }
        }
        Ok(urls)
    }

    async fn health(&self) -> bool {
        true
    }
}

impl AddonClient {
    /// Queries installed catalog/metadata addons.
    pub async fn search_catalog(
        &self,
        query: &str,
        _page: usize,
    ) -> Result<Vec<CatalogItem>, ProviderError> {
        let addons = crate::config::load_addons();
        let catalog_addons: Vec<_> = addons
            .iter()
            .filter(|a| a.enabled && (a.provides_meta || a.provides_catalog))
            .collect();

        if catalog_addons.is_empty() {
            return Err(ProviderError::Unavailable(
                "No catalog/metadata addon enabled. Open /config to configure one.".to_string(),
            ));
        }

        let mut combined = Vec::new();
        for addon in catalog_addons {
            let base_url = Self::base_addon_url(&addon.manifest_url);
            if let Ok(movies) = self
                .fetch_catalog_search(&base_url, "movie", "top", query)
                .await
            {
                combined.extend(movies);
            }
            if let Ok(series) = self
                .fetch_catalog_search(&base_url, "series", "top", query)
                .await
            {
                combined.extend(series);
            }
            if !combined.is_empty() {
                break;
            }
        }

        if combined.is_empty() {
            return Err(ProviderError::NotFound);
        }

        let mut seen = std::collections::HashSet::new();
        Ok(combined
            .into_iter()
            .filter(|m| seen.insert(m.id.clone()))
            .map(|m| adapter::meta_to_catalog_item(&m))
            .collect())
    }

    /// Fetches enriched metadata from installed Stremio addons.
    pub async fn details(&self, id: &str) -> Result<MediaDetails, ProviderError> {
        let addons = crate::config::load_addons();
        let meta_addons: Vec<_> = addons
            .iter()
            .filter(|a| a.enabled && a.provides_meta)
            .collect();

        let types_to_try = ["movie", "series", "tv", "anime", "other"];
        let mut best_detail: Option<models::MetaDetail> = None;

        for addon in &meta_addons {
            let base_url = Self::base_addon_url(&addon.manifest_url);
            for t in types_to_try {
                if let Ok(d) = self.fetch_meta(&base_url, t, id).await {
                    let has_valid_id = d.id == id;
                    let has_title = !d.name.trim().is_empty()
                        || d.title.as_deref().is_some_and(|t| !t.trim().is_empty());
                    if has_valid_id && has_title {
                        if !d.videos.is_empty()
                            || d.r#type.eq_ignore_ascii_case("series")
                            || d.r#type.eq_ignore_ascii_case("tv")
                            || d.r#type.eq_ignore_ascii_case("movie")
                        {
                            return Ok(adapter::meta_detail_to_media_details(&d));
                        }
                        if best_detail.is_none() {
                            best_detail = Some(d);
                        }
                    }
                }
            }
        }

        for addon in &addons {
            if addon.enabled
                && !meta_addons
                    .iter()
                    .any(|m| m.manifest_url == addon.manifest_url)
            {
                let base_url = Self::base_addon_url(&addon.manifest_url);
                for t in types_to_try {
                    if let Ok(d) = self.fetch_meta(&base_url, t, id).await {
                        let has_valid_id = d.id == id;
                        let has_title = !d.name.trim().is_empty()
                            || d.title.as_deref().is_some_and(|t| !t.trim().is_empty());
                        if has_valid_id && has_title {
                            if !d.videos.is_empty()
                                || d.r#type.eq_ignore_ascii_case("series")
                                || d.r#type.eq_ignore_ascii_case("tv")
                                || d.r#type.eq_ignore_ascii_case("movie")
                            {
                                return Ok(adapter::meta_detail_to_media_details(&d));
                            }
                            if best_detail.is_none() {
                                best_detail = Some(d);
                            }
                        }
                    }
                }
            }
        }

        if let Some(d) = best_detail {
            return Ok(adapter::meta_detail_to_media_details(&d));
        }

        Err(ProviderError::NotFound)
    }

    /// Legacy Provider ID.
    pub fn legacy_id(&self) -> ProviderKind {
        ProviderKind::Addons
    }
}
