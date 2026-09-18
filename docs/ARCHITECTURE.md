# Sloth — Architecture Document

> **Version:** 1.0.0  
> **Status:** Reference Design  
> **Last Updated:** 2026-09-18  

---

## 1. High-Level Overview

Sloth is a single-binary Rust application built on Ratatui. It follows a strict layered architecture where the TUI layer never touches network code directly — all I/O goes through the Provider abstraction layer and is dispatched via an async Action channel.

```
┌─────────────────────────────────────────────────────────────────┐
│                          TUI Layer                               │
│   Screens (Home, Details, Sports, F1, Anime, Settings, Help)    │
│   Widgets (Poster, Badge, Modal, Input, Scrollbar, Settings)    │
│   Theme Engine  |  Event Loop  |  Action Channel               │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Action
┌──────────────────────────▼──────────────────────────────────────┐
│                       Service Layer                              │
│   SearchService  |  MetadataService  |  TrackingService         │
│   CacheService   |  NotifierService  |  DiscordRpcService       │
└──────────────────────────┬──────────────────────────────────────┘
                           │ ProviderRequest
┌──────────────────────────▼──────────────────────────────────────┐
│                      Provider Layer                              │
│                                                                  │
│  Movies/TV           Anime              Sports          F1      │
│  ┌──────────┐   ┌──────────────┐   ┌──────────┐   ┌────────┐  │
│  │MovieBox  │   │HiAnime       │   │Streamed  │   │F1Cal.  │  │
│  │4KHDHub   │   │AllAnime      │   │IPTV-org  │   │IPTV-F1 │  │
│  │Addons    │   │AniList Meta  │   │          │   │        │  │
│  └──────────┘   └──────────────┘   └──────────┘   └────────┘  │
│                                                                  │
│  IPTV/Live TV        Metadata           Tracking               │
│  ┌──────────┐   ┌──────────────┐   ┌──────────────────────┐   │
│  │M3U Parse │   │TMDB          │   │AniList OAuth         │   │
│  │EPG Parse │   │AniList       │   │Trakt.tv OAuth        │   │
│  └──────────┘   └──────────────┘   │SQLite Local          │   │
│                                     └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                    Infrastructure Layer                          │
│   reqwest HttpClient  |  LRU Cache  |  SQLite (sqlx)           │
│   mpv IPC Socket      |  FS Paths   |  Config (TOML)           │
│   tokio Runtime       |  mimalloc   |  tracing Logger          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Map

### 2.1 Source Tree

```
sloth-tui/
├── Cargo.toml
├── Cargo.lock
├── .cargo/
│   └── config.toml           # LTO, codegen-units, target features
├── src/
│   ├── main.rs               # Entry point, tokio runtime, TerminalGuard
│   ├── lib.rs                # Re-exports for integration tests
│   │
│   ├── config.rs             # TOML config struct + dirs::config_dir
│   ├── models.rs             # Shared data types (MediaType, Quality, etc.)
│   ├── net.rs                # Shared reqwest Client factory (UA, timeouts)
│   ├── logging.rs            # tracing_subscriber setup, log rotation
│   ├── service.rs            # High-level orchestration, subtitle dirs
│   │
│   ├── providers/
│   │   ├── mod.rs            # Provider trait + ProviderCapabilities
│   │   ├── models.rs         # Media, StreamUrl, Episode, ProviderError
│   │   ├── registry.rs       # Provider registry + fallback chain logic
│   │   │
│   │   ├── moviebox/         # [EXISTING — from MovieBox-Tui]
│   │   │   ├── mod.rs
│   │   │   ├── client.rs
│   │   │   ├── adapt.rs
│   │   │   ├── crypto.rs
│   │   │   ├── session.rs
│   │   │   └── title.rs
│   │   │
│   │   ├── fourkhdhub/       # [EXISTING — from MovieBox-Tui]
│   │   │   ├── mod.rs
│   │   │   ├── client.rs
│   │   │   ├── hubcloud.rs
│   │   │   └── parser.rs
│   │   │
│   │   ├── addons/           # [EXISTING — Stremio addon framework]
│   │   │   ├── mod.rs
│   │   │   ├── adapter.rs
│   │   │   ├── aggregator.rs
│   │   │   ├── client.rs
│   │   │   └── models.rs
│   │   │
│   │   ├── bdix/             # [EXISTING — Bangladesh BDIX providers]
│   │   │   ├── mod.rs
│   │   │   ├── common.rs
│   │   │   ├── circleftp/
│   │   │   └── dhakaflix/
│   │   │
│   │   ├── anime/            # [NEW]
│   │   │   ├── mod.rs        # AnimeProvider trait
│   │   │   ├── hianime.rs    # HiAnime/AniWatch scraper (primary)
│   │   │   ├── allanime.rs   # AllAnime GraphQL API (dub fallback)
│   │   │   └── aniskip.rs    # AniSkip API for intro/outro timestamps
│   │   │
│   │   ├── sports/           # [NEW]
│   │   │   ├── mod.rs        # SportsProvider trait
│   │   │   ├── streamed.rs   # streamed.pk REST API
│   │   │   └── iptv_sports.rs # iptv-org sport M3U bundles
│   │   │
│   │   ├── f1/               # [NEW]
│   │   │   ├── mod.rs        # F1Provider trait
│   │   │   ├── calendar.rs   # Season calendar + countdown
│   │   │   ├── streams.rs    # F1 stream resolution
│   │   │   └── iptv_f1.rs   # iptv-org F1 channels
│   │   │
│   │   └── tv/               # [EXISTING — IPTV M3U]
│   │       ├── mod.rs
│   │       ├── models.rs
│   │       └── parser.rs
│   │
│   ├── metadata/             # [NEW]
│   │   ├── mod.rs
│   │   ├── tmdb.rs           # TMDB v3 REST API
│   │   └── anilist.rs        # AniList GraphQL API
│   │
│   ├── tracking/             # [NEW]
│   │   ├── mod.rs
│   │   ├── anilist_sync.rs   # AniList OAuth2 + mutation sync
│   │   ├── trakt.rs          # Trakt.tv OAuth2 + scrobble
│   │   └── discord_rpc.rs    # Discord Rich Presence
│   │
│   ├── player/               # [EXISTING + EXTENDED]
│   │   ├── mod.rs (was player.rs)
│   │   ├── mpv.rs            # mpv launch + IPC socket
│   │   ├── vlc.rs            # VLC launch
│   │   ├── iina.rs           # IINA (macOS) launch
│   │   └── tracker.rs        # [EXISTING] mpv position tracker
│   │
│   ├── db/                   # [NEW]
│   │   ├── mod.rs
│   │   ├── schema.sql        # SQLite schema
│   │   ├── history.rs        # Watch history CRUD
│   │   └── favorites.rs      # Favorites CRUD (replaces src/favorites.rs)
│   │
│   ├── daemon/               # [NEW]
│   │   └── notifier.rs       # Background OS notification task
│   │
│   ├── cache.rs              # [EXISTING] LRU cache for images
│   ├── download.rs           # [EXISTING] Download with resume
│   ├── favorites.rs          # [EXISTING — migrated to db/favorites.rs]
│   ├── history.rs            # [EXISTING — migrated to db/history.rs]
│   ├── proxy.rs              # [EXISTING] Proxy support
│   ├── updater/              # [EXISTING] Auto-updater
│   │   ├── mod.rs
│   │   ├── apply.rs
│   │   ├── artifact.rs
│   │   ├── check.rs
│   │   ├── download.rs
│   │   ├── extract.rs
│   │   └── verify.rs
│   │
│   └── tui/
│       ├── mod.rs
│       ├── action.rs         # [EXISTING + EXTENDED] Action enum
│       ├── event.rs          # [EXISTING] Event loop
│       ├── terminal.rs       # [EXISTING] Terminal setup/teardown
│       ├── theme.rs          # [EXISTING + EXTENDED] 6 themes
│       ├── text.rs           # [EXISTING] Text utilities
│       ├── overlay.rs        # [EXISTING] Overlay rendering
│       ├── commands.rs       # [EXISTING] Command palette
│       ├── state.rs          # [EXISTING + EXTENDED] App state
│       │
│       ├── app/
│       │   ├── mod.rs
│       │   ├── run.rs        # [EXISTING] Main event loop
│       │   ├── keyboard.rs   # [EXISTING + EXTENDED] Keyboard handling
│       │   ├── mouse.rs      # [EXISTING] Mouse handling
│       │   ├── navigation.rs # [EXISTING + EXTENDED]
│       │   ├── requests.rs   # [EXISTING + EXTENDED]
│       │   ├── playback.rs   # [EXISTING + EXTENDED]
│       │   ├── search.rs     # [EXISTING + EXTENDED]
│       │   ├── system.rs     # [EXISTING]
│       │   ├── addons.rs     # [EXISTING]
│       │   ├── download.rs   # [EXISTING]
│       │   ├── favorites.rs  # [EXISTING]
│       │   ├── tv.rs         # [EXISTING]
│       │   ├── network.rs    # [EXISTING]
│       │   ├── anime.rs      # [NEW] Anime tab logic
│       │   ├── sports.rs     # [NEW] Sports tab logic
│       │   └── f1.rs         # [NEW] F1 tab logic
│       │
│       ├── screens/
│       │   ├── home.rs       # [EXISTING + EXTENDED] Tab bar + all tabs
│       │   ├── details.rs    # [EXISTING + EXTENDED] Rich detail view
│       │   ├── help.rs       # [EXISTING + EXTENDED]
│       │   ├── sports.rs     # [NEW] Three-column sports browser
│       │   ├── f1.rs         # [NEW] F1 calendar screen
│       │   └── anime.rs      # [NEW] Anime browser + schedule
│       │
│       └── widgets/
│           ├── mod.rs
│           ├── badge.rs      # [EXISTING]
│           ├── input.rs      # [EXISTING]
│           ├── modal.rs      # [EXISTING]
│           ├── poster.rs     # [EXISTING + EXTENDED]
│           ├── scrollbar.rs  # [EXISTING]
│           ├── settings.rs   # [EXISTING + EXTENDED]
│           ├── countdown.rs  # [NEW] Countdown timer widget (F1/Sports)
│           └── track_bar.rs  # [NEW] Watch progress bar
│
├── tests/
│   ├── common/mod.rs
│   ├── addons_manifest.rs    # [EXISTING]
│   ├── content_pipeline.rs   # [EXISTING + EXTENDED]
│   ├── error_handling.rs     # [EXISTING + EXTENDED]
│   ├── favorites_lifecycle.rs# [EXISTING]
│   ├── history_audit.rs      # [EXISTING]
│   ├── live_stream_verification.rs # [EXISTING + EXTENDED]
│   ├── performance_audit.rs  # [EXISTING + EXTENDED]
│   ├── settings_hub.rs       # [EXISTING]
│   ├── tui_acceptance.rs     # [EXISTING + EXTENDED]
│   ├── update_lifecycle.rs   # [EXISTING]
│   ├── anime_pipeline.rs     # [NEW]
│   ├── sports_pipeline.rs    # [NEW]
│   └── f1_calendar.rs        # [NEW]
│
└── docs/
    ├── PRD.md
    ├── ARCHITECTURE.md       (this file)
    ├── DESIGN.md
    ├── SCHEMA.md
    ├── DEPENDENCIES.md
    ├── WORKFLOW.md
    ├── IMPLEMENTATION.md
    ├── TDD.md
    ├── RULES.md
    ├── AGENTS.md
    └── PROMPTS.md
