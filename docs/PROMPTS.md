# Sloth — Antigravity Prompts

> **Version:** 1.0.0  
> **Last Updated:** 2026-09-18  
> **Usage:** Copy each prompt verbatim into the Antigravity chat, in order.  
> **Home directory:** `d:/Coding/Antigravity/Sloth`  

---

## ⚠️ CRITICAL IDENTITY & CONTRIBUTIONS RULE — READ BEFORE RUNNING ANY PROMPT

> **GitHub username: `syedzahidsaleem` — this is the ONLY permitted username.**  
> **Commit Author Email: `syedzahidsaleem2@gmail.com` — verified GitHub email.**
>
> - Repository: `https://github.com/syedzahidsaleem/sloth-tui.git`
> - **NEVER use `syedzahid0307`** — not in remotes, not in CI YAMLs, not in links, not anywhere.
> - **Ensure git author is set correctly** so contributions register on your GitHub profile:
>   `git config user.name "syedzahidsaleem"`
>   `git config user.email "syedzahidsaleem2@gmail.com"`
> - Before every `git push`, run `git remote -v` and confirm the URL shows `syedzahidsaleem`.
> - If it shows any other username, run: `git remote set-url origin https://github.com/syedzahidsaleem/sloth-tui.git`

## ⚠️ PUSH AFTER EVERY FILE — READ BEFORE RUNNING ANY PROMPT

> **After every single file that is created or modified**, run:
> ```bash
> git add <that file>
> git commit -m "type(scope): what changed in this file"
> git push origin main
> ```
> One file = one commit = one push. Do not accumulate. Do not batch. Push immediately. This ensures your commit count and contribution graph increase significantly with every single step!

---

## How to Use This Document

Run these prompts **one at a time** in Antigravity, in the numbered order. Each prompt builds on the previous. Do not skip ahead. Wait for Antigravity to complete and verify before proceeding to the next.

After each prompt, verify:
1. `cargo check` passes
2. No new compilation errors
3. Test suite still passes: `cargo nextest run --all` (or `cargo test --all`)
4. `git log --oneline -5` — every file written in this prompt has its own commit
5. `git remote -v` — remote URL is `https://github.com/syedzahidsaleem/sloth-tui.git`

---

## PROMPT 1 — Project Scaffold

```
I am building a Rust TUI application called "sloth-tui". The home directory is d:/Coding/Antigravity/Sloth.

Please complete the following setup tasks:

1. Create `d:/Coding/Antigravity/Sloth/Cargo.toml` with this exact content:

[package]
name = "sloth-tui"
version = "0.1.0"
edition = "2024"
rust-version = "1.90.0"
description = "Terminal interface for movies, anime, sports, F1, and live TV — zero cost, always fast."
license = "MIT OR Apache-2.0"

[dependencies]
ratatui = "0.30.0"
crossterm = { version = "0.29.0", features = ["event-stream"] }
ratatui-image = { version = "11.0.0", default-features = false, features = ["crossterm"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "fs", "process", "net", "io-util", "sync"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "gzip", "cookies", "stream"] }
futures = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
lru = "0.12"
base64 = "0.22"
hmac = "0.12"
md-5 = "0.10"
rmp-serde = "1"
dirs = "5"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
url = "2"
regex = "1"
once_cell = "1"
parking_lot = "0.12"
async-trait = "0.1"
scraper = "0.20"
fastrand = "2"

[target.'cfg(not(target_os = "android"))'.dependencies]
mimalloc = { version = "0.1", default-features = false }

[dev-dependencies]
tokio = { version = "1", features = ["test-util", "macros"] }
tempfile = "3"
pretty_assertions = "1"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "debuginfo"
panic = "abort"

[profile.dev]
opt-level = 1

2. Create `d:/Coding/Antigravity/Sloth/.cargo/config.toml`:

[build]
rustflags = []

3. Create `d:/Coding/Antigravity/Sloth/src/lib.rs` with:
- Module declarations for: config, models, net, logging, service, providers, metadata, tracking, player, db, daemon, tui
- A top-level SlothError enum using thiserror with variants: NoProvidersAvailable, Database, Config, Io, Provider

4. Create `d:/Coding/Antigravity/Sloth/src/main.rs` with:
- A basic tokio main function
- mimalloc global allocator (cfg non-android)
- TerminalGuard struct with Drop that restores terminal state using crossterm
- A call to tracing_subscriber::fmt::init() for logging
- A placeholder that prints "Sloth TUI starting..." and exits

5. Create directory structure:
src/providers/
src/providers/anime/
src/providers/sports/
src/providers/f1/
src/metadata/
src/tracking/
src/player/
src/db/
src/daemon/
src/tui/screens/
src/tui/widgets/
src/tui/app/
migrations/

6. Initialize git and create the GitHub repository:

   a) Initialize git repo (if not already done):
      git init -b main
      git config user.name "syedzahidsaleem"

   b) Create the GitHub repository using the GitHub CLI (gh):
      gh repo create syedzahidsaleem/sloth-tui --public --description "Zero-cost terminal streaming for movies, anime, sports, F1, and live TV" --clone=false

   c) Set the remote:
      git remote add origin https://github.com/syedzahidsaleem/sloth-tui.git

   d) Verify the remote — THIS IS MANDATORY:
      git remote -v
      # Must show: origin  https://github.com/syedzahidsaleem/sloth-tui.git
      # If it shows syedzahid0307 or ANY other name, correct it before continuing.

7. After writing EACH file above (Cargo.toml, .cargo/config.toml, src/lib.rs, src/main.rs, and each directory's placeholder mod.rs), immediately run:
      git add <that specific file>
      git commit -m "chore(scaffold): add <filename>"
      git push origin main

   This means there will be multiple commits for this single prompt. That is correct and expected.

8. Run `cargo check` and fix any compilation errors. Each fix-commit is also pushed immediately.

All rules from d:/Coding/Antigravity/Sloth/docs/RULES.md apply — especially R19 (push after every file) and R20 (syedzahidsaleem only, never syedzahid0307).
```

---

## PROMPT 2 — Provider Trait & Registry

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the Provider trait system.

Read these files first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (sections 2.1, 2.2, 2.3)
- d:/Coding/Antigravity/Sloth/docs/RULES.md

Then implement:

