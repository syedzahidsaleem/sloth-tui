# Sloth — Technical Design Document (TDD)

> **Version:** 1.0.0  
> **Status:** Reference Implementation Spec  
> **Last Updated:** 2026-09-18  

---

## 1. Introduction

This document specifies the technical design of every major component in Sloth. It is the implementation contract — every Antigravity prompt for code generation must conform to the types, signatures, and behaviors defined here.

---

## 2. Provider Subsystem

### 2.1 Core Types

```rust
// src/providers/models.rs

use serde::{Deserialize, Serialize};

/// What kind of content a Media item represents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Series,
    Anime,
    LiveSport,
    F1,
    IptvChannel,
}

/// Stream quality tiers
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Quality {
    UHD4K,
    FHD1080,
    HD720,
    SD480,
    SD360,
    Auto,
    Unknown,
}

impl Quality {
    pub fn label(&self) -> &'static str {
        match self {
            Self::UHD4K    => "4K",
            Self::FHD1080  => "1080p",
            Self::HD720    => "720p",
            Self::SD480    => "480p",
            Self::SD360    => "360p",
            Self::Auto     => "Auto",
            Self::Unknown  => "?",
        }
    }
}

/// A reference to a specific episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRef {
    pub season: u32,
    pub episode: u32,
    pub title: Option<String>,
    pub duration_secs: Option<f64>,
}

/// A resolved stream URL ready for player consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUrl {
    pub url: String,
    pub quality: Quality,
    pub is_hls: bool,           // M3U8 vs direct MP4/MKV
    pub headers: Vec<(String, String)>, // custom headers (Referer, Origin, etc.)
    pub subtitle_url: Option<String>,
    pub provider_id: &'static str,
}

/// A media item discovered via search or browse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: String,             // provider-scoped: "moviebox:12345"
    pub provider_id: &'static str,
    pub kind: MediaType,
    pub title: String,
    pub year: Option<u32>,
    pub poster_url: Option<String>,
    pub rating: Option<f32>,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub total_episodes: Option<u32>,
    pub total_seasons: Option<u32>,
    pub external_ids: ExternalIds,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalIds {
    pub tmdb: Option<u32>,
    pub anilist: Option<u32>,
    pub mal: Option<u32>,
    pub trakt: Option<String>,
    pub imdb: Option<String>,
}

/// Enriched metadata from metadata providers (TMDB, AniList)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub rating: Option<f32>,
    pub vote_count: Option<u32>,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<CastMember>,
    pub trailer_url: Option<String>,
    pub recommendations: Vec<Media>,
    pub episodes: Option<Vec<EpisodeRef>>,  // for series/anime
    pub airing_schedule: Option<AiringSchedule>, // for anime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMember {
    pub name: String,
    pub character: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiringSchedule {
    pub status: AiringStatus,
    pub next_episode_at: Option<chrono::DateTime<chrono::Utc>>,
    pub season: Option<String>,    // "Fall 2026"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiringStatus {
    Airing,
    Finished,
    NotYetAired,
    Cancelled,
    Hiatus,
}

/// Provider error types
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("provider is rate limiting us (HTTP 429)")]
    RateLimited,

    #[error("content not found on this provider")]
    NotFound,

    #[error("provider response parsing failed: {0}")]
    Parsing(String),

    #[error("provider is temporarily unavailable")]
    Unavailable,

    #[error("auth required — configure in Settings")]
    AuthRequired,
}

impl ProviderError {
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Network(_)    => "Network error. Check your connection.",
            Self::RateLimited   => "Too many requests. Trying next source...",
            Self::NotFound      => "Not found on this provider.",
            Self::Parsing(_)    => "Provider response changed. Trying fallback...",
            Self::Unavailable   => "Provider is down. Trying next source...",
            Self::AuthRequired  => "Login required. Check Settings > Accounts.",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Unavailable)
    }
}
```

### 2.2 Provider Trait