```

---

## 3. Core Abstractions

### 3.1 Provider Trait

Every content source implements this unified trait:

```rust
/// Capability flags — providers declare what they support
#[derive(Debug, Clone)]
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
}

/// The one trait every provider implements
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> ProviderCapabilities;

    async fn search(&self, query: &str, kind: MediaType) 
        -> Result<Vec<Media>, ProviderError>;

    async fn resolve(&self, media: &Media, episode: Option<EpisodeRef>) 
        -> Result<Vec<StreamUrl>, ProviderError>;

    /// Optional: return metadata enrichment (poster URL, rating, etc.)
    async fn metadata(&self, media: &Media) 
        -> Option<MediaMetadata> { None }

    /// Optional: healthcheck (HEAD request to provider root)
    async fn health(&self) -> bool { true }
}
```

### 3.2 Fallback Chain

```rust
pub struct ProviderRegistry {
    /// Ordered list per content type — tried in order
    movie_providers: Vec<Arc<dyn Provider>>,
    anime_providers: Vec<Arc<dyn Provider>>,
    sports_providers: Vec<Arc<dyn Provider>>,
    f1_providers: Vec<Arc<dyn Provider>>,
    tv_providers: Vec<Arc<dyn Provider>>,
    /// Health status per provider id
    health_map: Arc<RwLock<HashMap<&'static str, bool>>>,
}