1. `src/providers/models.rs` — Define these exact types:
   - MediaType enum: Movie, Series, Anime, LiveSport, F1, IptvChannel
   - Quality enum: UHD4K, FHD1080, HD720, SD480, SD360, Auto, Unknown (with label() and quality_rank() -> u8 methods)
   - EpisodeRef struct: season, episode, title (Option<String>), duration_secs (Option<f64>)
   - StreamUrl struct: url (String), quality (Quality), is_hls (bool), headers (Vec<(String,String)>), subtitle_url (Option<String>), provider_id (&'static str)
   - Media struct: id (String), provider_id (&'static str), kind (MediaType), title (String), year (Option<u32>), poster_url (Option<String>), rating (Option<f32>), description (Option<String>), genres (Vec<String>), total_episodes (Option<u32>), total_seasons (Option<u32>), external_ids (ExternalIds)
   - ExternalIds struct: tmdb (Option<u32>), anilist (Option<u32>), mal (Option<u32>), trakt (Option<String>), imdb (Option<String>)
   - MediaMetadata struct: poster_url, backdrop_url, rating, vote_count, description, genres, cast (Vec<CastMember>), trailer_url, recommendations (Vec<Media>), episodes (Option<Vec<EpisodeRef>>)
   - AiringSchedule struct: status (AiringStatus enum), next_episode_at (Option<chrono::DateTime<chrono::Utc>>), season (Option<String>)
   - AiringStatus enum: Airing, Finished, NotYetAired, Cancelled, Hiatus
   - ProviderError enum with thiserror: Network(reqwest::Error), RateLimited, NotFound, Parsing(String), Unavailable, AuthRequired — with user_message() -> &'static str and is_retryable() -> bool methods

2. `src/providers/mod.rs` — Define:
   - ProviderCapabilities struct with bool fields: search, movies, tv_series, anime, live_sports, iptv, f1, subtitles, download, quality_selection, dub_sub_toggle
   - Provider trait (async_trait) with methods: id(), name(), capabilities(), search(), resolve(), episodes() (default: Err(NotFound)), metadata() (default: None), health() (default: true)
   - pub mod models; pub mod registry;

3. `src/providers/registry.rs` — Implement ProviderRegistry:
   - Fields: movie_chain, anime_chain, sports_chain, f1_chain, tv_chain (all Vec<Arc<dyn Provider>>), health (Arc<RwLock<HashMap<&'static str, bool>>>)
   - new(config: &Config) constructor — leave all chains as empty vecs for now (we'll fill them in later prompts)
   - chain_for(&MediaType) -> &[Arc<dyn Provider>]
   - search(query: &str, kind: MediaType) -> Vec<Media> — parallel futures::join_all, merge, dedup by lowercase title
   - resolve(media: &Media, episode: Option<&EpisodeRef>) -> Result<Vec<StreamUrl>, SlothError> — sequential fallback, skip unhealthy
   - run_health_checks() — parallel health ping, update health map
   - is_healthy(&'static str) -> bool — read from health map, default true if not present

All code must follow RULES.md: no unwrap(), async-trait, thiserror errors, full /// doc comments on all public items.

After implementation, run `cargo check` and fix all errors.
```

---

## PROMPT 3 — Database Layer

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the database layer.

Read first:
- d:/Coding/Antigravity/Sloth/docs/SCHEMA.md (full document)
- d:/Coding/Antigravity/Sloth/docs/RULES.md (R7 — Database rules)

Tasks:

1. Create `migrations/0001_initial.sql` with the COMPLETE SQL from SCHEMA.md. This includes all 10 tables: media, watch_history, favorites, anilist_entries, trakt_entries, auth_tokens, f1_calendar, sports_events, notifications_sent, schema_migrations. Include all CREATE INDEX statements and the INSERT into schema_migrations.

2. Create `src/db/mod.rs`:
   - open(path: &Path) -> Result<SqlitePool, SlothError> — uses SqlitePoolOptions, SqliteConnectOptions with WAL journal mode, foreign_keys ON, create_if_missing true, runs sqlx::migrate!("./migrations")
   - pub mod history; pub mod favorites;
   - WatchEntry struct: media_id (String), season (u32), episode (u32), episode_title (Option<String>), resume_position (f64), duration (Option<f64>), completed (bool), source_provider (Option<String>), quality (Option<String>)
   - HistoryRow struct: all fields from watch_history table + media fields (title, kind, poster_url)

3. Create `src/db/history.rs`:
   - upsert(pool: &SqlitePool, entry: &WatchEntry) -> Result<(), SlothError> — INSERT OR REPLACE into watch_history, set completed=1 if resume_position/duration >= 0.85
   - get_resume_position(pool: &SqlitePool, media_id: &str, season: u32, episode: u32) -> Option<f64>
   - recent(pool: &SqlitePool, limit: u32) -> Result<Vec<HistoryRow>, SlothError> — JOIN with media table, ORDER BY watched_at DESC
   - continue_watching(pool: &SqlitePool) -> Result<Vec<HistoryRow>, SlothError> — WHERE completed=0 AND resume_position BETWEEN 5% and 85% of duration, ORDER BY watched_at DESC LIMIT 10
   - mark_completed(pool: &SqlitePool, media_id: &str, season: u32, episode: u32) -> Result<(), SlothError>

4. Create `src/db/favorites.rs`:
   - add(pool: &SqlitePool, media: &Media) -> Result<(), SlothError> — first upsert media into media table, then insert into favorites
   - remove(pool: &SqlitePool, media_id: &str) -> Result<(), SlothError>
   - is_favorite(pool: &SqlitePool, media_id: &str) -> Result<bool, SlothError>
   - list(pool: &SqlitePool) -> Result<Vec<Media>, SlothError> — JOIN favorites with media

5. Update `src/lib.rs` to add Database variant to SlothError:
   Database(#[from] sqlx::Error)

6. Set DATABASE_URL environment variable for sqlx compile-time checking:
   Create `.env` file with: DATABASE_URL=sqlite://sloth.db
   Then run: sqlx database create && sqlx migrate run

7. Run `cargo check` and fix all errors.

Note: Use sqlx::query_as! and sqlx::query! macros throughout — no raw string queries.
All functions must be async and return proper Result types.
```

---

## PROMPT 4 — Import MovieBox-Tui Source

```
In the Sloth project at d:/Coding/Antigravity/Sloth, we need to import source code from the MovieBox-Tui repository.

The MovieBox-Tui repo is at: https://github.com/mesamirh/MovieBox-Tui

Do the following:

1. Fetch and read these files from the MovieBox-Tui GitHub repository (use raw.githubusercontent.com):
   - src/config.rs
   - src/models.rs
   - src/net.rs
   - src/logging.rs
   - src/service.rs
   - src/cache.rs
   - src/download.rs
   - src/proxy.rs
   - src/providers/models.rs (their provider models)
   - src/providers/mod.rs (their existing provider structure)
   - src/providers/moviebox/mod.rs
   - src/providers/moviebox/client.rs
   - src/providers/moviebox/adapt.rs
   - src/providers/moviebox/crypto.rs
   - src/providers/moviebox/session.rs
   - src/providers/moviebox/title.rs
   - src/providers/fourkhdhub/mod.rs
   - src/providers/fourkhdhub/client.rs
   - src/providers/fourkhdhub/hubcloud.rs
   - src/providers/fourkhdhub/parser.rs
   - src/providers/addons/mod.rs
   - src/providers/addons/adapter.rs
   - src/providers/addons/aggregator.rs
   - src/providers/addons/client.rs
   - src/providers/addons/models.rs
   - src/providers/tv/mod.rs
   - src/providers/tv/models.rs
   - src/providers/tv/parser.rs

2. For each file, write it to the equivalent path in d:/Coding/Antigravity/Sloth/src/ (same relative path).

3. Adapt the code:
   - Change all `moviebox_tui::` namespace references to `sloth_tui::` or the appropriate local module path
   - Keep all logic, comments, and structure identical — only change the crate name references
   - The MovieBox-Tui provider implementations (moviebox, fourkhdhub, addons, bdix, tv) must be adapted to implement our new `Provider` trait from src/providers/mod.rs
   - Each provider must implement: id(), name(), capabilities(), search(), resolve(), health()
   - Map their existing data structures to our Media, StreamUrl, EpisodeRef types

4. Also fetch and adapt the TUI core files:
   - src/tui/action.rs
   - src/tui/event.rs
   - src/tui/mod.rs
   - src/tui/terminal.rs
   - src/tui/text.rs
   - src/tui/theme.rs
   - src/tui/overlay.rs
   - src/tui/commands.rs
   - src/tui/state.rs
   - src/tui/widgets/badge.rs
   - src/tui/widgets/input.rs
   - src/tui/widgets/mod.rs
   - src/tui/widgets/modal.rs
   - src/tui/widgets/poster.rs
   - src/tui/widgets/scrollbar.rs
   - src/tui/widgets/settings.rs
   - src/tui/app/mod.rs
   - src/tui/app/run.rs
   - src/tui/app/keyboard.rs
   - src/tui/app/mouse.rs
   - src/tui/app/navigation.rs
   - src/tui/app/requests.rs
   - src/tui/app/playback.rs
   - src/tui/app/search.rs
   - src/tui/app/system.rs
   - src/tui/app/addons.rs
   - src/tui/app/download.rs
   - src/tui/app/favorites.rs
   - src/tui/app/tv.rs
   - src/tui/app/network.rs
   - src/tui/screens/home.rs
   - src/tui/screens/details.rs
   - src/tui/screens/help.rs
   - src/player/tracker.rs

5. After writing all files, update src/lib.rs and src/main.rs to wire everything together.

6. Run `cargo check`. Fix all compilation errors. The app should launch and show the basic MovieBox-Tui UI at this point.

This is a large task. Take it file by file. Read each source file, adapt it, write it, then move to the next. Report which files succeeded and any issues encountered.
```

---

## PROMPT 5 — Extended Tabs & New Theme System

```
In the Sloth project at d:/Coding/Antigravity/Sloth, extend the tab system and theme system.

Read first:
- d:/Coding/Antigravity/Sloth/docs/DESIGN.md (sections 2, 3)
- d:/Coding/Antigravity/Sloth/docs/TDD.md (sections 7.1, 8.1)

Tasks:

1. Update `src/tui/state.rs`:
   - Add Tab enum: Movies, Anime, Sports, F1, LiveTV, History, Favorites, Settings
   - Add AnimeTabState: results (Vec<Media>), selected_idx (usize), is_dub (bool), airing_schedule (Vec<AiringAnime>)
   - Add SportsTabState: sports (Vec<String>), selected_sport_idx (usize), matches (Vec<LiveMatch>), selected_match_idx (usize), streams (Vec<MatchStream>), selected_stream_idx (usize), focus (SportsColumnFocus enum: Sports/Matches/Streams)
   - Add F1TabState: calendar (Vec<F1Session>), selected_session_idx (usize), countdown (Option<Duration>), loading (bool)
   - Add these to AppState alongside existing state fields
   - Add active_tab: Tab field to AppState (default: Tab::Movies)

2. Update `src/tui/action.rs` to add new Action variants:
   - SwitchTab(Tab)
   - AnimeSubDubToggle
   - AnimeScheduleLoad
   - AnimeScheduleReceived(Vec<AiringAnime>)
   - SportSelected(String)
   - MatchListReceived(Vec<LiveMatch>)
   - MatchSelected(String)
   - StreamListReceived(Vec<MatchStream>)
   - F1CalendarLoad
   - F1CalendarReceived(Vec<F1Session>)
   - F1SessionSelected(F1Session)
   - SearchResultsReceived(Vec<Media>)
   - ResumePosition(f64)

3. Update `src/tui/theme.rs`:
   - Add new fields to Theme struct: bg_elevated, bg_surface, text_dim, text_muted, tab_active, tab_inactive (all Color)
   - Update existing themes to include these fields
   - Add these 5 new complete theme constants using exact color values from DESIGN.md:
     * CATPPUCCIN_MOCHA (make this the default, exact hex values from DESIGN.md section 2.3)
     * TOKYO_NIGHT
     * NORD
     * DRACULA  
     * GRUVBOX_DARK
     * ROSE_PINE
   - Update theme_by_name() to include all 6
   - Update ALL existing render code that references Theme to use the new fields

4. Update `src/tui/screens/home.rs`:
   - Replace the existing single-mode home screen with an 8-tab tab bar at the top
   - Tab bar shows: Movies | Anime | Sports | F1 | Live TV | History | Favorites | Settings
   - Active tab highlighted with theme.tab_active color, inactive with theme.tab_inactive
   - Tab switching via number keys 1-8 and Tab key
   - Movies tab: render existing home screen content
   - Anime, Sports, F1 tabs: render a placeholder "Coming soon" panel (we'll implement them in later prompts)
   - Live TV tab: render existing TV mode content
   - History, Favorites, Settings: route to existing screens

5. Add new keyboard handlers in `src/tui/app/keyboard.rs`:
   - Keys 1-8: SwitchTab action
   - Tab key: cycle through tabs
   - In sports tab: h/l switches between Sports/Matches/Streams columns

6. Run `cargo check`. The app must compile and the tab bar must appear. Movies tab must still work.
Verify: `cargo build && cargo run` — all 8 tabs visible, Movies tab works as before.
```

---

## PROMPT 6 — HiAnime Anime Provider

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the HiAnime anime provider.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 3.1 — HiAnime Provider Spec)
- d:/Coding/Antigravity/Sloth/docs/RULES.md (R3, R6)

HiAnime (hianime.to) is an anime streaming site with a public JSON API.

Implement `src/providers/anime/hianime.rs`:

The provider uses these API endpoints:
1. Search: GET https://hianime.to/api/v2/hianime/search?q={query}&page=1
   Response: { "data": { "animes": [{ "id": "...", "name": "...", "poster": "...", "rating": "...", "type": "...", "episodes": { "sub": N, "dub": N } }] } }

2. Episode list: GET https://hianime.to/api/v2/hianime/episodes/{anime-id}
   Response: { "data": { "episodes": [{ "episodeId": "...", "number": N, "title": "..." }] } }

3. Episode sources: GET https://hianime.to/api/v2/hianime/episode/sources?animeEpisodeId={id}&server=hd-1&category={sub|dub}
   Response: { "data": { "sources": [{ "url": "...", "type": "hls" }], "tracks": [{ "file": "...", "kind": "captions", "label": "English" }] } }

Implement:
1. HiAnimeProvider struct with client: Arc<reqwest::Client>
2. impl Provider for HiAnimeProvider:
   - id(): "hianime"
   - name(): "HiAnime"
   - capabilities(): search=true, anime=true, dub_sub_toggle=true, subtitles=true
   - search(): call search endpoint, parse animes array into Vec<Media> with kind=Anime
   - resolve(): 
     a) If episode is None, get episode list and use episode 1
     b) Call sources endpoint with category based on is_dub flag (store is_dub in provider or pass via Media extra field)
     c) Return StreamUrl with Referer header set to "https://hianime.to"
   - episodes(): call episode list endpoint, return Vec<EpisodeRef>
   - health(): HEAD https://hianime.to — return true if 200

3. Serde structs for all API responses (private, in the same file)

4. Wire to ProviderRegistry: add HiAnimeProvider to anime_chain

5. Create test fixture files:
   - tests/fixtures/hianime/search_naruto.json — realistic mock search response
   - tests/fixtures/hianime/episodes.json — realistic mock episode list
   - tests/fixtures/hianime/sources_sub.json — realistic mock sources response

6. Create integration test `tests/anime_pipeline.rs`:
   - Test: search for "Naruto" returns results
   - Test: resolve episode 1 returns a StreamUrl
   - Test: health() returns true when server responds 200
   - Use hardcoded fixture JSON (no live network calls in tests)

Run `cargo check` and `cargo test anime_pipeline`. Fix all errors.

IMPORTANT: The provider must never panic on malformed responses — return ProviderError::Parsing with a descriptive message.
All HTTP headers must include User-Agent: "Sloth-TUI/0.1.0"
```

---

## PROMPT 7 — AllAnime Provider

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the AllAnime provider.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 3.1 — anime providers)
- d:/Coding/Antigravity/Sloth/docs/RULES.md

AllAnime uses a GraphQL API at https://api.allanime.day/api

The GraphQL API accepts POST requests with {"query": "...", "variables": {...}}

Key GraphQL queries to implement:

1. Search query:
```graphql
query ($search: SearchInput, $limit: Int, $page: Int) {
  shows(
    search: $search
    limit: $limit
    page: $page
    translationType: sub
  ) {
    edges {
      _id
      name
      englishName
      thumbnail
      score
      availableEpisodes { sub dub }
    }
  }
}
```
Variables: {"search": {"query": "Naruto"}, "limit": 20, "page": 1}

2. Episode sources query:
```graphql
query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
  episode(
    showId: $showId
    translationType: $translationType
    episodeString: $episodeString
  ) {
    sourceUrls
  }
}
```

Implement `src/providers/anime/allanime.rs`:
1. AllAnimeProvider struct with client: Arc<reqwest::Client>
2. Private gql<T: DeserializeOwned>() helper method for GraphQL POST requests
3. impl Provider for AllAnimeProvider:
   - id(): "allanime"
   - name(): "AllAnime"
   - capabilities(): search=true, anime=true, dub_sub_toggle=true
   - search(): run search query, map to Vec<Media>
   - resolve(): run episode query, extract stream URL from sourceUrls
     (sourceUrls may need decoding: they are sometimes base64 or have "clock" prefix — handle gracefully)
   - health(): GET https://api.allanime.day — return true if reachable

4. Add AllAnimeProvider to ProviderRegistry anime_chain (after HiAnimeProvider):
```rust
anime_chain: vec![
    Arc::new(HiAnimeProvider::new()),
    Arc::new(AllAnimeProvider::new()),
],
```

5. Add AllAnime fixture and extend tests/anime_pipeline.rs:
   - Test: fallback to AllAnime when HiAnime returns empty results
   - Test: AllAnime search returns results for "One Piece"

Run `cargo check` and `cargo test anime_pipeline`.
```

