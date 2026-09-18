# Sloth — Implementation Plan

> **Version:** 1.0.0  
> **Last Updated:** 2026-09-18  
> **Status:** Execution Roadmap  

---

## Overview

This document is the hands-on execution guide. It maps each milestone to exact files, tasks, code patterns, and verification steps. Antigravity agents follow this document file-by-file when generating code.

---

## M1 — Foundation (Weeks 1–2)

**Goal:** Compilable Sloth project derived from MovieBox-Tui, with new tab architecture and extended theme system.

### M1.1 — Fork & Rename

**Files to create:**
- `Cargo.toml` — rename crate to `sloth-tui`, add all deps from DEPENDENCIES.md
- `.cargo/config.toml` — LTO, codegen-units, strip
- `src/lib.rs` — module declarations + re-exports
- `src/main.rs` — entry point (keep mimalloc, TerminalGuard from MovieBox-Tui)

**Files to copy from MovieBox-Tui (verbatim):**
```
src/cache.rs
src/config.rs           (will be extended in M1.3)
src/download.rs
src/logging.rs
src/models.rs           (will be extended)
src/net.rs
src/proxy.rs
src/service.rs
src/updater/            (all files)
src/providers/moviebox/ (all files)
src/providers/fourkhdhub/ (all files)
src/providers/addons/   (all files)
src/providers/bdix/     (all files)
src/providers/tv/       (all files)
src/tui/action.rs       (extend later)
src/tui/event.rs
src/tui/mod.rs
src/tui/overlay.rs
src/tui/terminal.rs
src/tui/text.rs
src/tui/theme.rs        (extend in M1.4)
src/tui/commands.rs
src/tui/state.rs        (extend in M1.3)
src/tui/widgets/        (all files)
src/tui/app/            (all files)
src/tui/screens/home.rs (extend in M1.3)
src/tui/screens/details.rs
src/tui/screens/help.rs
tests/                  (all existing tests)
```

**Verify:** `cargo check` passes.

---

### M1.2 — Provider Registry

**New file:** `src/providers/registry.rs`

Implement `ProviderRegistry` as defined in TDD.md §2.3:
- `new(config: &Config)` constructor with all existing providers in movie_chain
- `chain_for(&MediaType)` dispatch
- `search()` — parallel across chain, merge, deduplicate
- `resolve()` — sequential fallback with health check
- `run_health_checks()` — async parallel ping
- `is_healthy()` — read from health map

**New file:** `src/providers/mod.rs` — add `Provider` trait (TDD.md §2.2) + `pub mod registry`

**Update:** `src/service.rs` — replace direct provider calls with `ProviderRegistry`

**Verify:** `cargo check`, existing tests still pass.

---

### M1.3 — Extended State & Tabs

**Update `src/tui/state.rs`:**
Add new state structs (from TDD.md §7.1):
```rust
pub enum Tab { Movies, Anime, Sports, F1, LiveTV, History, Favorites, Settings }
pub struct AnimeTabState { ... }
pub struct SportsTabState { ... }
pub struct F1TabState { ... }
```

**Update `src/tui/action.rs`:**
Add new Action variants (from TDD.md §3.3):
```rust
// Anime
AnimeSubDubToggle,
AnimeAnilistSync,
AnimeScheduleLoad,
AnimeScheduleReceived(Vec<AiringAnime>),

// Sports
SportSelected(String),
MatchListReceived(Vec<LiveMatch>),
MatchSelected(String),
StreamListReceived(Vec<MatchStream>),

// F1
F1CalendarLoad,
F1CalendarReceived(Vec<F1Session>),
F1SessionSelected(F1Session),
```

**Update `src/tui/screens/home.rs`:**
- Add 8-tab tab bar at top
- Keep existing Movies content under Tab::Movies
- Add empty placeholder panels for: Anime, Sports, F1, Live TV
- History and Favorites redirect to existing screens

**Verify:** App launches, all 8 tabs visible, Movies tab works as before.

---

### M1.4 — Extended Theme System

**Update `src/tui/theme.rs`:**
Add new color tokens to `Theme` struct (from TDD.md §8.1):
```rust
pub bg_elevated: Color,
pub bg_surface: Color,
pub text_dim: Color,
pub text_muted: Color,
pub tab_active: Color,
pub tab_inactive: Color,
```