```rust
// src/providers/mod.rs

use async_trait::async_trait;
use crate::providers::models::*;

#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub search: bool,
    pub movies: bool,
    pub tv_series: bool,
    pub anime: bool,
    pub live_sports: bool,
    pub iptv: bool,
    pub f1: bool,
    pub subtitles: bool,
    pub download: bool,
    pub quality_selection: bool,
    pub dub_sub_toggle: bool,      // anime-specific
}

#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Unique machine identifier
    fn id(&self) -> &'static str;

    /// Human-readable name for UI display
    fn name(&self) -> &'static str;

    /// Declared capability set
    fn capabilities(&self) -> ProviderCapabilities;

    /// Search for content matching query
    async fn search(
        &self,
        query: &str,
        kind: MediaType,
    ) -> Result<Vec<Media>, ProviderError>;

    /// Resolve a stream URL for the given media and optional episode
    async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError>;

    /// Fetch episode list for series/anime (optional)
    async fn episodes(
        &self,
        media: &Media,
        season: u32,
    ) -> Result<Vec<EpisodeRef>, ProviderError> {
        Err(ProviderError::NotFound)
    }

    /// Metadata enrichment (optional — metadata providers implement this)
    async fn metadata(
        &self,
        media: &Media,
    ) -> Option<MediaMetadata> {
        None
    }

    /// Healthcheck: returns false if provider is currently down
    async fn health(&self) -> bool {
        true
    }
}
```

### 2.3 Provider Registry

```rust
// src/providers/registry.rs

pub struct ProviderRegistry {
    movie_chain:  Vec<Arc<dyn Provider>>,
    anime_chain:  Vec<Arc<dyn Provider>>,
    sports_chain: Vec<Arc<dyn Provider>>,
    f1_chain:     Vec<Arc<dyn Provider>>,
    tv_chain:     Vec<Arc<dyn Provider>>,
    health:       Arc<RwLock<HashMap<&'static str, bool>>>,
}

impl ProviderRegistry {
    pub fn new(config: &Config) -> Self {
        Self {
            movie_chain: vec![
                Arc::new(MovieBoxProvider::new()),
                Arc::new(FourKHDHubProvider::new()),
                Arc::new(StremioAddonProvider::from_config(config)),
            ],
            anime_chain: vec![
                Arc::new(HiAnimeProvider::new()),
                Arc::new(AllAnimeProvider::new()),
            ],
            sports_chain: vec![
                Arc::new(StreamedProvider::new()),
                Arc::new(IptvSportsProvider::new()),
            ],
            f1_chain: vec![
                Arc::new(F1StreamsProvider::new()),
                Arc::new(IptvF1Provider::new()),
            ],
            tv_chain: vec![
                Arc::new(IptvOrgProvider::from_config(config)),
            ],
            health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn chain_for(&self, kind: &MediaType) -> &[Arc<dyn Provider>] {
        match kind {
            MediaType::Movie | MediaType::Series => &self.movie_chain,
            MediaType::Anime                     => &self.anime_chain,
            MediaType::LiveSport                 => &self.sports_chain,
            MediaType::F1                        => &self.f1_chain,
            MediaType::IptvChannel               => &self.tv_chain,
        }
    }

    pub async fn search(
        &self,
        query: &str,
        kind: MediaType,
    ) -> Vec<Media> {
        let chain = self.chain_for(&kind);
        let futures: Vec<_> = chain.iter()
            .filter(|p| self.is_healthy(p.id()))
            .map(|p| p.search(query, kind.clone()))
            .collect();

        // Run all providers in parallel, merge and deduplicate
        let results = futures::future::join_all(futures).await;
        let mut merged: Vec<Media> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();

        // Deduplicate by title (simple fuzzy match)
        merged.dedup_by(|a, b| a.title.to_lowercase() == b.title.to_lowercase());
        merged
    }

    pub async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, SlothError> {
        let chain = self.chain_for(&media.kind);
        let mut last_err = None;

        for provider in chain {
            if !self.is_healthy(provider.id()) { continue; }

            match provider.resolve(media, episode).await {
                Ok(urls) if !urls.is_empty() => return Ok(urls),
                Ok(_) => continue,
                Err(ProviderError::RateLimited) => continue,
                Err(e) => {
                    tracing::warn!(
                        provider = provider.id(),
                        error = ?e,
                        "Provider failed, trying next"
                    );
                    last_err = Some(e);
                }
            }
        }

        Err(SlothError::NoProvidersAvailable {
            context: last_err.map(|e| e.to_string()),
        })
    }

    fn is_healthy(&self, id: &'static str) -> bool {
        self.health.read().get(id).copied().unwrap_or(true)
    }

    pub async fn run_health_checks(&self) {
        let checks: Vec<_> = self.all_providers()
            .iter()
            .map(|p| {
                let p = Arc::clone(p);
                async move { (p.id(), p.health().await) }
            })
            .collect();

        let results = futures::future::join_all(checks).await;
        let mut map = self.health.write();
        for (id, ok) in results {
            map.insert(id, ok);
        }
    }
}
```

