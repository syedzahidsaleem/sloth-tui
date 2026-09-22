# 🦥 Sloth

> **Terminal interface for movies, anime, sports, F1, and live TV — zero cost, always fast.**

[![CI](https://github.com/syedzahidsaleem/sloth-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/syedzahidsaleem/sloth-tui/actions/workflows/ci.yml)
[![Release](https://github.com/syedzahidsaleem/sloth-tui/actions/workflows/release.yml/badge.svg)](https://github.com/syedzahidsaleem/sloth-tui/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Sloth is a keyboard-driven, high-performance TUI streaming aggregator built in Rust with [Ratatui](https://ratatui.rs) and powered by [mpv](https://mpv.io). It unifies movies, television, anime, live global sports, Formula 1, and IPTV into a single terminal cockpit with automatic stream resolution, cloud sync, background notifications, and Discord Rich Presence.

---

## ✨ Features

- 🎬 **Movies & Series**: Aggregate streams across MovieBox, 4KHDHub, Stremio Community Addons, and BDIX mirrors with TMDB metadata, posters, and automatic subtitle extraction.
- ⛩️ **Anime Aggregation**: Dual-engine streaming via HiAnime and AllAnime with seamless Sub/Dub switching, AniList OAuth progress synchronization, and episode countdowns.
- ⚽ **Live Sports**: Streamed.pk live multi-sport aggregator covering football, basketball, combat sports, cricket, and motorsport with real-time stream status and live score updates.
- 🏎️ **Formula 1 Cockpit**: Comprehensive season calendar powered by OpenF1 and Jolpica APIs with local timezone conversion, live session countdown timers, and one-click stream access.
- 📺 **Live TV (IPTV)**: Built-in M3U parser supporting categorized channel browsing, search, and instant playback.
- 🚀 **mpv IPC Integration**: Bi-directional IPC socket communication with mpv tracking exact playback position, auto-resuming from watch history, and scrobbling.
- 📥 **Built-in Downloader**: High-speed multi-threaded chunked downloader with resume support and progress visualization directly inside the TUI.
- 🔔 **Notification Daemon**: Cross-platform background notification engine alerting you 15 minutes before F1 sessions start and when new tracked anime episodes drop.
- 🎮 **Discord Rich Presence**: Dynamic Rich Presence displaying current show/movie/session, live elapsed time, and state.
- 🎨 **Modern Aesthetics**: Curated color palettes including Catppuccin Mocha, Tokyo Night, Nord, Gruvbox, and Dracula with Kitty and Sixel terminal poster rendering.

---

## 📦 Installation

### Prerequisites

Sloth requires **mpv** for media playback:

- **Linux**: `sudo apt install mpv` or `sudo pacman -S mpv`
- **macOS**: `brew install mpv`
- **Windows**: `winget install shinchiro.mpv` or `choco install mpv`
- **Android (Termux)**: `pkg install mpv` (or install [mpv-android](https://github.com/mpv-android/mpv-android))

---

### Platform Packages

#### Linux
```bash
# Via curl install script
curl -fsSL https://raw.githubusercontent.com/syedzahidsaleem/sloth-tui/main/install.sh | bash

# Or via Cargo
cargo install sloth-tui
```

#### macOS
```bash
# Via Homebrew tap
brew tap syedzahidsaleem/sloth
brew install sloth-tui

# Or via Cargo
cargo install sloth-tui
```

#### Windows
```powershell
# Automated PowerShell installer (downloads latest release and configures PATH)
irm https://raw.githubusercontent.com/syedzahidsaleem/sloth-tui/main/install.ps1 | iex

# Or via winget
winget install syedzahidsaleem.sloth-tui

# Or via Cargo
cargo install sloth-tui
```

#### Android (Termux)
```bash
pkg update && pkg install rust git mpv
cargo install sloth-tui
```

---

## ⚡ Quick Start

1. **Launch Sloth**:
   ```bash
   sloth-tui
   ```
   *(Use `sloth-tui --quiet` to bypass the startup banner)*

2. **Navigate**:
   - Use `Tab` / `Shift+Tab` or numbers `1`–`5` to switch between Movies, Anime, Sports, F1, and Live TV.
   - Use `j` / `k` (or arrow keys) to browse items.

3. **Stream**:
   - Press `Enter` to resolve stream sources and start playback in mpv.
   - Press `/` anytime to search titles, teams, or channels.

---

## ⚙️ Configuration

Sloth saves configuration to:
- **Linux/macOS**: `~/.config/sloth/config.toml`
- **Windows**: `%APPDATA%\sloth\config.toml`

### Example `config.toml`

```toml
[general]
theme = "CatppuccinMocha"  # Options: CatppuccinMocha, TokyoNight, Nord, Gruvbox, Dracula
player = "mpv"
download_dir = "~/Downloads/Sloth"

[providers]
tmdb_api_key = ""         # Optional: your own TMDB API key
prefer_dub = false        # Prefer English dubs for anime

[tracking]
anilist_token = ""        # AniList OAuth access token
trakt_token = ""          # Trakt OAuth access token

[discord]
rpc_enabled = true

[notifications]
enabled = true
f1_lead_time_minutes = 15
anime_alerts = true
```

---

## ⌨️ Keybindings

| Key | Action | Context |
|---|---|---|
| `Tab` / `Shift+Tab` | Switch active tab | Global |
| `1` – `5` | Jump directly to tab (Movies, Anime, Sports, F1, TV) | Global |
| `/` | Open search dialog | Global |
| `Esc` | Clear search / close modal | Global |
| `q` | Quit application | Global |
| `j` / `k` or `↓` / `↑` | Navigate lists | Lists |
| `Enter` | Play stream / select item | General |
| `f` | Toggle favorite | Movies / TV |
| `d` | Download selected media | Movies / Anime |
| `t` | Toggle Sub / Dub audio | Anime |
| `a` | Sync watch progress to AniList | Anime |
| `w` | Watch live sports / F1 stream | Sports / F1 |
| `r` | Refresh schedule and streams | Sports / F1 |
| `?` | Toggle keybindings cheat sheet | Global |

---

## 🤝 Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details, test workflows, and coding guidelines.

```bash
# Clone and build
git clone https://github.com/syedzahidsaleem/sloth-tui.git
cd sloth-tui
cargo check
cargo test --all
```

---

## 📄 License

Licensed under either of [MIT License](LICENSE) or [Apache License 2.0](LICENSE) at your option.