Add remaining 5 themes alongside existing ones:
- `CATPPUCCIN_MOCHA` (set as default)
- `TOKYO_NIGHT`
- `NORD`
- `DRACULA`
- `GRUVBOX_DARK`
- `ROSE_PINE`

Update `theme_by_name()` to return all 6.

**Update Settings screen** to list all 6 themes.

**Verify:** All 6 themes selectable, no color compilation errors.

---

### M1.5 — Database Foundation

**New directory:** `src/db/`

**New file:** `src/db/mod.rs`
```rust
pub async fn open(path: &Path) -> Result<SqlitePool>
pub mod history;
pub mod favorites;
```

**New file:** `migrations/0001_initial.sql` — full schema from SCHEMA.md

**New file:** `src/db/history.rs`
```rust
pub async fn upsert(pool: &SqlitePool, entry: &WatchEntry) -> Result<()>
pub async fn get_resume_position(pool: &SqlitePool, media_id: &str, season: u32, episode: u32) -> Option<f64>
pub async fn recent(pool: &SqlitePool, limit: u32) -> Result<Vec<HistoryRow>>
pub async fn continue_watching(pool: &SqlitePool) -> Result<Vec<HistoryRow>>
```

**New file:** `src/db/favorites.rs`
```rust
pub async fn add(pool: &SqlitePool, media_id: &str) -> Result<()>
pub async fn remove(pool: &SqlitePool, media_id: &str) -> Result<()>
pub async fn is_favorite(pool: &SqlitePool, media_id: &str) -> bool
pub async fn list(pool: &SqlitePool) -> Result<Vec<Media>>
```

**Update `src/main.rs`:** Open SQLite pool at startup, pass to App via config/state.

**Verify:** `cargo test db::` passes with temp SQLite file.

---

## M2 — Anime (Weeks 3–4)

**Goal:** Functional anime tab with HiAnime + AllAnime providers, sub/dub toggle, AniList metadata.

### M2.1 — HiAnime Provider

**New directory:** `src/providers/anime/`

**New file:** `src/providers/anime/hianime.rs`

Implement `HiAnimeProvider`:
```rust
pub struct HiAnimeProvider { client: Arc<reqwest::Client> }

impl HiAnimeProvider {
    const BASE: &'static str = "https://hianime.to";
    const API: &'static str = "https://hianime.to/api/v2/hianime";
}

#[async_trait]
impl Provider for HiAnimeProvider {
    fn id(&self) -> &'static str { "hianime" }
    fn name(&self) -> &'static str { "HiAnime" }

    async fn search(&self, query: &str, kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        // GET /search?q={query}&page=1
        // Parse: response.data.animes -> Vec<Media>
    }

    async fn resolve(&self, media: &Media, episode: Option<&EpisodeRef>) 
        -> Result<Vec<StreamUrl>, ProviderError>
    {
        // GET /episode/servers?animeEpisodeId={id}
        // GET /episode/sources?animeEpisodeId={id}&server=hd-1&category={sub|dub}
        // Extract M3U8 URL
    }

    async fn episodes(&self, media: &Media, season: u32)
        -> Result<Vec<EpisodeRef>, ProviderError>
    {
        // GET /episodes/{anime-id}
        // Parse episode list
    }

    async fn health(&self) -> bool {
        // HEAD https://hianime.to -> 200 OK
    }
}
```

**Wiremock fixtures needed:**
- `tests/fixtures/hianime/search.json` — sample search response
- `tests/fixtures/hianime/episodes.json` — sample episode list
- `tests/fixtures/hianime/sources.json` — sample sources response

**Test file:** `tests/anime_pipeline.rs`

---

### M2.2 — AllAnime Provider

**New file:** `src/providers/anime/allanime.rs`

AllAnime uses a GraphQL API at `https://api.allanime.day/api`.