impl ProviderRegistry {
    /// Try providers in order, skip unhealthy, return first success
    pub async fn resolve(
        &self,
        media: &Media,
        episode: Option<EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, SlothError> {
        let providers = self.providers_for(media.kind);
        for provider in providers {
            if !self.is_healthy(provider.id()) { continue; }
            match provider.resolve(media, episode).await {
                Ok(urls) if !urls.is_empty() => return Ok(urls),
                Ok(_) => continue,           // empty, try next
                Err(ProviderError::RateLimited) => continue,
                Err(e) => {
                    tracing::warn!("{}: {:?}", provider.id(), e);
                    continue;
                }
            }
        }
        Err(SlothError::NoProvidersAvailable)
    }
}
```

### 3.3 Action Channel

All TUI state mutations flow through a single `mpsc` channel. This prevents data races and makes the event loop trivially auditable.

```rust
#[derive(Debug, Clone)]
pub enum Action {
    // Navigation
    SwitchTab(Tab),
    NavigateUp, NavigateDown, NavigateLeft, NavigateRight,
    Select, Back,

    // Search
    SearchQuery(String),
    SearchResultsReceived(Vec<Media>),

    // Playback
    PlaybackStarted { media: Media, episode: Option<EpisodeRef> },
    PlaybackEnded { resume_position_secs: Option<f64> },

