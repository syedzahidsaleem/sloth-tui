//! IPTV and M3U playlist provider module.

pub mod models;
pub mod parser;

pub use models::Channel;
pub use parser::M3UParser;

use async_trait::async_trait;
use crate::providers::models::{
    EpisodeRef, Media, MediaType, ProviderError, Quality, StreamUrl,
};
use crate::providers::{Provider, ProviderCapabilities};

#[async_trait]
impl Provider for M3UParser {
    fn id(&self) -> &'static str {
        "iptv"
    }

    fn name(&self) -> &'static str {
        "Live TV"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            movies: false,
            tv_series: false,
            anime: false,
            live_sports: false,
            iptv: true,
            f1: false,
            subtitles: false,
            download: false,
            quality_selection: false,
            dub_sub_toggle: false,
            supports_search: true,
            supports_pagination: false,
            supports_series: false,
            supports_subtitles: false,
            supports_homepage: false,
        }
    }

    async fn search(&self, _query: &str, _kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        Ok(Vec::new())
    }

    async fn resolve(
        &self,
        media: &Media,
        _episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        Ok(vec![StreamUrl {
            url: media.id.clone(),
            quality: Quality::Auto,
            is_hls: media.id.contains(".m3u8"),
            headers: Vec::new(),
            subtitle_url: None,
            provider_id: "iptv",
        }])
    }

    async fn health(&self) -> bool {
        true
    }
}
