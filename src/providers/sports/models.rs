//! Sports domain models for live matches and stream sources.

use serde::{Deserialize, Serialize};

use crate::providers::models::{ExternalIds, Media, MediaType, Quality};

/// A scheduled or in-progress live sporting event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveMatch {
    /// Unique identifier for the match.
    pub id: String,
    /// Human-readable match title (e.g. "Arsenal vs Chelsea").
    pub title: String,
    /// Sport category slug (e.g. "football", "basketball", "cricket").
    pub category: String,
    /// Name of the home team if applicable.
    pub home_team: Option<String>,
    /// Name of the away team if applicable.
    pub away_team: Option<String>,
    /// League, championship, or tournament name.
    pub competition: Option<String>,
    /// Scheduled match start timestamp in UTC.
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the match is currently live and active.
    pub is_live: bool,
    /// Whether the match is flagged as popular or featured.
    pub is_popular: bool,
    /// Direct URL to the match promotional poster or logo.
    pub poster_url: Option<String>,
    /// Available stream sources for this match.
    pub streams: Vec<MatchStream>,
}

/// Playable stream source option for a specific sporting event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchStream {
    /// Stream identifier or stream index.
    pub id: String,
    /// High-definition direct stream URL (m3u8).
    pub hd_url: Option<String>,
    /// Standard-definition direct stream URL (m3u8).
    pub sd_url: Option<String>,
    /// Embeddable web player URL if direct stream is unavailable.
    pub embed_url: Option<String>,
    /// Commentary audio/subtitle language.
    pub language: Option<String>,
    /// Inferred or declared video quality tier.
    pub quality: Quality,
}

impl MatchStream {
    /// Resolves the best playable stream URL available in this source.
    pub fn best_url(&self) -> Option<&str> {
        self.hd_url
            .as_deref()
            .or(self.sd_url.as_deref())
            .or(self.embed_url.as_deref())
    }
}

impl From<LiveMatch> for Media {
    fn from(m: LiveMatch) -> Self {
        Media {
            id: m.id,
            title: m.title,
            media_type: MediaType::LiveSport,
            year: None,
            overview: m.competition,
            poster_url: m.poster_url,
            backdrop_url: None,
            genres: vec![m.category],
            rating: None,
            duration_secs: None,
            seasons_count: None,
            episodes_count: None,
            provider_id: "streamed-pk",
            external_ids: ExternalIds::default(),
            cast: Vec::new(),
        }
    }
}

impl From<MatchStream> for crate::tui::state::MatchStream {
    fn from(s: MatchStream) -> Self {
        Self {
            id: s.id,
            hd_url: s.hd_url,
            sd_url: s.sd_url,
            embed_url: s.embed_url,
            language: s.language,
        }
    }
}

impl From<LiveMatch> for crate::tui::state::LiveMatch {
    fn from(m: LiveMatch) -> Self {
        let teams = match (m.home_team, m.away_team) {
            (Some(h), Some(a)) => Some((h, a)),
            _ => None,
        };
        Self {
            id: m.id,
            title: m.title,
            category: m.category,
            teams,
            competition: m.competition,
            starts_at: m.starts_at,
            is_popular: m.is_popular,
            poster: m.poster_url,
            streams: m.streams.into_iter().map(Into::into).collect(),
        }
    }
}
