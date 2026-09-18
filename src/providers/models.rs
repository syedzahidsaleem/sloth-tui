//! Provider domain models and error definitions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Provider identifier enum representing built-in and addon providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProviderKind {
    /// MovieBox provider.
    #[default]
    #[serde(rename = "moviebox", alias = "movie_box")]
    MovieBox,
    /// 4KHDHub provider.
    #[serde(rename = "fourkhdhub", alias = "four_k_hd_hub", alias = "4khdhub")]
    FourKHdHub,
    /// BDIX CircleFTP provider.
    #[serde(rename = "bdix_circleftp", alias = "bdix_circle_ftp")]
    BdixCircleFtp,
    /// BDIX DhakaFlix provider.
    #[serde(rename = "bdix_dhakaflix", alias = "bdix_dhaka_flix")]
    BdixDhakaFlix,
    /// Stremio Addons provider.
    #[serde(rename = "addons", alias = "addon")]
    Addons,
}

impl ProviderKind {
    /// Active enabled built-in streaming providers.
    pub const ENABLED: [Self; 4] = [
        Self::MovieBox,
        Self::FourKHdHub,
        Self::BdixCircleFtp,
        Self::BdixDhakaFlix,
    ];

    /// Static string cache key for disk storage and database.
    pub const fn cache_key(self) -> &'static str {
        match self {
            Self::MovieBox => "moviebox",
            Self::FourKHdHub => "fourkhdhub",
            Self::BdixCircleFtp => "bdix_circleftp",
            Self::BdixDhakaFlix => "bdix_dhakaflix",
            Self::Addons => "addons",
        }
    }

    /// User-visible display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::MovieBox => "MovieBox",
            Self::FourKHdHub => "4KHDHub",
            Self::BdixCircleFtp => "CircleFTP (BDIX)",
            Self::BdixDhakaFlix => "DhakaFlix (BDIX)",
            Self::Addons => "Addons",
        }
    }

    /// Parses string identifier into a ProviderKind variant.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "moviebox" => Some(Self::MovieBox),
            "4khdhub" | "fourkhdhub" => Some(Self::FourKHdHub),
            "bdix_circleftp" | "circleftp (bdix)" => Some(Self::BdixCircleFtp),
            "bdix_dhakaflix" | "dhakaflix (bdix)" => Some(Self::BdixDhakaFlix),
            "addons" | "addon" => Some(Self::Addons),
            _ => None,
        }
    }

    /// Checks if provider requires a BDIX network connection.
    pub const fn is_bdix(self) -> bool {
        matches!(self, Self::BdixCircleFtp | Self::BdixDhakaFlix)
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Composite media identifier containing provider kind and remote ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderMediaId {
    /// Source provider.
    pub provider: ProviderKind,
    /// Provider-specific item identifier.
    pub value: String,
}

/// Request context holding provider kind and request generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestContext {
    /// Provider being queried.
    pub provider: ProviderKind,
    /// UI generation / cancellation token.
    pub generation: u64,
}

/// Type of media content represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    /// Feature-length film.
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
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
    #[default]
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

    /// Maps vertical resolution in pixels to Quality tier.
    pub fn from_resolution(res: u64) -> Self {
        if res >= 2160 {
            Self::UHD4K
        } else if res >= 1080 {
            Self::FHD1080
        } else if res >= 720 {
            Self::HD720
        } else if res >= 480 {
            Self::SD480
        } else if res >= 360 {
            Self::SD360
        } else {
            Self::Unknown
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
        let provider_id = Box::leak(helper.provider_id.into_boxed_str());

        Ok(StreamUrl {
            url: helper.url,
            quality: helper.quality,
            is_hls: helper.is_hls,
            headers: helper.headers,
            subtitle_url: helper.subtitle_url,
            provider_id,
        })
    }
}

