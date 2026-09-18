# Sloth — Product Requirements Document (PRD)

> **Version:** 1.0.0  
> **Status:** Approved for Development  
> **Last Updated:** 2026-09-18  

---

## 1. Executive Summary

**Sloth** is a blazing-fast, zero-cost, cross-platform terminal user interface (TUI) for watching movies, TV shows, anime, sports, and live events — including Formula 1. It operates entirely through direct HTTP scraping and public APIs, with no debrid services, no subscriptions, and no runtime dependencies beyond an external video player.

The project forks and dramatically expands [MovieBox-Tui](https://github.com/mesamirh/MovieBox-Tui) (Rust, Ratatui), integrating content from multiple specialized open-source ecosystems into one unified, always-working terminal experience.

**Core Promise:** Type a title, press Enter, watch. No buffering pages, no login walls, no debrid accounts. If one source fails, the next kicks in automatically.

---

## 2. Problem Statement

Existing terminal streaming tools each cover only one content type or one source:

| Tool | What It Does | What It Misses |
|---|---|---|
| MovieBox-Tui | Movies/TV via MovieBox API | Anime, sports, F1, tracking |
| ani-cli | Anime via shell scripts | Movies, sports, F1, tracking |
| streamed-tui | Live sports via streamed.pk | Movies, anime, F1 calendar |
| f1tv scripts | F1 streams only | Everything else |

**The gap:** No single tool covers all content types with a unified, polished TUI, robust fallback chains, watch tracking, and a notification system — while remaining zero-cost and always available.

---

## 3. Goals

### Primary Goals
- **G1 – Universal Coverage:** One tool to watch movies, TV series, anime (sub/dub), live sports (football, cricket, basketball, F1), and IPTV channels.
- **G2 – Zero Cost:** No debrid services, no paid APIs, no subscriptions ever required.
- **G3 – Maximum Availability:** Multi-provider fallback chains so content is always reachable even when individual scrapers go down.
- **G4 – Blazing Speed:** Sub-100ms UI response, parallel provider resolution, LRU caching, instant startup.
- **G5 – Cross-Platform:** Identical experience on Windows, macOS, Linux, and Android (Termux).

### Secondary Goals
- **G6 – Watch Tracking:** AniList (anime) and Trakt.tv (movies/TV) OAuth sync, local SQLite history with per-episode resume timestamps.
- **G7 – Rich Metadata:** TMDB enrichment for movies/TV, AniList for anime — posters, ratings, cast, recommendations.
- **G8 – Notifications:** Background daemon for new episode alerts and race/match start notifications.
- **G9 – Discord Rich Presence:** Show what you're watching with timestamps.
- **G10 – Beautiful UI:** 6 built-in themes (Catppuccin Mocha, Tokyo Night, Nord, Dracula, Gruvbox, Rose Pine). Kitty/Ghostty poster images rendered inline.

### Non-Goals
- Debrid integration (Real-Debrid, AllDebrid, Premiumize) — explicitly out of scope
- Torrent/P2P — Phase 3 stretch goal only
- Web UI or Electron wrapper
- Paid API tiers (TMDB free tier only)
- Subtitle downloading from paid services

---

## 4. Target Users

| Persona | Description | Key Need |
|---|---|---|
| **Power Terminal User** | Developer/sysadmin, lives in the terminal | Keyboard-driven, scriptable, composable with mpv |
| **Anime Watcher** | Seasonal anime follower | AniList sync, dub/sub toggle, airing schedule |
| **Sports Fan** | Watches F1, football, cricket | Live event browser, match schedule, race countdown |
| **Privacy Conscious** | Avoids web UIs and browser history | Local history, no cloud account required |
| **Bandwidth Aware** | On metered/slow connections | Quality selection, resume download, stream before download |

---

## 5. Content Types & Feature Requirements

### 5.1 Movies & TV Shows

**Sources (priority order):**
1. MovieBox API (existing, keep as-is from MovieBox-Tui)
2. 4KHDHub (existing)
3. Stremio community addons (existing addon framework)

**Features:**
- Full-text search with instant results
- Rich detail view: poster, rating, cast, plot, genre, year, runtime
- Season/episode browser for series
- Quality selection (4K/1080p/720p/480p auto-downgrade)
- Subtitle auto-fetch (OpenSubtitles scrape, existing logic)
- Bookmark/favorites (existing)
- Resume from last position via mpv IPC socket
- Trakt.tv watched sync (new)
- Download with resume support (existing)

### 5.2 Anime

**Sources (priority order):**
1. HiAnime (AniWatch) — primary, best sub/dub coverage
2. AllAnime — best English dub coverage
3. Allanime via GraphQL fallback

**Features:**
- Sub/Dub toggle per title
- AniList OAuth integration: search, watch progress sync, score submission
- Airing schedule browser (current season, upcoming)
- MAL score + AniList score display
- Intro/outro skip (AniSkip API timestamp markers)
- Episode batch navigation
- Watch history with resume timestamps

### 5.3 Live Sports

**Sources:**
1. streamed.pk API — primary live sports (football, basketball, cricket, tennis, etc.)
2. iptv-org sport-category M3U bundles — channel fallback

**Features:**
- Three-column browser: Sport > Match > Stream
- Live/upcoming match schedule with time-to-start countdown
- Multiple stream quality options per match
- Channel list browser (IPTV mode fallback)
- Stream status indicator (live, upcoming, finished)

### 5.4 Formula 1

**Sources:**
1. Direct public F1 stream sources (sky sports HLS, public F1 streams)
2. iptv-org F1 channels (Sky Sports F1 M3U)

**Features:**
- Full 2026+ season calendar with race/qualifying/sprint schedule
- Time-to-event countdown in the UI
- Historical race listing
- Live race, qualifying, practice session streaming
- Multi-feed support (onboard cameras, team radio feed HLS)

### 5.5 Live TV / IPTV

**Sources:**
- iptv-org/iptv full category-tagged M3U bundles (existing M3U import + auto-update)

**Features:**
- Category browser (news, sports, movies, entertainment by country)
- Channel search
- EPG (Electronic Program Guide) where available
- Custom M3U URL import (existing functionality, keep)

---

## 6. Technical Requirements

### 6.1 Performance

| Metric | Target |
|---|---|
| TUI frame render time | < 16ms (60fps capable) |
| Search result first byte | < 800ms |
| Stream URL resolution | < 2s (parallel providers) |
| App startup | < 200ms |
| Memory usage (idle) | < 30MB RSS |
| Provider parallel requests | Up to 8 concurrent |

### 6.2 Reliability

- **Fallback chain:** Every content type has >= 2 independent sources.
- **Error display:** User-friendly messages, never raw panics, graceful recovery.
- **Provider health check:** Background ping every 10 min, disable failing providers.
- **Retry logic:** 3 attempts with exponential backoff (100ms -> 400ms -> 1600ms).
- **Graceful degradation:** If metadata fails, stream still works; if poster fails, text fallback.

### 6.3 Cross-Platform

| Platform | Support Level |
|---|---|
| Linux (x86_64, aarch64) | Full — all features |
| macOS (Intel + Apple Silicon) | Full — all features |
| Windows (x86_64) | Full — all features |
| Android (Termux, aarch64) | Full except poster images in non-kitty terms |

### 6.4 Compilation

- Rust edition 2024, MSRV 1.90.0
- Single static binary distribution (no runtime dependencies)
- `mimalloc` allocator (non-Android, as in upstream)
- LTO enabled in release profile
- `cargo install sloth-tui` target (crates.io)

---

## 7. User Interface Requirements

### 7.1 Navigation Model

```
Home (Tab Bar)
├── Movies & TV       [1]
├── Anime             [2]
├── Sports            [3]
├── F1                [4]
├── Live TV           [5]
├── History           [6]
├── Favorites         [7]
└── Settings          [8]

Search: / or s (global, context-aware to current tab)
Help: ?
Quit: q or Ctrl+C
```

### 7.2 Keyboard Conventions

| Key | Action |
|---|---|
| `j`/`k` or Down/Up | Navigate list |
| `h`/`l` or Left/Right | Navigate columns / tabs |
| `Enter` | Select / confirm |
| `/` or `s` | Open search |
| `f` | Toggle favorite |
| `d` | Download |
| `Esc` | Back / cancel |
| `q` | Quit |
| `Tab` | Switch tab |
| `?` | Help |
| `t` | Switch theme |
| `p` | Switch player |

### 7.3 Themes

All 6 themes must be selectable at runtime without restart:
1. Catppuccin Mocha (default)
2. Tokyo Night
3. Nord
4. Dracula
5. Gruvbox Dark
6. Rose Pine

### 7.4 Image Protocol Support

| Protocol | Terminal |
|---|---|
| Kitty Graphics Protocol | kitty, ghostty |
| Sixel | xterm, mlterm, foot |
| Unicode Half-Block Fallback | Any terminal |

---

## 8. Integration Requirements

### 8.1 External Players

Must support auto-detect and manual config for:
- `mpv` (primary, required for IPC/resume)
- `vlc`
- `iina` (macOS)
- `celluloid`
- `totem`
- `mplayer`
- Any custom player via config `player.custom_command`

### 8.2 API Integrations

| Service | Purpose | Auth Method |
|---|---|---|
| TMDB | Movie/TV metadata | Free API key (config) |
| AniList | Anime metadata + tracking | OAuth2 PKCE |
| Trakt.tv | Movie/TV tracking | OAuth2 |
| OpenSubtitles | Subtitle discovery | Public scrape |
| AniSkip | Anime intro/outro timestamps | Public API, no auth |
| streamed.pk | Live sports data | Public API, no auth |
| iptv-org | M3U playlists | Public CDN, no auth |

### 8.3 mpv IPC Integration

On playback start, Sloth opens an mpv IPC socket. On exit, it reads `time-pos` property and saves to SQLite `watch_history` as a resume timestamp. Next launch of the same episode resumes from that position.

---

## 9. Configuration

Config file location:
- Linux/macOS: `~/.config/sloth-tui/config.toml`
- Windows: `%APPDATA%\sloth-tui\config.toml`
- Android: `~/storage/shared/sloth-tui/config.toml`

Minimum required config (all optional, sensible defaults):

```toml
[player]
preferred = "mpv"          # auto-detected if not set

[theme]
name = "catppuccin-mocha"  # default theme

[providers.tmdb]
api_key = ""               # optional, enables rich metadata

[providers.anilist]
enabled = true             # AniList OAuth

[providers.trakt]
enabled = false            # Trakt.tv OAuth

[image]
protocol = "auto"          # auto | kitty | sixel | block

[search]
default_tab = "movies"     # which tab is focused at launch

[notifications]
enabled = true
```

---

## 10. Data Storage

All local data stored under:
- Linux/macOS: `~/.local/share/sloth-tui/`
- Windows: `%LOCALAPPDATA%\sloth-tui\`

| File | Purpose |
|---|---|
| `sloth.db` | SQLite: history, favorites, resume timestamps |
| `cache/posters/` | LRU image cache (max 500MB, configurable) |
| `cache/metadata/` | Provider response cache (TTL: 1h movies, 5min sports) |
| `config.toml` | User configuration |
| `sloth.log` | Debug log (rotate daily, keep 3) |

---

## 11. Release Milestones

| Milestone | Features | Target |
|---|---|---|
| **M1 — Foundation** | Fork + cleanup MovieBox-Tui, new tab layout, theme system | Week 1-2 |
| **M2 — Anime** | HiAnime + AllAnime providers, sub/dub toggle, AniList OAuth | Week 3-4 |
| **M3 — Sports** | streamed.pk provider, three-column sports UI | Week 5 |
| **M4 — F1** | F1 calendar screen, countdown, stream resolution | Week 6 |
| **M5 — Tracking** | mpv IPC resume, SQLite history, Trakt.tv sync | Week 7 |
| **M6 — Metadata** | TMDB enrichment, AniList rich metadata, TMDB recommendations | Week 8 |
| **M7 — Polish** | Discord RPC, notifications daemon, all 6 themes, poster rendering | Week 9-10 |
| **M8 — Release** | Cross-platform CI/CD, crates.io publish, docs | Week 11-12 |

---

## 12. Success Metrics

- Cold start to first search result: <= 2 seconds
- Stream resolution (any source): <= 3 seconds
- Provider uptime aggregate: >= 99% (at least one provider always works)
- Zero runtime panics in production builds
- All 5 content types working on all 4 platforms from day 1
