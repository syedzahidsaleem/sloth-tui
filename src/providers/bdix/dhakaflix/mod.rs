//! DhakaFlix (BDIX) provider implementation.

pub mod client;

pub use client::{DhakaFlixClient, DhakaFlixError};

use crate::providers::models::{
    CatalogItem, EpisodeRef, Media, MediaType, ProviderError, ProviderKind, Quality, Release,
    StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities, ReleaseProvider};
use async_trait::async_trait;

impl From<DhakaFlixError> for ProviderError {
    fn from(err: DhakaFlixError) -> Self {
        match err {
            DhakaFlixError::Network(e) => ProviderError::Network(e.to_string()),
            DhakaFlixError::Parse(p) => ProviderError::Parsing(p),
        }
    }
}

#[async_trait]
impl Provider for client::DhakaFlixClient {
    fn id(&self) -> &'static str {
        "bdix_dhakaflix"
    }

    fn name(&self) -> &'static str {
        "DhakaFlix (BDIX)"
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
            subtitles: false,
            download: true,
            quality_selection: true,
            dub_sub_toggle: false,
            supports_search: true,
            supports_pagination: false,
            supports_series: true,
            supports_subtitles: false,
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
        _episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        let releases = self.streams(&media.id).await.map_err(ProviderError::from)?;
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
                    provider_id: "bdix_dhakaflix",
                });
            }
        }
        Ok(urls)
    }

    async fn health(&self) -> bool {
        self.health().await
    }
}

impl client::DhakaFlixClient {
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
        ProviderKind::BdixDhakaFlix
    }
}

#[async_trait]
impl ReleaseProvider for client::DhakaFlixClient {
    async fn episode_streams(
        &self,
        id: &str,
        _season: usize,
        _episode: usize,
    ) -> Result<Vec<Release>, ProviderError> {
        self.streams(id).await.map_err(ProviderError::from)
    }
}