```rust
pub struct AllAnimeProvider { client: Arc<reqwest::Client> }

impl AllAnimeProvider {
    const GRAPHQL_URL: &'static str = "https://api.allanime.day/api";

    async fn gql<T: DeserializeOwned>(&self, query: &str, variables: serde_json::Value)
        -> Result<T, ProviderError>
    {
        // POST JSON {"query": query, "variables": variables}
        // Parse data field from response
    }
}

// GraphQL queries to implement:
// searchAnime(search: {query: "..."}) -> anime list
// anime(id: "...") -> single anime with episodes
// episode(showId: "...", translationType: "sub"|"dub", episodeString: "1")
//   -> episode with streamingInfo
```

**Add to `ProviderRegistry::new()`:**
```rust
anime_chain: vec![
    Arc::new(HiAnimeProvider::new()),
    Arc::new(AllAnimeProvider::new()),   // NEW
],
```

---

### M2.3 — AniSkip Integration

**New file:** `src/providers/anime/aniskip.rs`

```rust
pub async fn fetch_timestamps(mal_id: u32, episode: u32) -> Option<AniSkipTimestamps>
// GET https://api.aniskip.com/v1/skip-times/{mal_id}/{episode}?types[]=op&types[]=ed

pub struct AniSkipTimestamps {
    pub op_start: Option<f64>,
    pub op_end: Option<f64>,
    pub ed_start: Option<f64>,
    pub ed_end: Option<f64>,
}
```

Wire to mpv: pass `--script-opts-append=skip_op_start={op_start},skip_op_end={op_end}` args when launching mpv (if timestamps available).

---

### M2.4 — Anime TUI Screen

**New file:** `src/tui/screens/anime.rs`

Layout:
```
┌─────────────────────────────────────────────────────────────────┐
│ Tab Bar: [Movies] [ANIME] [Sports] [F1] [Live TV] ...          │
├─────────────────┬───────────────────────────────────────────────┤
│  Search Results │  Episode List                                 │
│  ─────────────  │  ──────────────────────────────────           │
│  > Naruto       │  S1E01 The Enter! Naruto Uzumaki!             │
│    Bleach       │  S1E02 My Name is Konohamaru!                 │
│    One Piece    │  S1E03 ...                                    │
│                 │                                               │
│  [Sub] [Dub]    │  Rating: 8.1/10  AniList: Current            │
│                 │  Next Ep: Tomorrow 02:00                      │
└─────────────────┴───────────────────────────────────────────────┘
│ / Search  d Download  f Favorite  t Sub/Dub  a AniList Sync   │
└─────────────────────────────────────────────────────────────────┘
```

**New app module:** `src/tui/app/anime.rs`
- `handle_anime_search()` — calls registry.search with MediaType::Anime
- `handle_sub_dub_toggle()` — toggles AnimeTabState.is_dub
- `handle_anime_schedule_load()` — loads AniList airing schedule

**Verify:** Anime tab renders, search returns results, sub/dub toggle works.

---

## M3 — Sports (Week 5)

**Goal:** Three-column live sports browser backed by streamed.pk API.

### M3.1 — Streamed.pk Provider

**New file:** `src/providers/sports/streamed.rs`

```rust
pub struct StreamedProvider { client: Arc<reqwest::Client> }

impl StreamedProvider {
    const BASE: &'static str = "https://streamed.su";
}

// Implement:
// GET /api/matches/live     -> Vec<LiveMatch>
// GET /api/matches/{sport}  -> Vec<LiveMatch> for specific sport
// GET /api/stream/{sport}/{match_id} -> Vec<MatchStream>
```

**New model file:** `src/providers/sports/models.rs`
```rust
pub struct LiveMatch { pub id, title, category, teams, competition, starts_at, is_popular, streams }
pub struct MatchStream { pub id, hd_url, sd_url, embed_url, language }
```

### M3.2 — Sports TUI Screen

**New file:** `src/tui/screens/sports.rs`

Three-column layout:
```
┌─────────────┬──────────────────┬────────────────────────────────┐
│ Sports      │ Matches          │ Streams                        │
│ ──────────  │ ──────────────   │ ──────────────────────         │
│ > Football  │ > Man U vs City  │ > HD English Commentary        │
│   Cricket   │   Chelsea vs ARS │   HD Spanish Commentary        │
│   Basketball│   Liverpool vs…  │   SD English                   │
│   F1        │                  │                                │
│   Tennis    │   [LIVE] 45'     │   720p  ●────────              │
│   Boxing    │   [UP] in 2h 15m │                                │
└─────────────┴──────────────────┴────────────────────────────────┘
│ h/l Navigate  Enter Play  ? Help                               │
└─────────────────────────────────────────────────────────────────┘
```

