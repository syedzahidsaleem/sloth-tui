# Changelog

All notable changes to Sloth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-20

### Added
- **Multi-Media Aggregator Engine**:
  - Movie & TV streaming across MovieBox, 4KHDHub, Stremio Community Addons, and BDIX mirrors with TMDB metadata enrichment.
  - Dual-source anime streaming engine supporting HiAnime and AllAnime with seamless Sub/Dub switching and episode extraction.
  - Live global sports aggregation powered by Streamed.pk with real-time stream status, event categories, and live match links.
  - Formula 1 cockpit with complete season calendar from OpenF1 and Jolpica APIs, local timezone conversion, live session countdowns, and direct stream launching.
  - Live TV (IPTV) player with M3U playlist parsing, genre/country filtering, and instant channel search.
- **Terminal User Interface (Ratatui)**:
  - 5 core views: Movies & Series, Anime, Live Sports, Formula 1, and Live TV.
  - In-terminal poster artwork rendering supporting Kitty graphics protocol, Sixel, and Halfblock fallbacks.
  - Curated themes: Catppuccin Mocha, Tokyo Night, Nord, Gruvbox, and Dracula.
  - Context-aware keyboard shortcut hint bar across all views.
  - Animated ASCII startup banner with `--quiet` / `-q` CLI bypass flag.
  - In-TUI settings manager with live theme switching, player configuration, Discord RPC toggles, and notification preferences.
- **Player & Downloads**:
  - Bi-directional mpv IPC controller with playback position tracking, volume control, and auto-resume from watch history.
  - High-performance multi-connection chunked download manager with pause/resume and TUI progress tracking.
- **Tracking & Desktop Integration**:
  - AniList OAuth progress synchronization and anime tracking.
  - Discord Rich Presence (RPC) displaying current show, movie, or live sport session with elapsed time.
  - Cross-platform background notification daemon alerting for upcoming F1 sessions (configurable 15m lead time) and new anime episodes.
  - Notification deduplication with 7-day automated SQLite retention cleanup.
- **Storage & Infrastructure**:
  - SQLite database layer using `sqlx` migrations for favorites, watch history, cached API responses, and notification tracking.
  - Unified `ProviderRegistry` with automatic fallback routing and resilience against upstream downtime.
  - Multi-platform GitHub Actions CI pipeline and release workflow supporting Linux, macOS, and Windows.

### Changed
- Refactored core architecture from legacy single-provider design into a decoupled modular workspace.
- Consolidated watch history and bookmarks under a single SQLite repository with transactional integrity.

### Fixed
- Fixed SQLite in-memory test database pool isolation for reliable CI execution.
- Fixed cross-platform IPC socket path resolution on Windows named pipes versus Unix domain sockets.
- Corrected F1 session datetime parsing and timezone offsets for race weekends.
