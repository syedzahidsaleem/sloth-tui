# Sloth — Dependencies

> **Version:** 1.0.0  
> **Runtime:** Rust 1.90+, edition 2024  
> **Last Updated:** 2026-09-18  

---

## Cargo.toml (Complete)

```toml
[package]
name = "sloth-tui"
version = "0.1.0"
edition = "2024"
rust-version = "1.90.0"
description = "Terminal interface for movies, anime, sports, F1, and live TV — zero cost, always fast."
repository = "https://github.com/zahidsaleem/sloth-tui"
license = "MIT OR Apache-2.0"
keywords = ["tui", "streaming", "anime", "sports", "movies"]
categories = ["command-line-utilities", "multimedia"]
readme = "README.md"

# ============================================================
# Core TUI & Terminal
# ============================================================
[dependencies]
ratatui         = "0.30.2"
crossterm       = { version = "0.29.0", features = ["event-stream"] }
ratatui-image   = { version = "11.0.6", default-features = false, features = [
                    "crossterm",
                    "rustix",           # sixel support on Linux
                ] }

# ============================================================
# Async Runtime & HTTP
# ============================================================
tokio           = { version = "1", features = [
                    "rt-multi-thread",
                    "macros",
                    "time",
                    "fs",
                    "process",
                    "net",              # for mpv IPC Unix socket
                    "io-util",
                    "sync",
                ] }
reqwest         = { version = "0.12", default-features = false, features = [
                    "rustls-tls",       # pure-Rust TLS, no OpenSSL dep
                    "json",
                    "gzip",
                    "brotli",
                    "cookies",
                    "stream",
                ] }
futures         = "0.3"

# ============================================================
# Serialization
# ============================================================
serde           = { version = "1", features = ["derive"] }
serde_json      = "1"
toml            = "0.8"

# ============================================================
# Database
# ============================================================
sqlx            = { version = "0.8", default-features = false, features = [
                    "runtime-tokio",
                    "sqlite",
                    "macros",
                    "migrate",
                ] }

# ============================================================
# Image Handling (posters)
# ============================================================
image           = { version = "0.25", default-features = false, features = [
                    "jpeg",
                    "png",
                    "webp",
                ] }

# ============================================================
# Cache
# ============================================================
lru             = "0.12"

# ============================================================
# Crypto (MovieBox provider — keep from upstream)
# ============================================================
base64          = "0.22"
hmac            = "0.12"
md-5            = "0.10"
rmp-serde       = "1.3"                 # MessagePack — MovieBox protocol

# ============================================================
# Filesystem & Paths
# ============================================================
dirs            = "5"

# ============================================================
# Logging & Diagnostics
# ============================================================
tracing         = "0.1"
tracing-subscriber = { version = "0.3", features = [
                    "env-filter",
                    "json",
                ] }

# ============================================================
# Error Handling
# ============================================================
thiserror       = "1"
anyhow          = "1"

# ============================================================
# CLI Argument Parsing
# ============================================================
clap            = { version = "4", features = ["derive", "env"] }

# ============================================================
# Utilities
# ============================================================
rand            = "0.9"
chrono          = { version = "0.4", features = ["serde"] }
uuid            = { version = "1", features = ["v4"] }
url             = "2"
regex           = "1"
once_cell       = "1"
parking_lot     = "0.12"                # faster RwLock/Mutex than std

# ============================================================
# OAuth2 (AniList, Trakt)
# ============================================================
oauth2          = { version = "4", default-features = false, features = [
                    "rustls-tls",
                ] }

# ============================================================
# Discord Rich Presence
# ============================================================
discord-presence = "0.4"               # discord-rpc IPC protocol

# ============================================================
# OS Notifications (daemon notifier)
# ============================================================
notify-rust     = { version = "4", default-features = false, features = [
                    "z_dbus",           # Linux D-Bus
                ] }

# ============================================================
# Async Trait
# ============================================================
async-trait     = "0.1"

# ============================================================
# M3U Parser (IPTV)
# ============================================================
m3u             = "0.1"                 # or hand-rolled, simple format

# ============================================================
# HTML Parsing (scraper providers)
# ============================================================
scraper         = "0.20"               # CSS selector HTML parser

# ============================================================
# Random
# ============================================================
fastrand        = "2"                  # faster than rand for non-crypto

# ============================================================
# Platform-Specific
# ============================================================
[target.'cfg(not(target_os = "android"))'.dependencies]
mimalloc        = { version = "0.1", default-features = false }

[target.'cfg(windows)'.dependencies]
windows         = { version = "0.58", features = [
                    "Win32_UI_Shell",   # for player detection
                ] }

# ============================================================
# Dev & Test Dependencies
# ============================================================
[dev-dependencies]
tokio           = { version = "1", features = ["test-util", "macros"] }
wiremock        = "0.6"                 # mock HTTP server for provider tests
tempfile        = "3"
pretty_assertions = "1"
insta           = "1"                   # snapshot testing

# ============================================================
# Build Configuration
# ============================================================
[profile.release]
opt-level       = 3
lto             = "thin"
codegen-units   = 1
strip           = "debuginfo"
panic           = "abort"

[profile.dev]
opt-level       = 1

[profile.test]
opt-level       = 1

# ============================================================
# Features
# ============================================================
[features]
default         = ["notifications", "discord"]
notifications   = ["notify-rust"]
discord         = ["discord-presence"]
# Build without optional features for minimal binary
minimal         = []
```