**Live indicator:** Red pulsing `●` indicator for live matches.
**Countdown:** Green `[UP] in {time}` for upcoming.

---

## M4 — F1 (Week 6)

**Goal:** F1 season calendar with session countdown and stream resolution.

### M4.1 — F1 Calendar Provider

**New file:** `src/providers/f1/calendar.rs`

The F1 season calendar comes from the Ergast F1 API (free, no auth):
```
GET https://ergast.com/api/f1/2026.json -> season schedule
```

Parse into `Vec<F1Session>` with all session timestamps.
Cache in SQLite `f1_calendar` table (24h TTL).

**New file:** `src/providers/f1/streams.rs`

Stream resolution for F1 uses iptv-org F1 channels:
```
GET https://iptv-org.github.io/iptv/categories/sports.m3u
Filter for: Sky Sports F1, F1 TV, etc.
```

**New file:** `src/providers/f1/iptv_f1.rs` — same pattern as `src/providers/tv/`

### M4.2 — F1 TUI Screen

**New file:** `src/tui/screens/f1.rs`

```
┌─────────────────────────────────────────────────────────────────┐
│ 🏎 F1 2026 Season Calendar                                     │
├─────────────────────────────────────────────────────────────────┤
│ NEXT: 🏁 Japanese Grand Prix — Race                            │
│       Suzuka Circuit, Japan                                     │
│       ⏰ Starts in: 02d 14h 23m 07s   [WATCH LIVE]            │
├────────────┬────────────────────────────────────────────────────┤
│ Round │ Event                    │ Date        │ Status         │
│ ──────┼──────────────────────────┼─────────────┼─────────────── │
│  R01  │ Bahrain Grand Prix       │ Mar 16-18   │ ✓ Finished    │
│  R02  │ Saudi Arabian GP         │ Mar 22-24   │ ✓ Finished    │
│  R03  │ Australian GP            │ Apr 13-15   │ ✓ Finished    │
│  R04  │ Japanese GP              │ Apr 27-29   │ ⏳ NEXT       │
│  R05  │ Chinese GP               │ May 04-06   │   Upcoming    │
└────────┴──────────────────────────┴─────────────┴───────────────┘
│ Enter: View Sessions  w: Watch Live  r: Refresh  ? Help       │
└─────────────────────────────────────────────────────────────────┘
```

**New widget:** `src/tui/widgets/countdown.rs` — as defined in TDD.md §7.2

---

## M5 — Tracking (Week 7)

**Goal:** mpv resume position, watch history, AniList sync, Trakt.tv sync.

### M5.1 — mpv IPC Resume

**Refactor:** `src/player/mpv.rs` from `src/player.rs`

Add IPC socket support (TDD.md §6.1):
- Launch mpv with `--input-ipc-server={path}`
- On exit: read `time-pos` via socket JSON protocol
- Save to SQLite: `db::history::upsert()`

### M5.2 — AniList OAuth

**New file:** `src/tracking/anilist_sync.rs`

OAuth flow: implicit grant (no server needed)
1. Print URL to terminal: `https://anilist.co/api/v2/oauth/authorize?client_id=...&response_type=token`
2. User opens URL in browser, copies token from redirect URL
3. User pastes token into terminal prompt
4. Store in keyring/SQLite

Then implement sync mutations (TDD.md §3.3).

### M5.3 — Trakt.tv OAuth

**New file:** `src/tracking/trakt.rs`

OAuth flow: device code flow (no browser redirect needed)
1. POST `/oauth/device/code`
2. Print user_code + verification_url in terminal
3. Poll `/oauth/device/token` until approved
4. Store tokens

Implement: scrobble start, scrobble stop (saves watch progress).

---

## M6 — Metadata (Week 8)

**Goal:** TMDB enrichment for movies/TV, AniList for anime.