/// Primary representation of a media item in search results, lists, and playback.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Media {
    /// Unique media identifier scoped to the provider.
    pub id: String,
    /// Human-readable title of the media.
    pub title: String,
    /// Type of media content.
    pub media_type: MediaType,
    /// Release year if available.
    pub year: Option<u32>,
    /// Short synopsis or plot description.
    pub overview: Option<String>,
    /// Direct HTTP URL to poster image.
    pub poster_url: Option<String>,
    /// Direct HTTP URL to backdrop image.
    pub backdrop_url: Option<String>,
    /// List of genre classifications.
    pub genres: Vec<String>,
    /// Community rating score (0.0 - 10.0 scale).
    pub rating: Option<f32>,
    /// Total duration in seconds (for movies) or episode duration.
    pub duration_secs: Option<f64>,
    /// Number of seasons (for television/series content).
    pub seasons_count: Option<u32>,
    /// Total number of episodes (for series/anime content).
    pub episodes_count: Option<u32>,
    /// Identifier of the provider that discovered this media item.
    pub provider_id: &'static str,
    /// External catalog identifiers (IMDb, TMDb, etc.).
    pub external_ids: ExternalIds,
    /// Leading cast members.
    pub cast: Vec<CastMember>,
}

impl<'de> Deserialize<'de> for Media {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MediaHelper {
            id: String,
            title: String,
            media_type: MediaType,
            year: Option<u32>,
            overview: Option<String>,
            poster_url: Option<String>,
            backdrop_url: Option<String>,
            genres: Vec<String>,
            rating: Option<f32>,
            duration_secs: Option<f64>,
            seasons_count: Option<u32>,
            episodes_count: Option<u32>,
            provider_id: String,
            external_ids: ExternalIds,
            cast: Vec<CastMember>,
        }

        let helper = MediaHelper::deserialize(deserializer)?;
        let provider_id = Box::leak(helper.provider_id.into_boxed_str());

        Ok(Media {
            id: helper.id,
            title: helper.title,
            media_type: helper.media_type,
            year: helper.year,
            overview: helper.overview,
            poster_url: helper.poster_url,
            backdrop_url: helper.backdrop_url,
            genres: helper.genres,
            rating: helper.rating,
            duration_secs: helper.duration_secs,
            seasons_count: helper.seasons_count,
            episodes_count: helper.episodes_count,
            provider_id,
            external_ids: helper.external_ids,
            cast: helper.cast,
        })
    }
}

/// Cross-platform external database identifiers for metadata matching.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIds {
    /// IMDb identifier (e.g. "tt0111161").
    pub imdb: Option<String>,
    /// TheMovieDatabase (TMDb) numeric ID.
    pub tmdb: Option<u32>,
    /// MyAnimeList (MAL) ID for anime content.
    pub mal: Option<u32>,
    /// AniList ID for anime content.
    pub anilist: Option<u32>,
    /// TheTVDB numeric ID.
    pub tvdb: Option<u32>,
}

/// Credited actor or creator in a media production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastMember {
    /// Real name of the actor or creator.
    pub name: String,
    /// Name of the role or character portrayed.
    pub character: Option<String>,
}

/// Current airing lifecycle state for anime and television series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiringStatus {
    /// Currently broadcasting new episodes.
    Airing,
    /// Concluded broadcast run.
    Finished,
    /// Scheduled to premiere at a future date.
    Upcoming,
    /// Broadcast status unknown.
    Unknown,
}

/// Broadcast schedule information for currently airing series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiringSchedule {
    /// Day of the week when new episodes air (e.g. "Sunday").
    pub day_of_week: Option<String>,
    /// Time of broadcast in UTC format (e.g. "15:30").
    pub time_utc: Option<String>,
    /// Episode number of the upcoming broadcast.
    pub next_episode_number: Option<u32>,
    /// Approximate air date timestamp or ISO-8601 string.
    pub next_episode_air_date: Option<String>,
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

/// Item listed in a provider catalog or search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogItem {
    /// Media provider and ID.
    pub id: ProviderMediaId,
    /// Title of the content.
    pub title: String,
    /// Media type.
    pub media_type: MediaType,
    /// Release year string.
    pub year: Option<String>,
    /// Poster artwork URL.
    pub poster_url: Option<String>,
    /// Number of seasons if known.
    pub season_count: Option<usize>,
}

/// Episode representation with season, episode number, and title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    /// Season index.
    pub season: usize,
    /// Episode number.
    pub number: usize,
    /// Episode title.
    pub title: Option<String>,
    /// Episode synopsis overview.
    pub overview: Option<String>,
}

/// Season representation containing a list of episodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Season {
    /// Season number.
    pub number: usize,
    /// Episodes within this season.
    pub episodes: Vec<Episode>,
}