    // Anime
    AnimeSubDubToggle,
    AnimeAnilistSync,
    AnimeScheduleLoad,

    // Sports
    SportSelected(Sport),
    MatchSelected(Match),
    StreamSelected(StreamUrl),

    // F1
    F1SessionSelected(F1Session),
    F1CalendarLoad,

    // Tracking
    MarkWatched(Media),
    ResumePosition(f64),

    // System
    Tick,
    Resize(u16, u16),
    Error(String),
    Quit,
}
```

### 3.4 App State

```rust
pub struct AppState {
    pub active_tab: Tab,
    pub search_query: String,
    pub search_results: Vec<Media>,
    pub selected_media: Option<Media>,
    pub selected_episode: Option<EpisodeRef>,

    // Per-tab state
    pub movie_state: MovieTabState,
    pub anime_state: AnimeTabState,
    pub sports_state: SportsTabState,
    pub f1_state: F1TabState,
    pub tv_state: TvTabState,

    // Overlay
    pub overlay: Option<Overlay>,

    // Theme
    pub theme: Theme,

    // Loading/error
    pub is_loading: bool,
    pub status_message: Option<StatusMessage>,
}
```

---

## 4. Async Architecture

Sloth uses `tokio` as its async runtime. The main event loop is single-threaded (Ratatui requirement), but all I/O work is spawned as `tokio::spawn` tasks that send results back via the Action channel.

```
Main Thread (tokio current_thread)
    │
    ├── crossterm EventStream → keyboard/mouse events → Action channel
    │
    ├── tokio::time::interval (16ms tick) → Action::Tick
    │
    └── Action mpsc receiver → run.rs handler → state mutation → render

Spawned Tasks (tokio::spawn, on shared tokio threadpool)
    ├── search_task        → sends SearchResultsReceived
    ├── resolve_task       → sends PlaybackStarted or Error
    ├── metadata_task      → sends MetadataReceived
    ├── poster_task        → sends PosterDecoded
    ├── health_check_task  → updates health_map every 10min
    ├── anilist_sync_task  → sends AnilistSynced
    └── notifier_task      → fires OS notifications (daemon)
```

---

## 5. Data Flow: "Search and Play"

```
User types "Dune" in search box
    │
    ▼
Action::SearchQuery("Dune")
    │
    ▼
app/search.rs: spawn tokio task
    │
    ├── ProviderRegistry::search("Dune", MediaType::Movie)
    │     ├── MovieBoxProvider::search("Dune")     [parallel]
    │     ├── FourKHDHubProvider::search("Dune")   [parallel]
    │     └── StremioAddonProvider::search("Dune") [parallel]
    │
    ▼
Action::SearchResultsReceived(results)
    │
    ▼
state: search_results populated, list renders
    │
User presses Enter on "Dune (2021)"
    │
    ▼
Action::Select
    │
    ▼
app/navigation.rs: spawn resolve task
    │
    ├── MetadataService::enrich(media) → TMDB API [parallel]
    └── ProviderRegistry::resolve(media, None)
          ├── MovieBoxProvider::resolve(...)       [try first]
          │     └── Ok([StreamUrl { url, quality }])
          └── [fallbacks not needed]
    │
    ▼
Action::PlaybackStarted { media, episode: None }
    │
    ▼
player/mpv.rs: spawn mpv process with IPC socket
    │
    ▼
tracker.rs: wait for mpv exit, read time-pos from IPC
    │
    ▼
Action::PlaybackEnded { resume_position_secs: Some(3847.2) }
    │
    ▼
db/history.rs: upsert watch_history row
tracking/trakt.rs: scrobble (if enabled)
tracking/anilist_sync.rs: mark episode (if anime)
```

---

## 6. Cache Strategy

```
┌─────────────────────────────────────────────────────┐
│  In-Memory LRU Cache (hot path)                     │
│  - Poster images: LruCache<Url, DecodedImage>       │
│    max 200 entries, ~100MB typical                  │
│  - Provider responses: LruCache<CacheKey, Response> │
│    TTL enforced via stored Instant + max_age        │
└─────────────────────────────────────────────────────┘
           persistent backing store
┌─────────────────────────────────────────────────────┐
│  Filesystem Cache (~/.local/share/sloth-tui/cache/) │
│  - Posters: content-addressed .webp files           │
│    max 500MB, LRU eviction on startup               │
│  - Metadata: {provider}_{id}.json with mtime TTL   │
└─────────────────────────────────────────────────────┘
```

### Cache TTLs

| Content | TTL |
|---|---|
| Movie/TV metadata | 1 hour |
| Anime metadata | 1 hour |
| Sports live schedule | 5 minutes |
| F1 calendar | 24 hours |
| IPTV M3U | 6 hours |
| Poster images | 7 days |
| Provider health | 10 minutes |

---

## 7. Error Handling

All provider errors map to `ProviderError`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("provider is rate limiting us")]
    RateLimited,

    #[error("content not found")]
    NotFound,

    #[error("failed to parse provider response: {0}")]
    Parsing(String),

    #[error("provider is temporarily unavailable")]
    Unavailable,

    #[error("authentication required")]
    AuthRequired,
}

impl ProviderError {
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Network(_) => "Network error. Check your connection.",
            Self::RateLimited => "Too many requests. Trying next source...",
            Self::NotFound => "Content not found on this provider.",
            Self::Parsing(_) => "Provider response changed. Trying fallback...",
            Self::Unavailable => "Provider is down. Trying next source...",
            Self::AuthRequired => "Login required. Check Settings.",
        }
    }
}
```

The TUI layer only ever shows `user_message()`. Raw errors go to the log file only.

---

## 8. Threading Model

| Thread/Task | Purpose | Priority |
|---|---|---|
| Main tokio thread | Event loop + render | Critical |
| Search tasks | Provider HTTP queries | High |
| Resolve tasks | Stream URL resolution | High |
| Metadata tasks | TMDB / AniList enrichment | Medium |
| Poster decode tasks | Image decode + cache write | Low |
| Health check tasks | Provider ping every 10min | Lowest |
| Notifier task | OS notification daemon | Lowest |
| mpv IPC poller | Read playback position | High (during playback only) |

---

## 9. Cross-Platform Considerations

| Feature | Windows | macOS | Linux | Android (Termux) |
|---|---|---|---|---|
| Config path | `%APPDATA%` | `~/.config` | `~/.config` | `~/storage/shared` |
| Data path | `%LOCALAPPDATA%` | `~/.local/share` | `~/.local/share` | `~/storage/shared` |
| Player detect | PATH + Registry | PATH + `/Applications` | PATH | PATH |
| Image protocol | Kitty (if WezTerm) | Kitty (if kitty/ghostty) | Kitty/Sixel | Block only |
| Allocator | mimalloc | mimalloc | mimalloc | system (no mimalloc) |
| IPC socket | Named pipe | Unix socket | Unix socket | Unix socket |
| Notifications | Windows toast | macOS notify | libnotify | termux-notification |

---

## 10. Security Considerations

- No hardcoded API keys in source code — all keys from config file
- HTTPS enforced for all provider requests (reqwest TLS)
- No eval/exec of provider-supplied content (URLs only, passed to player)
- SQLite database is local-only, no sync to cloud
- OAuth tokens stored in OS keyring (keyring crate) where available, otherwise config file
- No telemetry, no analytics, no remote logging

---

## 11. Build Configuration

```toml
# .cargo/config.toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "debuginfo"
panic = "abort"

[profile.dev]
opt-level = 1       # faster incremental builds

[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]
```

---

## 12. Source References

| Component | Derived From | License |
|---|---|---|
| Core TUI (providers, cache, download, player) | [mesamirh/MovieBox-Tui](https://github.com/mesamirh/MovieBox-Tui) | MIT OR Apache-2.0 |
| Sports API client pattern | [Salastil/streamed-tui](https://github.com/Salastil/streamed-tui) | MIT |
| Anime provider patterns | [pranshuj73/oni](https://github.com/pranshuj73/oni) | MIT |
| AniList OAuth pattern | [jerry](https://github.com/justchokingaround/jerry) | GPL-3.0 |
| IPTV channel data | [iptv-org/iptv](https://github.com/iptv-org/iptv) | Unlicense |
| F1 stream patterns | [YAGNIKHARIYANI/f1tv](https://github.com/YAGNIKHARIYANI/f1tv) | MIT |