---

## PROMPT 8 — Anime TUI Screen

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the full Anime tab UI screen.

Read first:
- d:/Coding/Antigravity/Sloth/docs/DESIGN.md (section 3.3 — Anime Tab Layout)
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 7.1 — AnimeTabState)
- d:/Coding/Antigravity/Sloth/docs/RULES.md (R8, R9)

Tasks:

1. Create `src/tui/screens/anime.rs`:
   Implement a two-panel layout:
   - Left panel (35% width): Search results list
     * Each item: title (bold), rating (dim star), Sub/Dub episode count badges
     * Selected item highlighted with theme.accent background
     * Scrollable with scrollbar
   - Right panel (65% width): Episode list OR detail view
     * If no anime selected: show "Search for anime above" placeholder
     * If anime selected but no episode: show metadata (title, rating, genres, description, AniList status)
     * If anime selected: show scrollable episode list
       - Each episode: "S{season}E{episode:02} — {title}"
       - Watched episodes: shown with theme.text_dim color + ✓ checkmark
       - Current resume episode: shown with ▶ prefix in theme.accent color
   - Bottom area (2 rows): [Sub] [Dub] toggle buttons + AniList sync status

2. Create `src/tui/app/anime.rs`:
   - handle_anime_search(state: &mut AppState, tx: &Sender<Action>, query: String)
     * Spawns tokio task that calls registry.search(query, MediaType::Anime)
     * Sends SearchResultsReceived(results) action
   - handle_sub_dub_toggle(state: &mut AppState)
     * Toggles state.anime_state.is_dub
     * If a media is selected and is currently playing, does nothing (just updates state)
   - handle_anime_episode_select(state: &mut AppState, tx: &Sender<Action>)
     * Gets selected media + episode
     * Spawns resolve task
     * On success: sends PlaybackStarted action