/// Audio track / dubbed language option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioTrackOption {
    /// Provider subject ID for this audio track.
    pub subject_id: String,
    /// Language code or name.
    pub language: String,
    /// Human display label.
    pub label: String,
}

/// Detailed media metadata returned by provider inspection endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaDetails {
    /// Media ID.
    pub id: ProviderMediaId,
    /// Title of the movie or series.
    pub title: String,
    /// Media type.
    pub media_type: MediaType,
    /// Release year.
    pub year: Option<String>,
    /// Detailed overview description.
    pub description: Option<String>,
    /// Catchphrase or tagline.
    pub tagline: Option<String>,
    /// IMDb rating string.
    pub imdb_rating: Option<String>,
    /// Director name.
    pub director: Option<String>,
    /// Starring actors.
    pub stars: Option<String>,
    /// Print formats available.
    pub prints: Option<String>,
    /// Available audio languages.
    pub audios: Option<String>,
    /// Poster artwork URL.
    pub poster_url: Option<String>,
    /// Runtime duration string.
    pub duration: Option<String>,
    /// Associated genres.
    pub genres: Vec<String>,
    /// Seasons and episodes.
    pub seasons: Vec<Season>,
    /// Dubbed audio options.
    pub dubs: Vec<AudioTrackOption>,
}

impl MediaDetails {
    /// Checks whether this media item is a multi-episode series.
    pub fn is_series(&self) -> bool {
        self.media_type == MediaType::Series || !self.seasons.is_empty()
    }

    /// Checks whether multiple audio tracks / dubs are available.
    pub fn has_languages(&self) -> bool {
        self.dubs.len() > 1
    }

    /// Returns the poster or cover URL.
    pub fn cover_url(&self) -> Option<&str> {
        self.poster_url.as_deref()
    }

    /// Creates a `MediaDetails` from a search result and optional cached details.
    pub fn from_search_result(
        item: &crate::models::SearchResult,
        preview: Option<&MediaDetails>,
    ) -> Self {
        if let Some(p) = preview.filter(|p| p.id.value == item.id && p.id.provider == item.provider)
        {
            let mut details = p.clone();
            if details.title.trim().is_empty() {
                details.title = item.title.clone();
            }
            if details.year.is_none() && !item.release_year.trim().is_empty() {
                details.year = Some(item.release_year.clone());
            }
            if details.poster_url.is_none() {
                details.poster_url = item.cover_url.clone();
            }
            details
        } else {
            MediaDetails {
                id: ProviderMediaId {
                    provider: item.provider,
                    value: item.id.clone(),
                },
                title: item.title.clone(),
                media_type: if item.stype == 2 {
                    MediaType::Series
                } else {
                    MediaType::Movie
                },
                year: if !item.release_year.trim().is_empty() {
                    Some(item.release_year.clone())
                } else {
                    None
                },
                description: None,
                tagline: None,
                imdb_rating: None,
                director: None,
                stars: None,
                prints: None,
                audios: None,
                poster_url: item.cover_url.clone(),
                duration: None,
                genres: vec![],
                seasons: vec![],
                dubs: vec![],
            }
        }
    }
}

/// Download or stream mirror source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMirror {
    /// Mirror label.
    pub label: String,
    /// URL to resolver or media file.
    pub resolver_url: String,
    /// HTTP headers needed to fetch or stream.
    pub headers: Vec<(String, String)>,
    /// Whether this is a direct media file URL.
    pub direct_file: bool,
}

/// Subtitle track option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtitleOption {
    /// Track language or label.
    pub name: String,
    /// Subtitle URL.
    pub url: String,
}

/// Release source file or stream candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    /// Originating provider.
    pub provider: ProviderKind,
    /// Release filename.
    pub filename: String,
    /// Quality or resolution label (e.g. "1080p", "4K").
    pub quality: Option<String>,
    /// Video codec (e.g. "x264", "HEVC").
    pub codec: Option<String>,
    /// Audio language.
    pub language: Option<String>,
    /// File size in bytes.
    pub size_bytes: Option<u64>,
    /// Season number for series release.
    pub season: Option<usize>,
    /// Episode number for series release.
    pub episode: Option<usize>,
    /// Source mirrors.
    pub mirrors: Vec<SourceMirror>,
    /// Optional resource identifier.
    #[serde(default)]
    pub resource_id: Option<String>,
}

