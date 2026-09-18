//! Provider domain models and error definitions.

use serde::{Deserialize, Serialize};

/// Type of media content represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    /// Feature-length film.
    Movie,
    /// Multi-episode television series.
    Series,
    /// Japanese animation series or movie.
    Anime,
    /// Live sports event or match.
    LiveSport,
    /// Formula 1 session, race, or stream.
    F1,
    /// Live IPTV broadcast channel.
    IptvChannel,
}

/// Available stream resolution and quality tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Quality {
    /// 4K Ultra High Definition (2160p).
    UHD4K,
    /// 1080p Full High Definition.
    FHD1080,
    /// 720p High Definition.
    HD720,
    /// 480p Standard Definition.
    SD480,
    /// 360p Low Definition.
    SD360,
    /// Automatic / adaptive bitrate selection.
    Auto,
    /// Unknown or unspecified quality.
    Unknown,
}

impl Quality {
    /// Short human-readable display label for quality tier.
    pub fn label(&self) -> &'static str {
        match self {
            Self::UHD4K => "4K",
            Self::FHD1080 => "1080p",
            Self::HD720 => "720p",
            Self::SD480 => "480p",
            Self::SD360 => "360p",
            Self::Auto => "Auto",
            Self::Unknown => "?",
        }
    }

    /// Numerical rank for sorting quality tiers (higher value = higher quality).
    pub fn quality_rank(&self) -> u8 {
        match self {
            Self::UHD4K => 6,
            Self::FHD1080 => 5,
            Self::HD720 => 4,
            Self::SD480 => 3,
            Self::SD360 => 2,
            Self::Auto => 1,
            Self::Unknown => 0,
        }
    }
}

/// Reference to a specific episode within a series or anime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeRef {
    /// Season number (1-based).
    pub season: u32,
    /// Episode number within the season (1-based).
    pub episode: u32,
    /// Optional episode title.
    pub title: Option<String>,
    /// Optional runtime duration in seconds.
    pub duration_secs: Option<f64>,
}

/// A resolved playable stream URL with playback metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StreamUrl {
    /// Direct media URL (HLS playlist or direct file).
    pub url: String,
    /// Stream resolution tier.
    pub quality: Quality,
    /// Whether this stream is an HLS (m3u8) manifest.
    pub is_hls: bool,
    /// Custom HTTP headers required for streaming (e.g. Referer, User-Agent).
    pub headers: Vec<(String, String)>,
    /// Optional URL for external subtitle tracks (VTT/SRT).
    pub subtitle_url: Option<String>,
    /// Unique identifier of the provider that resolved this stream.
    pub provider_id: &'static str,
}

impl<'de> Deserialize<'de> for StreamUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StreamUrlHelper {
            url: String,
            quality: Quality,
            is_hls: bool,
            headers: Vec<(String, String)>,
            subtitle_url: Option<String>,
            provider_id: String,
        }

        let helper = StreamUrlHelper::deserialize(deserializer)?;
        Ok(StreamUrl {
            url: helper.url,
            quality: helper.quality,
            is_hls: helper.is_hls,
            headers: helper.headers,
            subtitle_url: helper.subtitle_url,
            provider_id: Box::leak(helper.provider_id.into_boxed_str()),
        })
    }
}

impl StreamUrl {
    /// Convenience helper returning the numeric rank of this stream's quality.
    pub fn quality_rank(&self) -> u8 {
        self.quality.quality_rank()
    }
}

/// External database identifiers for media synchronization and metadata linking.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIds {
    /// The Movie Database ID.
    pub tmdb: Option<u32>,
    /// AniList media ID.
    pub anilist: Option<u32>,
    /// MyAnimeList media ID.
    pub mal: Option<u32>,
    /// Trakt.tv media slug or ID.
    pub trakt: Option<String>,
    /// IMDb ID (e.g. "tt0111161").
    pub imdb: Option<String>,
}

/// Core media representation discovered through providers or search.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Media {
    /// Unique provider-scoped identifier (e.g. "moviebox:12345").
    pub id: String,
    /// Identifier of the originating provider.
    pub provider_id: &'static str,
    /// Content category of this item.
    pub kind: MediaType,
    /// Primary title or display name.
    pub title: String,
    /// Release year if known.
    pub year: Option<u32>,
    /// URL pointing to cover or poster image.
    pub poster_url: Option<String>,
    /// Average user rating (0.0 to 10.0 scale).
    pub rating: Option<f32>,
    /// Synopsis or plot summary.
    pub description: Option<String>,
    /// Associated genre tags.
    pub genres: Vec<String>,
    /// Total number of episodes if applicable.
    pub total_episodes: Option<u32>,
    /// Total number of seasons if applicable.
    pub total_seasons: Option<u32>,
    /// External service identifiers.
    pub external_ids: ExternalIds,
}