3. Wire the Anime tab in `src/tui/screens/home.rs`:
   - When active_tab == Tab::Anime, delegate rendering to anime::render()
   - Pass AnimeTabState and Theme to the anime render function

4. Wire keyboard in `src/tui/app/keyboard.rs`:
   - In Anime tab context:
     * t: AnimeSubDubToggle action
     * Enter: select anime or play episode (context-aware)
     * j/k: navigate results list (left panel focus) or episode list (right panel focus)
     * l: move focus from results to episode list
     * h: move focus back to results

5. Connect search: when user types in search bar while Anime tab is active, 
   call handle_anime_search() instead of the movie search handler.

Run `cargo check`. The Anime tab must render correctly with placeholder content.
Test: `cargo run` — navigate to Anime tab, verify layout matches DESIGN.md section 3.3.
```

---

## PROMPT 9 — Sports Provider (streamed.pk)

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the live sports provider.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 4.1 — Streamed.pk Provider)
- d:/Coding/Antigravity/Sloth/docs/RULES.md

The streamed.su (formerly streamed.pk) API is a public JSON API for live sports streams.

Base URL: https://streamed.su

Implement `src/providers/sports/streamed.rs`:

1. Define models in `src/providers/sports/models.rs`:
```rust
pub struct LiveMatch {
    pub id: String,
    pub title: String,
    pub category: String,        // "football", "cricket", "basketball", etc.
    pub home_team: Option<String>,
    pub away_team: Option<String>,
    pub competition: Option<String>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_live: bool,
    pub is_popular: bool,
    pub poster_url: Option<String>,
    pub streams: Vec<MatchStream>,
}