---

## 3. Anime Subsystem

### 3.1 HiAnime Provider Spec

HiAnime (aniwatch.to / hianime.to) serves anime via a JavaScript-rendered frontend. The API used is the public JSON API that the frontend calls.

**Endpoints:**
```
GET https://hianime.to/api/v2/hianime/search?q={query}&page=1
GET https://hianime.to/api/v2/hianime/episode/sources?animeEpisodeId={id}&server=hd-1&category={sub|dub}
```

**Implementation pattern:**
```rust
pub struct HiAnimeProvider {
    client: Arc<reqwest::Client>,
    base_url: &'static str,  // "https://hianime.to"
}

impl HiAnimeProvider {
    /// Extract M3U8 URL from episode sources response
    async fn extract_hls_url(&self, episode_id: &str, category: &str) 
        -> Result<StreamUrl, ProviderError> 
    {
        let resp: HiAnimeSourcesResponse = self.client
            .get(format!("{}/api/v2/hianime/episode/sources", self.base_url))
            .query(&[
                ("animeEpisodeId", episode_id),
                ("server", "hd-1"),
                ("category", category),
            ])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                ProviderError::RateLimited
            } else {
                ProviderError::Network(e)
            })?
            .json()
            .await?;

        let url = resp.sources
            .into_iter()
            .max_by_key(|s| s.quality_rank())
            .ok_or(ProviderError::NotFound)?
            .url;

        Ok(StreamUrl {
            url,
            quality: Quality::Auto,
            is_hls: true,
            headers: vec![
                ("Referer".into(), "https://hianime.to".into()),
            ],
            subtitle_url: resp.tracks
                .into_iter()
                .find(|t| t.kind == "captions" && t.label.contains("English"))
                .map(|t| t.file),
            provider_id: self.id(),
        })
    }
}
```

### 3.2 AniSkip Integration

```rust
// src/providers/anime/aniskip.rs

const ANISKIP_API: &str = "https://api.aniskip.com/v1";

pub struct AniSkipTimestamps {
    pub op_start: Option<f64>,
    pub op_end: Option<f64>,
    pub ed_start: Option<f64>,
    pub ed_end: Option<f64>,
}

pub async fn fetch_timestamps(
    mal_id: u32,
    episode: u32,
) -> Option<AniSkipTimestamps> {
    // GET /skip-times/{malId}/{episodeNumber}?types[]=op&types[]=ed
    todo!()
}
```

### 3.3 AniList OAuth + Sync

```rust
// src/tracking/anilist_sync.rs

const ANILIST_API: &str = "https://graphql.anilist.co";
const ANILIST_AUTH: &str = "https://anilist.co/api/v2/oauth";

// The AniList OAuth flow is PKCE-based (no client secret needed)
// 1. Open browser to: https://anilist.co/api/v2/oauth/authorize?client_id=...&response_type=token
// 2. User pastes the token back into the terminal prompt
// 3. Store in DB auth_tokens table

pub struct AniListClient {
    http: Arc<reqwest::Client>,
    token: Option<String>,
}

impl AniListClient {
    /// Mark episode as watched via SaveMediaListEntry mutation
    pub async fn mark_episode_watched(
        &self,
        anilist_id: u32,
        episode: u32,
    ) -> Result<(), anyhow::Error> {
        let query = r#"
            mutation ($mediaId: Int, $progress: Int) {
                SaveMediaListEntry (mediaId: $mediaId, progress: $progress) {
                    id
                    progress
                    status
                }
            }
        "#;
        // POST to ANILIST_API with Authorization: Bearer {token}
        todo!()
    }

    /// Fetch current airing schedule
    pub async fn fetch_airing_schedule(&self, season: AniListSeason, year: i32)
        -> Result<Vec<AiringAnime>, anyhow::Error>
    {
        todo!()
    }
}
```