impl Release {
    /// Checks if this release provides multiple selectable resolutions.
    pub fn is_multi_resolution(&self) -> bool {
        self.quality
            .as_deref()
            .is_some_and(|q| q.eq_ignore_ascii_case("multi") || q.eq_ignore_ascii_case("multi-res"))
    }

    /// Returns resolution as a pixel height (e.g. 2160, 1080, 720).
    pub fn resolution_u64(&self) -> u64 {
        self.quality
            .as_deref()
            .and_then(|q| {
                let trimmed = q.trim();
                if trimmed.eq_ignore_ascii_case("4k") || trimmed.eq_ignore_ascii_case("uhd") {
                    return Some(2160);
                }
                trimmed.trim_end_matches(['p', 'P']).parse::<u64>().ok()
            })
            .unwrap_or(1080)
    }

    /// Returns signed resolution or -1 for multi-resolution releases.
    pub fn resolution_i64(&self) -> i64 {
        if self.is_multi_resolution() {
            -1
        } else {
            self.resolution_u64() as i64
        }
    }

    /// Returns human-readable label for the release source.
    pub fn source_label(&self) -> &str {
        self.mirrors
            .first()
            .map(|m| m.label.as_str())
            .unwrap_or_else(|| match self.provider {
                ProviderKind::FourKHdHub => "4KHDHub",
                ProviderKind::BdixCircleFtp => "CircleFTP",
                ProviderKind::BdixDhakaFlix => "DhakaFlix",
                ProviderKind::Addons => "Addon",
                ProviderKind::MovieBox => "Direct",
            })
    }

    /// Returns the direct stream or resolver URL of the primary mirror.
    pub fn direct_url(&self) -> Option<&str> {
        self.mirrors.first().map(|m| m.resolver_url.as_str())
    }
}

/// Resolved playable media source with headers and subtitles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackSource {
    /// Originating provider.
    pub provider: ProviderKind,
    /// Playable stream or file URL.
    pub url: String,
    /// HTTP headers needed by the video player.
    pub headers: Vec<(String, String)>,
    /// Optional subtitle track URL.
    pub subtitle: Option<String>,
    /// Label for UI display.
    pub source_label: String,
}