pub struct MatchStream {
    pub id: String,
    pub hd_url: Option<String>,
    pub sd_url: Option<String>,
    pub embed_url: Option<String>,
    pub language: Option<String>,
    pub quality: Quality,
}
```

2. API endpoints to use:
   - GET /api/matches/live — returns array of live matches
   - GET /api/matches/all-sports — returns all matches (live + upcoming)
   - GET /api/stream/{category}/{match_id} — returns stream details for a match
   
   The match response looks like:
   { "id": "...", "title": "...", "category": "football", "teams": {"home": "...", "away": "..."}, 
     "popular": false, "sources": [{"id": "...", "streamNo": 1}] }

3. StreamedProvider struct implementing Provider trait:
   - id(): "streamed-pk"
   - name(): "Streamed"
   - capabilities(): live_sports=true
   - search(): filter matches by query string (case-insensitive title match)
   - resolve(): call /api/stream/{category}/{id}, extract best quality M3U8 URL
   - health(): GET /api/matches/live — true if returns valid JSON array

4. Additional public methods (not in trait) for sports-specific data:
   - fetch_live_matches() -> Result<Vec<LiveMatch>>
   - fetch_matches_by_sport(sport: &str) -> Result<Vec<LiveMatch>>
   - fetch_streams(category: &str, match_id: &str) -> Result<Vec<MatchStream>>

5. Add to ProviderRegistry sports_chain

6. Create `tests/sports_pipeline.rs`:
   - Test: fetch_live_matches returns populated list
   - Test: resolve a match returns StreamUrl
   - Use fixture JSON files in tests/fixtures/sports/

Run `cargo check` and `cargo test sports_pipeline`.

Important: Stream URLs from streamed.su often require a Referer header. Always set:
headers: vec![("Referer".into(), "https://streamed.su".into())]
```

---

## PROMPT 10 — Sports TUI Screen

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the Sports tab three-column UI.

Read first:
- d:/Coding/Antigravity/Sloth/docs/DESIGN.md (section 3.4 — Sports Tab Layout)
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 7.1 — SportsTabState)
- d:/Coding/Antigravity/Sloth/docs/RULES.md (R8, R9)

Tasks:

1. Create `src/tui/screens/sports.rs`:
   Three-column layout:
   - Left (20%): Sports category list
     Sports categories (hardcoded list to start): Football, Cricket, Basketball, Tennis, Boxing, Baseball, F1, Darts, MMA
     Selected category highlighted
   - Middle (45%): Match list filtered by selected sport
     Each match shows:
     - "● {Home} vs {Away}" (red ● if live, ⏰ if upcoming)
     - Competition name in dim text
     - If live: "LIVE [min]'" or "LIVE"
     - If upcoming: time until start "in Xh Ym"
   - Right (35%): Stream list for selected match
     Each stream: quality badge + language
     "HD English" "SD Spanish" etc.
     Selected stream shows quality bar

   Focus indicator: left/middle/right panel border color = theme.border_focus when focused, theme.border otherwise.

2. Create `src/tui/app/sports.rs`:
   - handle_sport_selected(state, tx, sport_name): loads matches for sport, sends MatchListReceived
   - handle_match_selected(state, tx, match_id): loads streams, sends StreamListReceived  
   - handle_stream_play(state, tx): resolves selected stream and sends PlaybackStarted
   - refresh_live_data(state, tx): re-fetches current sport matches (called every 5 min via Tick)

3. Update column focus handling in `src/tui/app/keyboard.rs`:
   In Sports tab:
   - h: move focus left (Streams -> Matches -> Sports)
   - l: move focus right (Sports -> Matches -> Streams)
   - j/k: scroll within focused column
   - Enter: confirm selection in focused column
   - r: refresh live data

4. Wire Sports tab in home.rs

5. Wire SportsTabState updates to Action channel

Run `cargo check`. Test that the three-column layout renders and column focus switching works.
```

---

## PROMPT 11 — F1 Calendar Provider & Screen

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the Formula 1 feature.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 4.2 — F1 Calendar)
- d:/Coding/Antigravity/Sloth/docs/DESIGN.md (section 3.5 — F1 Tab Layout)

Tasks:

1. Create `src/providers/f1/calendar.rs`:

F1 calendar data from Ergast API (free, no auth):
GET http://ergast.com/api/f1/2026.json

Also try Jolpica API (Ergast successor):
GET https://api.jolpi.ca/ergast/f1/2026/

Response parsing:
- MRData.RaceTable.Races array
- Each race: season, round, raceName, Circuit.circuitName, Circuit.Location.country, Circuit.Location.locality
- FirstPractice, SecondPractice, ThirdPractice, Qualifying, Sprint, SprintQualifying, Race — all have date+time fields

Map to F1Session struct (from TDD.md section 4.2):
```rust
pub struct F1Session {
    pub round: u32,
    pub name: String,
    pub circuit: String,
    pub country: String,
    pub city: String,
    pub sessions: Vec<F1SessionSlot>,
}

pub struct F1SessionSlot {
    pub kind: F1SessionKind,
    pub starts_at: chrono::DateTime<chrono::Utc>,
}