---

## Dependency Rationale

### Why `rustls-tls` over `native-tls`?

`native-tls` links against the OS TLS library (OpenSSL on Linux, SChannel on Windows, SecureTransport on macOS). This means:
- OpenSSL version conflicts on Linux distributions
- Extra binary size overhead
- Complex cross-compilation

`rustls` is pure Rust, statically linked, zero system dependency. It produces a truly portable static binary on all platforms with `musl` targets. It is also faster on modern CPUs due to optimized Rust code.

### Why `sqlx` over `rusqlite`?

`sqlx` provides:
- Compile-time query checking (`query!` macro)
- Async-native (no blocking thread pool tricks)
- Built-in migration runner
- Connection pooling via `SqlitePool`
- Zero-copy row extraction

`rusqlite` is sync-only and requires `spawn_blocking` for every query, adding latency and thread overhead.

### Why `scraper` for HTML parsing?

`scraper` uses the same parsing engine as Firefox (`html5ever`) with CSS selector querying. Providers like HiAnime serve regular HTML that must be parsed with selector-based extraction. `scraper` handles malformed HTML gracefully without panicking — essential for scraper stability.

### Why `parking_lot` over `std::sync`?

`parking_lot::RwLock` and `Mutex` are 2-3x faster than std equivalents in low-contention scenarios due to a more efficient futex-based implementation. The provider health map and config are shared-read hot paths.

### Why `discord-presence` over `discord-rpc`?

`discord-presence` is an actively maintained pure-Rust implementation of the Discord IPC protocol. It handles socket path detection on all platforms (including Windows named pipes) and is fully async-compatible.

### Why `notify-rust` over `libnotify`?

`notify-rust` supports Windows (toast notifications), macOS (NSUserNotification/UNNotification), and Linux (D-Bus) from a single API. Optional dependency — disabled on Android where `termux-notification` is used instead.

---

## Crate Version Lock Policy

- All crate versions are specified with **compatible version** (`"X.Y"`) not `"*"`.
- `Cargo.lock` is **committed** to the repository for reproducible builds.
- Dependabot configured to auto-open PRs for security updates on a weekly schedule.
- Breaking version bumps (e.g., reqwest 0.12 -> 0.13) require a dedicated PR with manual testing.

---

## External Runtime Dependencies

These are **not** Rust crates — they are external binaries that Sloth requires or optionally uses at runtime.

| Binary | Requirement | Purpose |
|---|---|---|
| `mpv` | **Recommended** | Primary video player with IPC resume support |
| `vlc` | Optional | Fallback video player |
| `iina` | Optional (macOS) | macOS-native video player |
| `yt-dlp` | Optional | URL extraction helper for some providers |
| `ffmpeg` | Optional | Downloaded file format conversion |

Sloth auto-detects players in PATH. If no player is found, it shows a setup prompt on first launch.

---

## Supply Chain Security

All dependencies are:
- Pinned in `Cargo.lock`
- Audited via `cargo audit` in CI (weekly)
- Sourced exclusively from crates.io (no git dependencies in production)
- License-compatible (MIT/Apache-2.0 project, all deps are MIT/Apache/ISC/BSD)

### License Summary

| Crate | License |
|---|---|
| ratatui | MIT |
| tokio | MIT |
| reqwest | MIT |
| sqlx | MIT OR Apache-2.0 |
| serde | MIT OR Apache-2.0 |
| scraper | MIT |
| crossterm | MIT |
| discord-presence | MIT |
| notify-rust | MIT OR Apache-2.0 |
| oauth2 | MIT OR Apache-2.0 |
| All others | MIT / Apache-2.0 / ISC |

---

## Build Matrix (CI)

```yaml
# Tested platforms in CI
targets:
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl    # static binary
  - aarch64-unknown-linux-gnu
  - x86_64-apple-darwin
  - aarch64-apple-darwin
  - x86_64-pc-windows-msvc
  - aarch64-linux-android         # Termux
```

---

## Upgrade Path from MovieBox-Tui

The following crates are inherited from MovieBox-Tui with version bumps as needed:

| MovieBox-Tui | Sloth | Notes |
|---|---|---|
| `ratatui = "0.30.2"` | Same | Keep locked |
| `crossterm = "0.29.0"` | Same | Keep locked |
| `ratatui-image = "11.0.6"` | Same | Keep locked |
| `base64 = "0.23.0"` | `"0.22"` | Normalize to stable |
| `rmp-serde = "1.3.1"` | Same | Keep |
| `hmac = "0.13.0"` | `"0.12"` | Normalize |
| `md-5 = "0.11.0"` | `"0.10"` | Normalize |
| `lru = "0.18.1"` | `"0.12"` | Latest stable |
| `rand = "0.10.2"` | `"0.9"` | Stable |
| `dirs = "6.0.0"` | `"5"` | Stable |
| `futures = "0.3.31"` | Same | |

> Note: Exact versions will be finalized when the Cargo.lock is generated. The versions above are the target range and may shift slightly to resolve dependency conflicts.