impl PlaybackSource {
    /// Creates a minimal `PlaybackSource` without extra headers.
    pub fn bare(provider: ProviderKind, url: impl Into<String>, subtitle: Option<String>) -> Self {
        Self {
            provider,
            url: url.into(),
            headers: Vec::new(),
            subtitle,
            source_label: provider.label().to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Adapters / Conversions to Sloth types
// ---------------------------------------------------------------------------

impl From<CatalogItem> for Media {
    fn from(item: CatalogItem) -> Self {
        let year = item.year.as_deref().and_then(|y| y.parse::<u32>().ok());
        Self {
            id: item.id.value,
            title: item.title,
            media_type: item.media_type,
            year,
            overview: None,
            poster_url: item.poster_url,
            backdrop_url: None,
            genres: Vec::new(),
            rating: None,
            duration_secs: None,
            seasons_count: item.season_count.map(|s| s as u32),
            episodes_count: None,
            provider_id: item.id.provider.cache_key(),
            external_ids: ExternalIds::default(),
            cast: Vec::new(),
        }
    }
}

impl From<MediaDetails> for Media {
    fn from(details: MediaDetails) -> Self {
        let year = details.year.as_deref().and_then(|y| y.parse::<u32>().ok());
        let rating = details
            .imdb_rating
            .as_deref()
            .and_then(|r| r.parse::<f32>().ok());
        let seasons_count = if details.seasons.is_empty() {
            None
        } else {
            Some(details.seasons.len() as u32)
        };
        let episodes_count = if details.seasons.is_empty() {
            None
        } else {
            Some(details.seasons.iter().map(|s| s.episodes.len() as u32).sum())
        };
        Self {
            id: details.id.value,
            title: details.title,
            media_type: details.media_type,
            year,
            overview: details.description,
            poster_url: details.poster_url,
            backdrop_url: None,
            genres: details.genres,
            rating,
            duration_secs: None,
            seasons_count,
            episodes_count,
            provider_id: details.id.provider.cache_key(),
            external_ids: ExternalIds::default(),
            cast: Vec::new(),
        }
    }
}

impl From<Episode> for EpisodeRef {
    fn from(ep: Episode) -> Self {
        Self {
            season: ep.season as u32,
            episode: ep.number as u32,
            title: ep.title,
            duration_secs: None,
        }
    }
}

impl From<PlaybackSource> for StreamUrl {
    fn from(src: PlaybackSource) -> Self {
        let is_hls = src.url.contains(".m3u8");
        Self {
            url: src.url,
            quality: Quality::Auto,
            is_hls,
            headers: src.headers,
            subtitle_url: src.subtitle,
            provider_id: src.provider.cache_key(),
        }
    }
}

// ---------------------------------------------------------------------------
// Error Definitions
// ---------------------------------------------------------------------------

/// Errors that can occur during provider querying and stream resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderError {
    /// Network connection failure.
    #[error("Network connection failed: {0}")]
    Network(String),
    /// Rate limited by remote provider.
    #[error("Rate limited by provider")]
    RateLimited(Option<u64>),
    /// Item not found on provider.
    #[error("Item not found on provider")]
    NotFound,
    /// Failed to parse provider response.
    #[error("Failed to parse response: {0}")]
    Parsing(String),
    /// Provider is temporarily unavailable.
    #[error("Provider is temporarily unavailable: {0}")]
    Unavailable(String),
    /// Request timed out.
    #[error("Request timed out")]
    Timeout,
    /// Requested quality is not supported.
    #[error("Unsupported quality: {0}")]
    UnsupportedQuality(String),
    /// Internal provider error.
    #[error("Internal provider error: {0}")]
    Internal(String),
    /// Provider requires authentication.
    #[error("Authentication required")]
    AuthRequired,
}

impl ProviderError {
    /// User-friendly message customized by provider kind.
    pub fn user_message(&self, provider: ProviderKind) -> String {
        let label = match provider {
            ProviderKind::BdixCircleFtp => "CircleFTP",
            ProviderKind::BdixDhakaFlix => "DhakaFlix",
            _ => provider.label(),
        };

        match self {
            Self::Network(msg) => {
                if provider.is_bdix() {
                    format!("{label} unreachable: requires BDIX network.")
                } else {
                    let lower = msg.to_ascii_lowercase();
                    if lower.contains("timed out") || lower.contains("timeout") {
                        format!("{label} timed out.")
                    } else {
                        format!("Cannot reach {label}.")
                    }
                }
            }
            Self::Timeout => format!("{label} timed out."),
            Self::RateLimited(secs) => match secs {
                Some(s) => format!("Rate limited. Wait {s}s."),
                None => "Rate limited. Try later.".to_string(),
            },
            Self::NotFound => "No results found.".to_string(),
            Self::Parsing(_) => format!("{label} parse error."),
            Self::Unavailable(msg) => {
                let trimmed = msg.trim();
                if let Some(status) = trimmed.strip_prefix("HTTP status ") {
                    format!("{label} error ({status}).")
                } else if trimmed.is_empty() {
                    format!("{label} unavailable.")
                } else {
                    format!("{label} unavailable: {trimmed}")
                }
            }
            Self::UnsupportedQuality(q) => format!("Unsupported quality: {q}"),
            Self::Internal(msg) => format!("{label} error: {msg}"),
            Self::AuthRequired => "Authentication required.".to_string(),
        }
    }

    /// User-friendly status bar message without provider context.
    pub fn status_message(&self) -> &'static str {
        match self {
            Self::Network(_) => "Network error. Check your connection.",
            Self::Timeout => "Request timed out.",
            Self::RateLimited(_) => "Too many requests. Trying next source...",
            Self::NotFound => "Not found on this provider.",
            Self::Parsing(_) => "Provider response changed. Trying fallback...",
            Self::Unavailable(_) => "Provider is down. Trying next source...",
            Self::UnsupportedQuality(_) => "Quality not available.",
            Self::Internal(_) => "Provider internal error.",
            Self::AuthRequired => "Login required.",
        }
    }

    /// Determines whether the error is transient and should trigger a retry or fallback.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout | Self::RateLimited(_) | Self::Unavailable(_)
        )
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if let Some(status) = err.status() {
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                Self::RateLimited(None)
            } else if status == reqwest::StatusCode::NOT_FOUND {
                Self::NotFound
            } else {
                Self::Unavailable(format!("HTTP status {status}"))
            }
        } else {
            Self::Network(err.to_string())
        }
    }
}