pub enum F1SessionKind {
    FreePractice1, FreePractice2, FreePractice3,
    Qualifying, Sprint, SprintQualifying, Race,
}
```

Implement:
- fetch_calendar(season: u32) -> Result<Vec<F1Session>>
- Cache in SQLite f1_calendar table (24h TTL — check fetched_at before re-fetching)
- next_session(calendar: &[F1Session]) -> Option<(F1Session, F1SessionSlot)>
- time_until(slot: &F1SessionSlot) -> Duration

2. Create `src/providers/f1/streams.rs`:

F1 streams come from iptv-org sports category M3U:
GET https://iptv-org.github.io/iptv/categories/sports.m3u

Filter for channels containing "F1", "Formula", "Sky Sports F1"
Return their M3U8 URLs as StreamUrl objects

3. Create `src/tui/screens/f1.rs`:
Implement the layout from DESIGN.md section 3.5:
- Top section: NEXT SESSION box with event name, circuit, country
- Countdown widget: prominent display of time until next session
- Bottom: scrollable race calendar table with columns: Round | Event | Date | Status
  - Status: ✓ Complete (dim), ⏳ NEXT (accent), Upcoming (text)
  - NEXT race row is highlighted

4. Create `src/tui/widgets/countdown.rs`:
```rust
pub struct CountdownWidget {
    pub target: chrono::DateTime<chrono::Utc>,
    pub label: String,
}
```
Render as: "02d 14h 23m 07s" in large bold text
Update every second via Tick action

5. Create `src/tui/app/f1.rs`:
- handle_f1_calendar_load(state, tx): fetch calendar, save to DB, send F1CalendarReceived
- handle_f1_session_play(state, tx, session): resolve F1 stream, send PlaybackStarted
- update_countdown(state): calculate Duration until next session, update F1TabState.countdown

6. Wire F1 tab in home.rs, keyboard.rs

7. Add F1 test: `tests/f1_calendar.rs`
   - Test: calendar parses correctly from fixture JSON
   - Test: next_session returns correct next event

Run `cargo check` and `cargo test f1_calendar`.
```

---

## PROMPT 12 — mpv IPC & Resume Position

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement mpv IPC integration for resume position tracking.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 6.1 — mpv IPC Integration)
- d:/Coding/Antigravity/Sloth/docs/RULES.md

Tasks:

1. Create `src/player/mpv.rs`:

MpvPlayer struct:
```rust
pub struct MpvPlayer {
    child: tokio::process::Child,
    ipc_path: PathBuf,
}
```

Implement:
a) spawn(stream: &StreamUrl, resume_pos: Option<f64>, config: &PlayerConfig) -> Result<Self>:
   - Determine IPC socket path: /tmp/sloth-mpv-{uuid}.sock on Unix, \\.\pipe\sloth-mpv-{uuid} on Windows
   - Build mpv command with args:
     * stream URL
     * --no-terminal (mpv runs detached)
     * --input-ipc-server={ipc_path}
     * Custom headers via --http-header-fields={key}: {value}
     * --start={pos} if resume_pos is Some
     * --sub-file={url} if subtitle_url is Some
     * --demuxer-lavf-o=protocol_whitelist=... if is_hls
   - Spawn with tokio::process::Command
   - Wait 300ms for IPC socket to be created
   - Return MpvPlayer

b) get_position(&self) -> Option<f64>:
   - Connect to IPC socket (tokio net UnixStream or Windows named pipe)
   - Send JSON: {"command": ["get_property", "time-pos"]}\n
   - Read response JSON: {"data": 3847.2, "error": "success"}
   - Return data field if error == "success"
   - Timeout after 1 second
   - Return None on any error (mpv may have just closed)

c) wait_for_exit(&mut self) -> Option<f64>:
   - Try to get current position one final time
   - Then wait for child process to exit: self.child.wait().await
   - Return the captured position

d) ipc_socket_path() -> PathBuf (private):
   - #[cfg(unix)]: temp_dir().join(format!("sloth-mpv-{}.sock", uuid::Uuid::new_v4()))
   - #[cfg(windows)]: PathBuf::from(format!(r"\\.\pipe\sloth-mpv-{}", uuid::Uuid::new_v4()))

2. Update `src/tui/app/playback.rs`:
After PlaybackStarted action:
   - Spawn mpv via MpvPlayer::spawn()
   - Get resume position from DB: db::history::get_resume_position()
   - After wait_for_exit() completes, send PlaybackEnded { resume_position_secs: Option<f64> }

Handle PlaybackEnded action:
   - Call db::history::upsert() with the resume position
   - If tracking enabled: call trakt/anilist sync (stubs for now)

3. Update `src/player/mod.rs` to expose MpvPlayer and also a generic player launch function:
```rust
pub async fn launch_player(
    stream: &StreamUrl,
    resume_pos: Option<f64>,
    config: &PlayerConfig,
    db_pool: &SqlitePool,
    media_id: &str,
    season: u32,
    episode: u32,
) -> Result<(), SlothError>
```
This function handles the full playback lifecycle: launch -> wait -> save position.

