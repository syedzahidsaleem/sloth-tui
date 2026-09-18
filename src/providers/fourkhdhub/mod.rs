//! 4KHDHub scraper and stream resolver module.

pub mod client;
pub mod hubcloud;
pub mod parser;

pub use client::{FourKHdHubClient, FourKHdHubError};

use async_trait::async_trait;
use crate::providers::models::{
    CatalogItem, EpisodeRef, Media, MediaType, ProviderError, ProviderKind, Quality, Release,
    StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities, ReleaseProvider};

impl From<FourKHdHubError> for ProviderError {
    fn from(err: FourKHdHubError) -> Self {
        match err {
            FourKHdHubError::Network(e) => ProviderError::Network(e.to_string()),
            FourKHdHubError::InvalidUrl(u) => ProviderError::Parsing(format!("Invalid URL: {u}")),
            FourKHdHubError::Parse(p) => ProviderError::Parsing(p),
            FourKHdHubError::NoPlayableMirror(msg) => ProviderError::Unavailable(msg),
        }
    }
}

#[async_trait]
impl Provider for FourKHdHubClient {
    fn id(&self) -> &'static str {
        "fourkhdhub"
    }

    fn name(&self) -> &'static str {
        "4KHDHub"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            movies: true,
            tv_series: true,
            anime: false,
            live_sports: false,
            iptv: false,
            f1: false,
            subtitles: true,
            download: true,
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
        let catalog = self.search(query).await.map_err(ProviderError::from)?;
        Ok(catalog.into_iter().map(Media::from).collect())
    }

    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let (season, ep_num) = match episode {
            Some(e) => (e.season as usize, e.episode as usize),
            None => (0, 0),
        };
        let releases = self.episode_streams(&media.id, season, ep_num).await?;
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
                    provider_id: "fourkhdhub",
                });
            }
        }
        Ok(urls)
    }

    async fn health(&self) -> bool {
        self.health_check().await.is_ok()
    }
}

impl FourKHdHubClient {
    /// Legacy Provider search returning CatalogItem vector.
    pub async fn search_catalog(
        &self,
        query: &str,
        _page: usize,
    ) -> Result<Vec<CatalogItem>, ProviderError> {
        self.search(query).await.map_err(ProviderError::from)
    }

    /// Legacy Provider ID.
    pub fn legacy_id(&self) -> ProviderKind {
        ProviderKind::FourKHdHub
    }
}

#[async_trait]
impl ReleaseProvider for FourKHdHubClient {
    async fn episode_streams(
        &self,
        id: &str,
        season: usize,
        episode: usize,
    ) -> Result<Vec<Release>, ProviderError> {
        self.releases(id, season, episode)
            .await
            .map_err(ProviderError::from)
    }
}