impl<'de> Deserialize<'de> for Media {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MediaHelper {
            id: String,
            provider_id: String,
            kind: MediaType,
            title: String,
            year: Option<u32>,
            poster_url: Option<String>,
            rating: Option<f32>,
            description: Option<String>,
            genres: Vec<String>,
            total_episodes: Option<u32>,
            total_seasons: Option<u32>,
            external_ids: ExternalIds,
        }

        let helper = MediaHelper::deserialize(deserializer)?;
        Ok(Media {
            id: helper.id,
            provider_id: Box::leak(helper.provider_id.into_boxed_str()),
            kind: helper.kind,
            title: helper.title,
            year: helper.year,
            poster_url: helper.poster_url,
            rating: helper.rating,
            description: helper.description,
            genres: helper.genres,
            total_episodes: helper.total_episodes,
            total_seasons: helper.total_seasons,
            external_ids: helper.external_ids,
        })
    }
}

/// Cast member or actor credits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastMember {
    /// Actor or performer name.
    pub name: String,
    /// Character name portrayed.
    pub character: Option<String>,
    /// Profile picture URL.
    pub photo_url: Option<String>,
}

/// Broadcast or airing status for serialized content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiringStatus {
    /// Currently releasing new episodes.
    Airing,
    /// Completely finished broadcasting.
    Finished,
    /// Scheduled to air in the future.
    NotYetAired,
    /// Production discontinued prematurely.
    Cancelled,
    /// On temporary hiatus.
    Hiatus,
}

/// Anime or TV series broadcast scheduling details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiringSchedule {
    /// Current broadcast status.
    pub status: AiringStatus,
    /// Timestamp for next upcoming episode release.
    pub next_episode_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Season descriptor (e.g. "Fall 2026").
    pub season: Option<String>,
}

/// Comprehensive media details from metadata sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// High-resolution poster URL.
    pub poster_url: Option<String>,
    /// Wide backdrop or banner image URL.
    pub backdrop_url: Option<String>,
    /// Score rating out of 10.0.
    pub rating: Option<f32>,
    /// Number of votes recorded.
    pub vote_count: Option<u32>,
    /// Full storyline or overview.
    pub description: Option<String>,
    /// Categorized genres.
    pub genres: Vec<String>,
    /// Main cast and crew credits.
    pub cast: Vec<CastMember>,
    /// Video trailer link (YouTube/direct MP4).
    pub trailer_url: Option<String>,
    /// Related or recommended media items.
    pub recommendations: Vec<Media>,
    /// Detailed episode listings when available.
    pub episodes: Option<Vec<EpisodeRef>>,
    /// Broadcast schedule if currently tracking airing status.
    pub airing_schedule: Option<AiringSchedule>,
}

/// Errors that can occur during provider querying and stream resolution.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Underlying HTTP request failed.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The remote provider responded with HTTP 429 Too Many Requests.
    #[error("provider is rate limiting us (HTTP 429)")]
    RateLimited,

    /// The requested media or episode was not found on this provider.
    #[error("content not found on this provider")]
    NotFound,

    /// Provider response could not be parsed into expected data structures.
    #[error("provider response parsing failed: {0}")]
    Parsing(String),

    /// The provider service is unreachable or temporarily down.
    #[error("provider is temporarily unavailable")]
    Unavailable,

    /// Provider requires account authentication.
    #[error("auth required — configure in Settings")]
    AuthRequired,
}

impl ProviderError {
    /// User-friendly error message suitable for displaying in the TUI status bar.
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Network(_) => "Network error. Check your connection.",
            Self::RateLimited => "Too many requests. Trying next source...",
            Self::NotFound => "Not found on this provider.",
            Self::Parsing(_) => "Provider response changed. Trying fallback...",
            Self::Unavailable => "Provider is down. Trying next source...",
            Self::AuthRequired => "Login required. Check Settings > Accounts.",
        }
    }

    /// Determines whether the error is transient and should trigger a retry or fallback.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimited | Self::Unavailable
        )
    }
}