---

## 4. Sports Subsystem

### 4.1 Streamed.pk Provider

The streamed.pk site exposes a public JSON API.

**Key Endpoints:**
```
GET https://streamed.su/api/matches/live          -> live matches
GET https://streamed.su/api/matches/all-sports    -> upcoming/all
GET https://streamed.su/api/stream/{sport}/{id}   -> stream URLs
```

**State machine for Sports tab:**
```
SportsList (left column)
    │ select sport
    ▼
MatchList (middle column) [filtered by sport]
    │ select match  
    ▼
StreamList (right column) [sources for match]
    │ select stream
    ▼
Player launch
```

```rust
// src/providers/sports/streamed.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMatch {
    pub id: String,
    pub title: String,
    pub category: String,       // "football", "basketball", etc.
    pub teams: Option<(String, String)>,
    pub competition: Option<String>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_popular: bool,
    pub poster: Option<String>,
    pub streams: Vec<MatchStream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStream {
    pub id: String,
    pub hd_url: Option<String>,
    pub sd_url: Option<String>,
    pub embedUrl: Option<String>,
    pub language: Option<String>,
}
```

### 4.2 F1 Calendar

```rust
// src/providers/f1/calendar.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct F1Session {
    pub round: u32,
    pub name: String,           // "Bahrain Grand Prix"
    pub circuit: String,
    pub country: String,
    pub sessions: Vec<F1SessionSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct F1SessionSlot {
    pub kind: F1SessionKind,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub stream_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum F1SessionKind {
    FreePractice1,
    FreePractice2,
    FreePractice3,
    Qualifying,
    Sprint,
    SprintQualifying,
    Race,
}

impl F1SessionKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FreePractice1    => "Free Practice 1",
            Self::FreePractice2    => "Free Practice 2",
            Self::FreePractice3    => "Free Practice 3",
            Self::Qualifying       => "Qualifying",
            Self::Sprint           => "Sprint",
            Self::SprintQualifying => "Sprint Qualifying",
            Self::Race             => "Race",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Race             => "🏁",
            Self::Qualifying       => "⏱",
            Self::Sprint           => "⚡",
            Self::SprintQualifying => "⚡⏱",
            _                      => "🔧",
        }
    }
}
```

---

## 5. Metadata Subsystem

### 5.1 TMDB Client

```rust
// src/metadata/tmdb.rs

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w500";

pub struct TmdbClient {
    http: Arc<reqwest::Client>,
    api_key: Option<String>,    // None = no enrichment
}

impl TmdbClient {
    pub async fn search_movie(&self, query: &str) -> Result<Vec<TmdbMovie>> { todo!() }
    pub async fn movie_details(&self, id: u32) -> Result<TmdbMovie> { todo!() }
    pub async fn tv_details(&self, id: u32) -> Result<TmdbTv> { todo!() }
    pub async fn tv_season(&self, id: u32, season: u32) -> Result<TmdbSeason> { todo!() }

    pub fn poster_url(&self, path: &str) -> String {
        format!("{}{}", TMDB_IMAGE_BASE, path)
    }
}
```

---

## 6. Player Subsystem

### 6.1 mpv IPC Integration

mpv exposes a JSON IPC protocol over a Unix socket (Linux/macOS) or named pipe (Windows).

