//! CircleFTP (BDIX) provider implementation.

pub mod client;
pub mod parser;

pub use client::{CircleFtpClient, CircleFtpError};

use crate::providers::models::{
    CatalogItem, EpisodeRef, Media, MediaType, ProviderError, ProviderKind, Quality, Release,
    StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities, ReleaseProvider};
use async_trait::async_trait;

impl From<CircleFtpError> for ProviderError {
    fn from(err: CircleFtpError) -> Self {
        match err {
            CircleFtpError::Network(e) => ProviderError::Network(e.to_string()),
            CircleFtpError::Parse(p) => ProviderError::Parsing(p),
        }
    }
}

#[async_trait]
impl Provider for client::CircleFtpClient {
    fn id(&self) -> &'static str {
        "bdix_circleftp"
    }

    fn name(&self) -> &'static str {
        "CircleFTP (BDIX)"
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
                    provider_id: "bdix_circleftp",
                });
            }
        }
        Ok(urls)
    }

    async fn health(&self) -> bool {
        self.health().await
    }
}

impl client::CircleFtpClient {
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
        ProviderKind::BdixCircleFtp
    }
}

#[async_trait]
impl ReleaseProvider for client::CircleFtpClient {
    async fn episode_streams(
        &self,
        id: &str,
        season: usize,
        episode: usize,
    ) -> Result<Vec<Release>, ProviderError> {
        self.releases(id, Some(season), Some(episode))
            .await
            .map_err(ProviderError::from)
    }
}