impl From<String> for ProviderError {
    fn from(msg: String) -> Self {
        Self::Unavailable(msg)
    }
}

impl From<&str> for ProviderError {
    fn from(msg: &str) -> Self {
        Self::Unavailable(msg.to_string())
    }
}

/// Strips emojis and decorative symbols from text.
pub fn strip_emojis(input: &str) -> String {
    input
        .chars()
        .filter(|&c| {
            let u = c as u32;
            !((0x1F000..=0x1FAFF).contains(&u)
                || (0x2600..=0x27BF).contains(&u)
                || (0x2300..=0x23FF).contains(&u)
                || (0x2B00..=0x2BFF).contains(&u)
                || (0xFE00..=0xFE0F).contains(&u)
                || u == 0x200D)
        })
        .collect::<String>()
}

/// Normalizes whitespace and strips emojis from stream and release descriptions.
pub fn clean_stream_text(input: &str) -> String {
    let without_emojis = strip_emojis(input);
    let mut cleaned = String::new();
    let mut last_was_space = false;
    for c in without_emojis.chars() {
        if c.is_whitespace() {
            if !last_was_space && !cleaned.is_empty() {
                cleaned.push(' ');
                last_was_space = true;
            }
        } else {
            cleaned.push(c);
            last_was_space = false;
        }
    }
    cleaned.trim().to_string()
}

/// Extracts a 4-digit release year (starting with 1 or 2) from arbitrary text.
pub fn extract_4digit_year(raw: &str) -> String {
    raw.as_bytes()
        .windows(4)
        .find(|window| window.iter().all(u8::is_ascii_digit) && matches!(window[0], b'1' | b'2'))
        .and_then(|window| std::str::from_utf8(window).ok())
        .map(str::to_string)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_resolution_parsing() {
        let make_release = |q: Option<&str>| Release {
            provider: ProviderKind::BdixCircleFtp,
            filename: "Test.mkv".to_string(),
            quality: q.map(|s| s.to_string()),
            codec: None,
            language: None,
            size_bytes: None,
            season: None,
            episode: None,
            mirrors: Vec::new(),
            resource_id: None,
        };

        assert_eq!(make_release(Some("4K")).resolution_u64(), 2160);
        assert_eq!(make_release(Some("4k")).resolution_u64(), 2160);
        assert_eq!(make_release(Some("2160p")).resolution_u64(), 2160);
        assert_eq!(make_release(Some("1080p")).resolution_u64(), 1080);
        assert_eq!(make_release(None).resolution_u64(), 1080);
    }

    #[test]
    fn test_provider_error_user_message_concise() {
        let bdix_net_err = ProviderError::Network("tcp connect error: operation timed out".into());
        assert_eq!(
            bdix_net_err.user_message(ProviderKind::BdixCircleFtp),
            "CircleFTP unreachable: requires BDIX network."
        );
        assert_eq!(
            bdix_net_err.user_message(ProviderKind::BdixDhakaFlix),
            "DhakaFlix unreachable: requires BDIX network."
        );

        let timeout_err = ProviderError::Network("operation timed out".into());
        assert_eq!(
            timeout_err.user_message(ProviderKind::MovieBox),
            "MovieBox timed out."
        );

        let connect_err = ProviderError::Network("dns lookup failed".into());
        assert_eq!(
            connect_err.user_message(ProviderKind::FourKHdHub),
            "Cannot reach 4KHDHub."
        );

        let rate_limit = ProviderError::RateLimited(Some(30));
        assert_eq!(
            rate_limit.user_message(ProviderKind::MovieBox),
            "Rate limited. Wait 30s."
        );

        let not_found = ProviderError::NotFound;
        assert_eq!(
            not_found.user_message(ProviderKind::MovieBox),
            "No results found."
        );

        let http_status = ProviderError::Unavailable("HTTP status 502".into());
        assert_eq!(
            http_status.user_message(ProviderKind::FourKHdHub),
            "4KHDHub error (502)."
        );
    }
}