```rust
// src/player/mpv.rs

pub struct MpvPlayer {
    child: tokio::process::Child,
    ipc_path: PathBuf,
}

impl MpvPlayer {
    pub async fn spawn(
        stream: &StreamUrl,
        resume_pos: Option<f64>,
        config: &PlayerConfig,
    ) -> Result<Self> {
        let ipc_path = Self::ipc_socket_path();
        let mut cmd = tokio::process::Command::new("mpv");

        cmd.arg(&stream.url)
           .arg("--no-terminal")
           .arg(format!("--input-ipc-server={}", ipc_path.display()));

        // Custom headers
        for (k, v) in &stream.headers {
            cmd.arg(format!("--http-header-fields={}: {}", k, v));
        }

        // Resume position
        if let Some(pos) = resume_pos {
            cmd.arg(format!("--start={}", pos));
        }

        // Subtitle
        if let Some(sub) = &stream.subtitle_url {
            cmd.arg(format!("--sub-file={}", sub));
        }

        // HLS specific
        if stream.is_hls {
            cmd.arg("--demuxer-lavf-o=protocol_whitelist=file,http,https,tcp,tls,crypto");
        }

        let child = cmd.spawn()?;
        // Wait briefly for IPC socket to be created
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(Self { child, ipc_path })
    }

    /// Read current playback position via IPC
    pub async fn get_position(&self) -> Option<f64> {
        // Connect to IPC, send: {"command": ["get_property", "time-pos"]}\n
        // Parse: {"data": 3847.2, "request_id": 0, "error": "success"}
        todo!()
    }

    /// Wait for mpv to exit and return final position
    pub async fn wait_for_exit(&mut self) -> Option<f64> {
        let pos = self.get_position().await;
        let _ = self.child.wait().await;
        pos
    }

    fn ipc_socket_path() -> PathBuf {
        #[cfg(unix)]
        { std::env::temp_dir().join("sloth-mpv.sock") }
        #[cfg(windows)]
        { PathBuf::from(r"\\.\pipe\sloth-mpv") }
    }
}
```

---

## 7. TUI State Machine

### 7.1 App State

```rust
// src/tui/state.rs (extended)

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Movies,
    Anime,
    Sports,
    F1,
    LiveTV,
    History,
    Favorites,
    Settings,
}

#[derive(Debug, Clone, Default)]
pub struct AnimeTabState {
    pub results: Vec<Media>,
    pub selected_idx: usize,
    pub is_dub: bool,               // false = sub (default)
    pub airing_schedule: Vec<AiringAnime>,
    pub schedule_visible: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SportsTabState {
    pub sports: Vec<String>,        // ["football", "basketball", ...]
    pub selected_sport_idx: usize,
    pub matches: Vec<LiveMatch>,
    pub selected_match_idx: usize,
    pub streams: Vec<MatchStream>,
    pub selected_stream_idx: usize,
    pub focus: SportsColumnFocus,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SportsColumnFocus {
    #[default]
    Sports,
    Matches,
    Streams,
}

#[derive(Debug, Clone, Default)]
pub struct F1TabState {
    pub calendar: Vec<F1Session>,
    pub selected_session_idx: usize,
    pub countdown: Option<std::time::Duration>,
    pub loading: bool,
}
```

### 7.2 Countdown Widget

```rust
// src/tui/widgets/countdown.rs

pub struct CountdownWidget {
    pub duration: std::time::Duration,
    pub label: String,
}

impl Widget for CountdownWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let total = self.duration.as_secs();
        let days    = total / 86400;
        let hours   = (total % 86400) / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;

        let text = if days > 0 {
            format!("{}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
        } else {
            format!("{:02}m {:02}s", minutes, seconds)
        };

        // Render with accent color from current theme
        // ...
    }
}
```

---

## 8. Theme System

### 8.1 Theme Definition