### M6.1 — TMDB Client

**New file:** `src/metadata/tmdb.rs`

```rust
impl TmdbClient {
    // If api_key is None, all methods return Ok(None)
    // This ensures graceful degradation when no key configured
    
    pub async fn enrich_movie(&self, media: &mut Media) -> Result<()>
    pub async fn enrich_tv(&self, media: &mut Media) -> Result<()>
    pub async fn season_episodes(&self, tmdb_id: u32, season: u32) -> Result<Vec<EpisodeRef>>
    pub async fn recommendations(&self, tmdb_id: u32, kind: MediaType) -> Result<Vec<Media>>
}
```

**Enrichment pipeline:** When a search result is selected, spawn metadata task → update AppState with enriched metadata via Action.

### M6.2 — AniList GraphQL Metadata

**New file:** `src/metadata/anilist.rs`

```rust
impl AniListMetadata {
    pub async fn search(&self, query: &str) -> Result<Vec<AniListMedia>>
    pub async fn details(&self, id: u32) -> Result<AniListMedia>
    pub async fn airing_schedule(&self, season: AniListSeason, year: i32) -> Result<Vec<AiringAnime>>
    pub async fn user_list(&self) -> Result<Vec<AniListEntry>>  // requires auth
}
```

---

## M7 — Polish (Weeks 9–10)

**Goal:** Discord RPC, OS notifications, all 6 themes complete, final settings.

### M7.1 — Discord Rich Presence

**New file:** `src/tracking/discord_rpc.rs`

```rust
pub struct DiscordRpc { client: DiscordPresenceClient }

impl DiscordRpc {
    const APP_ID: &'static str = "SLOTH_DISCORD_APP_ID";

    pub fn set_watching(&mut self, media: &Media, episode: Option<&EpisodeRef>)
    pub fn set_idle(&mut self)
    pub fn clear(&mut self)
}

// Wire to PlaybackStarted/PlaybackEnded actions
```

### M7.2 — Notification Daemon

**New file:** `src/daemon/notifier.rs`

```rust
pub async fn run_notifier(pool: SqlitePool, config: NotificationsConfig) {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await; // check every 5 min
        
        // Check F1: is any session starting within 15 min?
        // Check anime: any new episodes in airing schedule?
        // Send OS notification if not already sent (check notifications_sent table)
    }
}
```

Spawn as `tokio::spawn(notifier::run_notifier(...))` in main.rs.

### M7.3 — Settings Screen Polish

**Update** `src/tui/widgets/settings.rs`:
Add new sections:
- **Accounts** — AniList login status + login button, Trakt status + login button
- **Notifications** — toggle, lead time for F1/match alerts
- **Discord** — toggle Rich Presence
- **Providers** — enable/disable individual providers, add custom Stremio addon URL
- **Quality** — default preferred quality

---

## M8 — Release (Weeks 11–12)

**Goal:** All tests green, cross-platform CI, crates.io publish, documentation.

### M8.1 — Final Test Pass

```bash
cargo nextest run --all
cargo clippy -- -D warnings
cargo audit
cargo fmt --all --check
```

Fix all failures. Update snapshot tests with `cargo insta review`.

### M8.2 — CI/CD Setup

Create `.github/workflows/ci.yml` and `.github/workflows/release.yml` as defined in WORKFLOW.md §5.

### M8.3 — User Documentation

Create `docs/user/`:
- `installation.md` — all platforms
- `quickstart.md` — first 5 minutes
- `configuration.md` — all config options
- `providers.md` — all content sources
- `keybindings.md` — complete keybinding reference
- `troubleshooting.md` — common issues

### M8.4 — README

Write compelling `README.md` with:
- Animated GIF/screenshot of all 5 content types
- One-line install instructions per platform
- Feature list
- Links to docs

---

## Implementation Priority Rules

1. **Always make it compile first** — stub out implementations with `todo!()` if needed
2. **Always make existing tests pass** — never break MovieBox-Tui functionality
3. **Provider before TUI** — providers first, then wire to UI
4. **Database before tracking** — SQLite schema before sync logic
5. **Feature flagged** — Discord and notifications behind feature flags from day 1
