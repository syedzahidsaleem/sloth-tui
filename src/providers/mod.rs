//! Providers module for media scrapers and stream resolvers.

use async_trait::async_trait;

pub mod anime;
pub mod f1;
pub mod models;
pub mod registry;
pub mod sports;

pub use models::*;
pub use registry::ProviderRegistry;

/// Declares supported features and content domains of a provider.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProviderCapabilities {
    /// Supports keyword searching.
    pub search: bool,
    /// Can provide or resolve feature films.
    pub movies: bool,
    /// Can provide or resolve TV series.
    pub tv_series: bool,
    /// Can provide or resolve anime series/movies.
    pub anime: bool,
    /// Can provide or resolve live sports streams.
    pub live_sports: bool,
    /// Can provide or resolve IPTV live channels.
    pub iptv: bool,
    /// Can provide or resolve Formula 1 streams or calendar.
    pub f1: bool,
    /// Can fetch external or embedded subtitles.
    pub subtitles: bool,
    /// Supports direct downloading of media streams.
    pub download: bool,
    /// Allows user selection between multiple video qualities.
    pub quality_selection: bool,
    /// Supports toggling between dubbed and subbed audio/text.
    pub dub_sub_toggle: bool,
}

/// Core interface implemented by all media providers.
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Unique machine identifier for this provider (e.g. "moviebox", "hianime").
    fn id(&self) -> &'static str;

    /// Human-friendly display name for the user interface.
    fn name(&self) -> &'static str;

    /// Declared feature capabilities.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Searches for media items matching the given query string and content type.
    async fn search(&self, query: &str, kind: MediaType) -> Result<Vec<Media>, ProviderError>;

    /// Resolves stream URLs for playback of a given media item and optional episode.
    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError>;

    /// Fetches the episode listing for a specific season of a series or anime.
    async fn episodes(&self, media: &Media, season: u32) -> Result<Vec<EpisodeRef>, ProviderError> {
        let _ = (media, season);
        Err(ProviderError::NotFound)
    }

    /// Fetches enriched metadata for a media item (implemented by metadata providers).
    async fn metadata(&self, media: &Media) -> Option<MediaMetadata> {
        let _ = media;
        None
    }

    /// Healthcheck returning whether the provider is currently reachable and operational.
    async fn health(&self) -> bool {
        true
    }
}