```rust
// src/tui/theme.rs

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    // Backgrounds
    pub bg:          Color,
    pub bg_elevated: Color,
    pub bg_surface:  Color,
    // Text
    pub text:        Color,
    pub text_dim:    Color,
    pub text_muted:  Color,
    // Accent
    pub accent:      Color,
    pub accent_dim:  Color,
    // Status
    pub success:     Color,
    pub warning:     Color,
    pub error:       Color,
    pub info:        Color,
    // Borders
    pub border:      Color,
    pub border_focus:Color,
    // Tab bar
    pub tab_active:  Color,
    pub tab_inactive:Color,
}

pub const CATPPUCCIN_MOCHA: Theme = Theme {
    name:          "catppuccin-mocha",
    bg:            Color::Rgb(30, 30, 46),
    bg_elevated:   Color::Rgb(36, 36, 54),
    bg_surface:    Color::Rgb(49, 50, 68),
    text:          Color::Rgb(205, 214, 244),
    text_dim:      Color::Rgb(166, 173, 200),
    text_muted:    Color::Rgb(108, 112, 134),
    accent:        Color::Rgb(137, 180, 250),  // Blue
    accent_dim:    Color::Rgb(116, 199, 236),  // Sapphire
    success:       Color::Rgb(166, 227, 161),  // Green
    warning:       Color::Rgb(249, 226, 175),  // Yellow
    error:         Color::Rgb(243, 139, 168),  // Red
    info:          Color::Rgb(137, 220, 235),  // Sky
    border:        Color::Rgb(88, 91, 112),
    border_focus:  Color::Rgb(137, 180, 250),
    tab_active:    Color::Rgb(137, 180, 250),
    tab_inactive:  Color::Rgb(108, 112, 134),
};

pub const TOKYO_NIGHT: Theme = Theme {
    name:          "tokyo-night",
    bg:            Color::Rgb(26, 27, 38),
    bg_elevated:   Color::Rgb(31, 32, 47),
    bg_surface:    Color::Rgb(41, 46, 66),
    text:          Color::Rgb(169, 177, 214),
    text_dim:      Color::Rgb(122, 129, 165),
    text_muted:    Color::Rgb(86, 95, 137),
    accent:        Color::Rgb(122, 162, 247),
    accent_dim:    Color::Rgb(187, 154, 247),
    success:       Color::Rgb(158, 206, 106),
    warning:       Color::Rgb(224, 175, 104),
    error:         Color::Rgb(247, 118, 142),
    info:          Color::Rgb(125, 207, 255),
    border:        Color::Rgb(58, 65, 105),
    border_focus:  Color::Rgb(122, 162, 247),
    tab_active:    Color::Rgb(122, 162, 247),
    tab_inactive:  Color::Rgb(86, 95, 137),
};

// Nord, Dracula, Gruvbox, Rose Pine follow same pattern...

pub fn theme_by_name(name: &str) -> &'static Theme {
    match name {
        "catppuccin-mocha" => &CATPPUCCIN_MOCHA,
        "tokyo-night"      => &TOKYO_NIGHT,
        "nord"             => &NORD,
        "dracula"          => &DRACULA,
        "gruvbox"          => &GRUVBOX_DARK,
        "rose-pine"        => &ROSE_PINE,
        _                  => &CATPPUCCIN_MOCHA,
    }
}
```

---

## 9. Configuration

### 9.1 Config Struct

```rust
// src/config.rs

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    pub player: PlayerConfig,
    pub theme: ThemeConfig,
    pub providers: ProvidersConfig,
    pub image: ImageConfig,
    pub search: SearchConfig,
    pub notifications: NotificationsConfig,
    pub download: DownloadConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerConfig {
    pub preferred: String,                  // "mpv"
    pub custom_command: Option<String>,
    pub extra_args: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            preferred: "mpv".into(),
            custom_command: None,
            extra_args: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvidersConfig {
    pub tmdb: TmdbConfig,
    pub anilist: AnilistConfig,
    pub trakt: TraktConfig,
    pub stremio_addons: Vec<String>,        // addon manifest URLs
    pub iptv_url: Option<String>,           // custom M3U URL
    pub disabled: Vec<String>,             // provider ids to disable
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TmdbConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnilistConfig {
    pub enabled: bool,
    pub client_id: Option<String>,
}

impl Default for AnilistConfig {
    fn default() -> Self {
        Self { enabled: true, client_id: None }
    }
}
```

---

## 10. Error Hierarchy

```rust
// src/lib.rs (top-level error)

#[derive(Debug, thiserror::Error)]
pub enum SlothError {
    #[error("no providers available: {context:?}")]
    NoProvidersAvailable { context: Option<String> },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("auth error: {0}")]
    Auth(String),
}
```

---

## 11. Testing Approach

### Unit Tests (per module)
- Parser functions with fixture JSON/HTML
- Stream URL extraction correctness
- Quality ranking logic
- Cache TTL expiry
- Database CRUD operations

### Integration Tests (tests/ directory)
- Content pipeline: search -> resolve (with wiremock mocked HTTP)
- Provider fallback: primary fails -> secondary resolves
- AniList sync: dirty queue -> mock push -> clear dirty flag
- mpv IPC: mock socket -> position read
- SQLite migrations: v1 schema applied cleanly

### Snapshot Tests (insta)
- TUI screen renders for each tab (snapshot of Ratatui buffer)
- Theme rendering (colors applied correctly)

### Performance Tests
- Search latency under load (tokio::time test utilities)
- LRU cache hit/miss ratio
- Database query timing assertions