4. Create `src/player/vlc.rs` and `src/player/iina.rs` as simpler alternatives:
   - VLC: spawn vlc with --play-and-exit, no IPC (no resume position support)
   - IINA (macOS only, #[cfg(target_os = "macos")]): spawn iina with --mpv-input-ipc-server (IINA supports mpv IPC)

5. Auto-detect player in `src/config.rs`:
   - Check PATH for: mpv, vlc, iina, celluloid
   - If preferred player from config is not found, fall back to first detected player
   - Show warning in status bar if no player found

Run `cargo check`. No live testing needed — mpv IPC is tested manually.
```

---

## PROMPT 13 — AniList OAuth & Sync

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement AniList OAuth and watch progress sync.

Read first:
- d:/Coding/Antigravity/Sloth/docs/TDD.md (section 3.3 — AniList OAuth)
- d:/Coding/Antigravity/Sloth/docs/RULES.md

AniList OAuth is an implicit grant flow — no client secret needed.
The user must create an AniList API client at https://anilist.co/settings/developer
and provide the client_id. The redirect_uri must be set to https://anilist.co/api/v2/oauth/pin

Tasks:

1. Create `src/tracking/anilist_sync.rs`:

AniListClient struct:
```rust
pub struct AniListClient {
    http: Arc<reqwest::Client>,
    access_token: Option<String>,
}
```

Implement:
a) login_url(client_id: &str) -> String:
   Returns: https://anilist.co/api/v2/oauth/authorize?client_id={id}&response_type=token

b) authenticate(pool: &SqlitePool) -> Result<Option<Self>>:
   - Check auth_tokens table for "anilist" entry that hasn't expired
   - If found, return AniListClient with stored token
   - If not found, return Ok(None) — caller will prompt user to login

c) start_auth_flow(pool: &SqlitePool, client_id: &str) -> Result<String>:
   - Print the login URL
   - Prompt user to paste their access token (from the URL fragment after redirect)
   - Store in auth_tokens table
   - Return the token

d) gql<T: DeserializeOwned>(&self, query: &str, variables: serde_json::Value) -> Result<T>:
   - POST https://graphql.anilist.co with Authorization: Bearer {token}
   - Parse { "data": {...} } response

e) mark_episode_watched(&self, anilist_id: u32, episode: u32) -> Result<()>:
   - Call SaveMediaListEntry mutation (from TDD.md)
   - Update anilist_entries table: set progress=episode, dirty=0

f) fetch_user_list(&self) -> Result<Vec<AniListEntry>>:
   - Query MediaListCollection for current user
   - Store results in anilist_entries table

g) sync_dirty_entries(&self, pool: &SqlitePool) -> Result<()>:
   - Query anilist_entries WHERE dirty=1
   - For each entry, call appropriate mutation
   - Set dirty=0 on success

2. Update `src/tracking/mod.rs`:
   pub mod anilist_sync;

3. Wire to PlaybackEnded action in playback.rs:
   - If media.kind == Anime && anilist client is available:
     * Mark episode as watched if completed (>85% watched)
     * Queue as dirty if sync fails (retry later)

4. Add Settings screen entry for AniList:
   In settings.rs Accounts section:
   - Show "AniList: Not logged in" with [Login] button if no token
   - Show "AniList: Logged in as {username}" if token present
   - Login button triggers auth flow prompt

Run `cargo check`. No live AniList testing in CI.
```

---

## PROMPT 14 — TMDB Metadata Enrichment

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement TMDB metadata enrichment.

TMDB API v3 documentation: https://developer.themoviedb.org/reference/intro/getting-started
Base URL: https://api.themoviedb.org/3
Image base: https://image.tmdb.org/t/p/w500

Tasks:

1. Create `src/metadata/tmdb.rs`:

TmdbClient struct:
```rust
pub struct TmdbClient {
    http: Arc<reqwest::Client>,
    api_key: Option<String>,   // if None, all methods return Ok(None)
}
```

Implement these methods:

a) search_movie(query: &str, year: Option<u32>) -> Result<Option<TmdbMovieResult>>
   GET /search/movie?query={query}&year={year}&api_key={key}
   Return the best match (first result)

b) movie_details(id: u32) -> Result<TmdbMovie>
   GET /movie/{id}?api_key={key}&append_to_response=credits,recommendations

c) tv_details(id: u32) -> Result<TmdbTv>
   GET /tv/{id}?api_key={key}&append_to_response=credits,recommendations

d) tv_season_episodes(id: u32, season: u32) -> Result<Vec<EpisodeRef>>
   GET /tv/{id}/season/{season}?api_key={key}

e) enrich_media(media: &mut Media) -> Result<()>:
   - If api_key is None: return Ok(()) immediately (graceful skip)
   - Based on media.kind:
     * Movie: search_movie(title, year) -> movie_details -> update media fields
     * Series: search for TV -> tv_details -> update media fields
   - Update: poster_url, rating, description, genres, external_ids.tmdb
   - Store enriched data in SQLite media table

f) poster_url(path: &str) -> String
   Returns: https://image.tmdb.org/t/p/w500{path}

2. Create response structs (private serde structs):
   TmdbSearchResponse, TmdbMovie, TmdbTv, TmdbSeason, TmdbCredits, TmdbRecommendations, TmdbCastMember

3. Create `src/metadata/mod.rs`:
   pub mod tmdb;
   pub mod anilist;  // stub for now

4. Wire metadata enrichment:
   In `src/tui/app/requests.rs`, when a media item is selected:
   - Spawn tokio task: TmdbClient::enrich_media(&mut media)
   - Send MetadataEnriched(media) action
   - Update state.selected_media with enriched data
   - Trigger re-render of details panel

5. Update details screen `src/tui/screens/details.rs`:
   - Show TMDB rating alongside provider rating
   - Show cast list (first 5 members)
   - Show genres as small badges
   - Show "Powered by TMDB" attribution in bottom corner (TMDB requires attribution)

Run `cargo check`.

Note: If no TMDB api_key is configured, the details panel still works — it just shows
whatever metadata the provider returned, without enrichment.
```

---

## PROMPT 15 — Discord Rich Presence

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement Discord Rich Presence.

The discord-presence crate implements the Discord IPC protocol in pure Rust.
Documentation: https://docs.rs/discord-presence

Tasks:

1. Add discord-presence to Cargo.toml under the discord feature:
```toml
[features]
default = ["notifications", "discord"]
discord = ["dep:discord-presence"]

[dependencies]
discord-presence = { version = "0.4", optional = true }
```

2. Create `src/tracking/discord_rpc.rs`:

```rust
#[cfg(feature = "discord")]
pub struct DiscordRpc {
    client: discord_presence::Client,
    enabled: bool,
}
```

Implement:
a) new(enabled: bool) -> Self — create Client with Sloth's Discord app ID
   (Use a placeholder app ID for now: "1234567890" — user will need to create one)

b) set_watching(&mut self, media: &Media, episode: Option<&EpisodeRef>):
   - State: "Watching"
   - Details: media.title
   - State line: if episode: "S{s}E{e} — {title}" else "Movie"
   - Large image: "sloth_logo" (or poster if API supports it)
   - Timestamps: set start = now

c) set_browsing(&mut self):
   - State: "Browsing"
   - Details: "Looking for something to watch"
   - Clear timestamps

d) clear(&mut self):
   - Clear all activity

e) Stub when feature disabled:
```rust
#[cfg(not(feature = "discord"))]
pub struct DiscordRpc;
#[cfg(not(feature = "discord"))]
impl DiscordRpc {
    pub fn new(_enabled: bool) -> Self { Self }
    pub fn set_watching(&mut self, _: &Media, _: Option<&EpisodeRef>) {}
    pub fn set_browsing(&mut self) {}
    pub fn clear(&mut self) {}
}
```

3. Wire to App:
   - Add DiscordRpc to App struct
   - On PlaybackStarted: call set_watching()
   - On PlaybackEnded: call set_browsing()
   - On app exit: call clear()

4. Add to settings screen:
   - "Discord Rich Presence: [On/Off]" toggle

Run `cargo check --features discord` and `cargo check --no-default-features`.
Both must compile cleanly.
```

---

## PROMPT 16 — Notification Daemon

