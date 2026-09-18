//! Core application models, search results, and notification types.

pub use crate::providers::models::{
    AudioTrackOption, CatalogItem, Episode, MediaDetails, MediaType, PlaybackSource, ProviderError,
    ProviderKind, ProviderMediaId, Release, Season, SourceMirror, SubtitleOption,
};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Identity representation for matching media across different providers.
#[derive(Debug, Clone, Copy)]
pub struct SubjectIdentity<'a> {
    /// Name of provider.
    pub provider: &'a str,
    /// Identifier within provider.
    pub subject_id: &'a str,
    /// Title of the content.
    pub title: &'a str,
    /// Subject type: 1 for movie, 2 for series.
    pub stype: i64,
    /// Release year string.
    pub release_year: &'a str,
}

impl<'a> SubjectIdentity<'a> {
    /// Compares two identities across or within providers.
    pub fn matches(&self, other: &SubjectIdentity<'_>) -> bool {
        if self.stype != other.stype {
            return false;
        }

        let prov_a = crate::providers::models::ProviderKind::parse(self.provider);
        let prov_b = crate::providers::models::ProviderKind::parse(other.provider);
        let same_provider = match (prov_a, prov_b) {
            (Some(pa), Some(pb)) => pa == pb,
            _ => self
                .provider
                .trim()
                .eq_ignore_ascii_case(other.provider.trim()),
        };

        if !same_provider {
            return false;
        }

        if !self.subject_id.is_empty() && !other.subject_id.is_empty() {
            return self.subject_id == other.subject_id;
        }

        let clean_a = crate::providers::moviebox::clean_moviebox_title(self.title);
        let clean_b = crate::providers::moviebox::clean_moviebox_title(other.title);
        if !clean_a.is_empty() && clean_a.eq_ignore_ascii_case(clean_b) {
            let year_a = self.release_year.trim();
            let year_b = other.release_year.trim();
            if !year_a.is_empty() && !year_b.is_empty() {
                return year_a == year_b;
            }
            return true;
        }
        false
    }
}

/// Search result item returned from provider search queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Provider item ID.
    pub id: String,
    /// Content title.
    pub title: String,
    /// Subject type: 1 for movie, 2 for series.
    pub stype: i64,
    /// Release year string.
    pub release_year: String,
    /// Poster artwork URL.
    pub cover_url: Option<String>,
    /// Selected season index.
    pub season: usize,
    /// Selected episode index.
    pub episode: usize,
    /// Providing source kind.
    pub provider: ProviderKind,
}

impl SearchResult {
    /// Converts a catalog item into a search result.
    pub fn from_catalog_item(item: CatalogItem) -> Self {
        Self {
            id: item.id.value,
            title: item.title,
            stype: if item.media_type == MediaType::Series {
                2
            } else {
                1
            },
            release_year: item.year.unwrap_or_default(),
            cover_url: item.poster_url,
            season: 0,
            episode: 1,
            provider: item.id.provider,
        }
    }
}

/// Metrics used to sort browsable catalogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowseMetric {
    /// Trending score.
    Trending,
    /// All-time rating.
    Rating,
    /// Recent release rating.
    RecentRating,
    /// Playback count / popularity.
    Popularity,
}

/// Predefined browse filter categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowsePreset {
    /// Popular trending titles.
    Trending,
    /// Highest rated of all time.
    TopRatedAllTime,
    /// Highly rated recent releases.
    TopRatedRecent,
    /// Most watched content.
    MostWatched,
}

impl BrowsePreset {
    /// List of all default browse presets.
    pub const ALL: [Self; 4] = [
        Self::Trending,
        Self::TopRatedAllTime,
        Self::TopRatedRecent,
        Self::MostWatched,
    ];

    /// Human-friendly display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Trending => "Trending Now",
            Self::TopRatedAllTime => "Top Rated (All-Time)",
            Self::TopRatedRecent => "Top Rated (Recent Releases)",
            Self::MostWatched => "Most Watched",
        }
    }

    /// Associated sorting metric.
    pub fn metric(self) -> BrowseMetric {
        match self {
            Self::Trending => BrowseMetric::Trending,
            Self::TopRatedAllTime => BrowseMetric::Rating,
            Self::TopRatedRecent => BrowseMetric::RecentRating,
            Self::MostWatched => BrowseMetric::Popularity,
        }
    }

    /// Sort direction.
    pub fn descending(self) -> bool {
        true
    }
}

/// Aggregate metrics cache for browse listings.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BrowseMetrics {
    /// Trending score.
    pub trending: Option<f64>,
    /// All-time rating score.
    pub rating: Option<f64>,
    /// Recent rating score.
    pub recent_rating: Option<f64>,
    /// Popularity score.
    pub popularity: Option<f64>,
}

impl BrowseMetrics {
    /// Retrieves metric value by category.
    pub fn value(self, metric: BrowseMetric) -> Option<f64> {
        match metric {
            BrowseMetric::Trending => self.trending,
            BrowseMetric::Rating => self.rating,
            BrowseMetric::RecentRating => self.recent_rating,
            BrowseMetric::Popularity => self.popularity,
        }
    }
}

/// Cache of streams and pagination state for a specific subject.
#[derive(Debug, Default, Clone)]
pub struct SubjectStreamPool {
    /// Releases grouped by (season, episode).
    pub episode_index: HashMap<(usize, usize), Vec<Release>>,
    /// Track of already fetched page indices per resolution.
    pub fetched_pages: HashMap<u32, HashSet<usize>>,
    /// Total pages available per resolution.
    pub total_pages: HashMap<u32, usize>,
    /// Available resolution tiers.
    pub available_resolutions: Vec<u32>,
}

/// Urgency level of an on-screen notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    /// Informational message.
    Info,
    /// Success confirmation.
    Success,
    /// Warning message.
    Warning,
    /// Critical error.
    Error,
}

impl NotificationKind {
    /// Duration before auto-dismissal.
    pub fn total_duration(&self) -> Duration {
        match self {
            NotificationKind::Info => Duration::from_secs(4),
            NotificationKind::Success => Duration::from_secs(5),
            NotificationKind::Warning => Duration::from_secs(7),
            NotificationKind::Error => Duration::from_secs(10),
        }
    }
}

/// Toast notification displayed in the TUI status area.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Notification category.
    pub kind: NotificationKind,
    /// Short notification title.
    pub title: String,
    /// Message body.
    pub message: String,
    /// Expiration timestamp.
    pub expires_at: Instant,
}

impl Notification {
    /// Creates a new notification with default duration.
    pub fn new(
        kind: NotificationKind,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let duration = kind.total_duration();
        Self {
            kind,
            title: title.into(),
            message: message.into(),
            expires_at: Instant::now() + duration,
        }
    }

    /// Checks if this notification has expired.
    pub fn expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}