```
In the Sloth project at d:/Coding/Antigravity/Sloth, implement the background notification daemon.

Read: d:/Coding/Antigravity/Sloth/docs/TDD.md (section 7.2 — daemon/notifier.rs)

The daemon runs as a background tokio task and sends OS notifications for:
1. F1 sessions starting within 15 minutes
2. New anime episodes available (from AniList airing schedule)

Tasks:

1. Add notify-rust to Cargo.toml under notifications feature:
```toml
[features]
notifications = ["dep:notify-rust"]

[dependencies]
notify-rust = { version = "4", optional = true, default-features = false, features = ["z_dbus"] }
```

2. Create `src/daemon/notifier.rs`:

```rust
pub async fn run_notifier(
    pool: SqlitePool,
    config: NotificationsConfig,
)
```

Implementation:
- Loop every 5 minutes (tokio::time::interval)
- Check F1 calendar from SQLite for sessions starting within 15 minutes
  * If found and not in notifications_sent: send notification + insert into notifications_sent
- Check AniList airing schedule (if configured) for new episodes
  * If found and not in notifications_sent: send notification + insert

Notification sending function:
```rust
fn send_notification(title: &str, body: &str) {
    #[cfg(feature = "notifications")]
    {
        // Linux/macOS
        #[cfg(not(target_os = "windows"))]
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .icon("media-playback-start")
            .show();
        
        // Windows: use notify_rust with winrt backend
        #[cfg(target_os = "windows")]
        // ... Windows toast notification
    }
    #[cfg(not(feature = "notifications"))]
    let _ = (title, body); // suppress unused warnings
}
```

3. Update `src/main.rs`:
   - Spawn notifier as a background task if notifications enabled in config:
```rust
if config.notifications.enabled {
    tokio::spawn(daemon::notifier::run_notifier(pool.clone(), config.notifications.clone()));
}
```

4. Add to settings screen:
   - "Notifications: [On/Off]" toggle
   - "F1 Alert Lead Time: [15 min ▼]" selector

Run `cargo check --features notifications` and `cargo check --no-default-features`.
```

---

## PROMPT 17 — Integration Tests & Polish

```
In the Sloth project at d:/Coding/Antigravity/Sloth, run the full test suite and fix all failures. Then apply final polish.

Tasks:

1. Run the complete test suite:
   cargo nextest run --all
   
   Fix every failure. Document what was fixed.

2. Run clippy:
   cargo clippy -- -D warnings
   
   Fix every warning.

3. Run fmt check:
   cargo fmt --all --check
   
   Apply formatting: cargo fmt --all

4. Run security audit:
   cargo audit
   
   Report any high-severity issues.

5. Final polish tasks:
   a) Add a startup banner in main.rs (displayed briefly before TUI starts):
      - ASCII art "SLOTH" 
      - Version number
      - "Loading providers..."
      - Delay 500ms then launch TUI
      (Only shown if terminal supports it — skip if --quiet flag)

   b) Add keyboard shortcut display in the bottom bar:
      - The hint bar at the bottom must be context-aware
      - Movies tab: "/ Search  j↓ k↑  Enter Select  f Fav  d DL  q Quit"
      - Anime tab: "/ Search  t Sub/Dub  a AniList  Enter Play  q Quit"
      - Sports tab: "h← l→ Navigate  Enter Play  r Refresh  q Quit"
      - F1 tab: "Enter Sessions  w Watch  r Refresh  q Quit"

   c) Verify all 6 themes work:
      - Switch through each theme with t key
      - All screens must render correctly in all themes
      - No hardcoded colors in any render function

   d) Error recovery: ensure that if any provider fails during startup,
      the app still launches and shows an appropriate status message.
      The app must never refuse to start due to a network error.

   e) Add "Continue Watching" section to the Movies tab home screen:
      - Query db::history::continue_watching()
      - Show as a horizontal row of cards above the main content
      - Each card: poster thumbnail + title + progress bar

6. Final verification:
   cargo build --release
   ./target/release/sloth-tui
   
   Verify:
   - [ ] App launches in < 200ms
   - [ ] All 8 tabs visible
   - [ ] Movies search works
   - [ ] Anime search works
   - [ ] Sports tab shows three columns
   - [ ] F1 tab shows calendar
   - [ ] All 6 themes work
   - [ ] Help screen (?) shows all keybindings
   - [ ] Settings screen shows all options
   - [ ] No panics during normal operation
```

---

## PROMPT 18 — Cross-Platform Verification & Release Build

```
In the Sloth project at d:/Coding/Antigravity/Sloth, perform final cross-platform verification and prepare release build.

Tasks:

1. Create `.github/workflows/ci.yml` with the content from WORKFLOW.md section 5 (CI Pipeline).

2. Create `.github/workflows/release.yml` with the content from WORKFLOW.md section 5 (Release Pipeline).

3. Update `Cargo.toml` with final metadata:
   - version: "0.1.0"
   - Add: homepage, repository, readme, categories, keywords

4. Create `README.md` with:
   - Project name "Sloth" with tagline
   - Feature list (movies, anime, sports, F1, live TV)
   - Installation instructions for all platforms:
     * Linux: curl install script or cargo install
     * macOS: brew or cargo install
     * Windows: winget or cargo install
     * Android: cargo install via Termux
   - Quick start guide (3 steps)
   - Configuration section
   - Keybindings table
   - Contributing section

5. Create `CHANGELOG.md` with v0.1.0 entry listing all features.

6. Verify cross-compilation (at minimum):
   cargo check --target x86_64-unknown-linux-musl
   cargo check --target x86_64-pc-windows-msvc
   
   Fix any platform-specific compilation errors.

7. Build release binary:
   cargo build --release
   
   Report:
   - Binary size (should be < 20MB)
   - Startup time measurement
   - Feature list in the binary (cargo metadata)

The project is now ready for v0.1.0 release.
```

---

## Notes for Running Prompts

### Order Matters
Prompts must be run in order 1-18. Each prompt assumes the previous ones completed successfully.

### Between Prompts
After each prompt:
1. Verify `cargo check` passes
2. Verify `cargo test --all` passes (or note which tests are new and pending fixtures)
3. Save your work: `git add -A && git commit -m "..."`

### If a Prompt Fails
If Antigravity encounters an error it cannot resolve:
1. Note the error message
2. Check if the relevant docs file addresses it (TDD.md, RULES.md, ARCHITECTURE.md)
3. Ask Antigravity: "The previous task had this error: {error}. Please fix it before continuing."

### Parallel Prompts
Prompts 6 and 7 (HiAnime + AllAnime) can be run in separate Antigravity sessions and merged.
Prompts 9 and 11 (Sports + F1) can also be parallelized.

### Verification Commands
```bash
# Quick check
cargo check

# Full test
cargo nextest run --all

# Clean build  
cargo clean && cargo build

# Release build check
cargo build --release
```
