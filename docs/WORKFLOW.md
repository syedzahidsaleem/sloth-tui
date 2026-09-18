# Sloth — Development Workflow

> **Version:** 1.0.0  
> **Last Updated:** 2026-09-18  

---

## Overview

This document defines the end-to-end development workflow for Sloth — from environment setup through CI/CD to release. Follow this workflow exactly to ensure reproducible builds and consistent code quality.

---

## 1. Environment Setup

### Prerequisites

```powershell
# Install Rust (Windows)
winget install Rustlang.Rust.MSVC

# Or via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version   # should be 1.90.0+
cargo --version
```

### Toolchain Setup

```bash
# Set edition 2024 toolchain
rustup toolchain install stable
rustup default stable
rustup update

# Install useful cargo extensions
cargo install cargo-watch    # hot reload during development
cargo install cargo-audit    # security audit
cargo install cargo-expand   # macro expansion debugging
cargo install cargo-nextest  # faster test runner
cargo install sqlx-cli       # database migration management
```

### Cross-Compilation Targets (Optional for local dev, required for CI)

```bash
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-apple-darwin
rustup target add aarch64-linux-android
```

### IDE Setup

- **VS Code + rust-analyzer** (recommended)
- Enable `checkOnSave.command = "clippy"` in rust-analyzer settings
- Install `Even Better TOML` extension for `Cargo.toml` editing

---

## 2. Project Initialization (M1 — scaffold-agent task)

> ### ⚠️ GitHub Identity & Contributions — Mandatory Before Any Git Command
>
> **GitHub username: `syedzahidsaleem` — ONLY this username, always.**  
> **Commit Author Email: `syedzahidsaleem2@gmail.com` — verified GitHub email.**
>
> ```bash
> # Configure git author so contributions register on your profile:
> git config user.name "syedzahidsaleem"
> git config user.email "syedzahidsaleem2@gmail.com"
>
> # Set remote correctly (if repo already exists):
> git remote set-url origin https://github.com/syedzahidsaleem/sloth-tui.git
>
> # Verify before every push:
> git remote -v
> # Must show: origin  https://github.com/syedzahidsaleem/sloth-tui.git
> ```
>
> **NEVER use `syedzahid0307`.** If it appears anywhere in a remote URL, CI YAML, README badge, or any file, fix it immediately.

> ### ⚠️ Push After Every File — Mandatory
>
> After creating or modifying **each individual file**:
> ```bash
> git add <that file>
> git commit -m "type(scope): brief description"
> git push origin main
> ```
> One file = one commit = one push. No batching. No exceptions.

### Step 1: Get MovieBox-Tui source

```bash
# Create project directory
mkdir -p d:/Coding/Antigravity/Sloth
cd d:/Coding/Antigravity/Sloth

# Initialize as new Rust project
cargo init --name sloth-tui

# Clone MovieBox-Tui to extract source
git clone https://github.com/mesamirh/MovieBox-Tui /tmp/moviebox-tui

# Copy relevant source modules
cp /tmp/moviebox-tui/src/providers/moviebox/* src/providers/moviebox/
cp /tmp/moviebox-tui/src/providers/fourkhdhub/* src/providers/fourkhdhub/
cp /tmp/moviebox-tui/src/providers/addons/* src/providers/addons/
cp /tmp/moviebox-tui/src/providers/bdix/* src/providers/bdix/
cp /tmp/moviebox-tui/src/providers/tv/* src/providers/tv/
cp /tmp/moviebox-tui/src/cache.rs src/
cp /tmp/moviebox-tui/src/download.rs src/
cp /tmp/moviebox-tui/src/player.rs src/  # will be refactored
cp /tmp/moviebox-tui/src/player/tracker.rs src/player/
cp /tmp/moviebox-tui/src/proxy.rs src/
cp /tmp/moviebox-tui/src/updater/* src/updater/
cp /tmp/moviebox-tui/src/tui/* src/tui/ -r
cp /tmp/moviebox-tui/tests/* tests/ -r
```

### Step 2: Setup Database Migrations

```bash
# Initialize sqlx migrations
sqlx database create --database-url sqlite://sloth.db
sqlx migrate add initial

# Copy schema from SCHEMA.md into migrations/0001_initial.sql
# Then run:
sqlx migrate run --database-url sqlite://sloth.db
```

### Step 3: Verify baseline compiles

```bash
cargo check
cargo test --all -- --test-threads=4
```

---

## 3. Development Loop

### Daily Workflow

```bash
# Start development session
cd d:/Coding/Antigravity/Sloth

# Run with auto-reload (cargo-watch)
cargo watch -x "run -- --debug"

# In separate terminal: run tests on file change
cargo watch -x "nextest run"

# Before committing:
cargo fmt --all
cargo clippy -- -D warnings
cargo test --all
```

### Adding a New Provider

```bash
# 1. Create module directory
mkdir src/providers/{provider_name}

# 2. Create files
touch src/providers/{provider_name}/mod.rs
touch src/providers/{provider_name}/client.rs
# Add more as needed (parser.rs, models.rs, etc.)

# 3. Register in providers/mod.rs

# 4. Add to ProviderRegistry::new() in registry.rs

# 5. Add integration test
touch tests/{provider_name}_pipeline.rs

# 6. Run tests
cargo nextest run {provider_name}
```

### Adding a New Screen

```bash
# 1. Create screen file
touch src/tui/screens/{screen_name}.rs

# 2. Register in src/tui/screens/mod.rs

# 3. Add state to AppState in src/tui/state.rs

# 4. Add Actions to src/tui/action.rs

# 5. Add keyboard handlers in src/tui/app/keyboard.rs

# 6. Wire to tab in src/tui/screens/home.rs

# 7. Add snapshot test
# Run: cargo test --test tui_acceptance
```

### Adding a Database Migration

```bash
# Create migration file
sqlx migrate add {description}
# This creates: migrations/{timestamp}_{description}.sql

# Write SQL in the file

# Apply migration
sqlx migrate run --database-url sqlite://./sloth.db

# Re-run cargo check to verify sqlx query macros still compile
# (requires DATABASE_URL env var set)
export DATABASE_URL="sqlite://./sloth.db"
cargo check
```

---

## 4. Git Workflow

### Branch Strategy

```
main                    # Always stable, CI passing
├── dev                 # Integration branch
│   ├── feat/m1-scaffold
│   ├── feat/m2-anime-providers
│   ├── feat/m2-anime-tui
│   ├── feat/m3-sports
│   ├── feat/m4-f1
│   ├── feat/m5-tracking
│   ├── feat/m6-metadata
│   └── feat/m7-polish
```

### Commit Message Format (Conventional Commits)

```
{type}({scope}): {description}

Types:
  feat     - new feature
  fix      - bug fix
  refactor - code restructure without behavior change
  docs     - documentation only
  test     - tests only
  chore    - build, deps, CI
  perf     - performance improvement

Scopes:
  providers/moviebox
  providers/hianime
  providers/allanime
  providers/streamed
  providers/f1
  tui/home
  tui/anime
  tui/sports
  tui/f1
  tui/theme
  db
  tracking/anilist
  tracking/trakt
  tracking/discord
  metadata/tmdb
  player/mpv
  config
  ci

Examples:
  feat(providers/hianime): implement search and stream resolution
  fix(player/mpv): handle IPC socket timeout on Windows
  test(providers/streamed): add wiremock fixtures for live match list
  chore(ci): add aarch64-linux-android build target
```

### PR Checklist

Before opening a PR, verify:
- [ ] `cargo fmt --all` — no diff
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo test --all` — all pass
- [ ] `cargo audit` — no high severity vulnerabilities
- [ ] Cross-compile check on at least 2 non-host platforms
- [ ] Snapshot tests updated (`cargo insta review`)
- [ ] New public APIs have `///` doc comments
- [ ] CHANGELOG.md updated

---

## 5. CI/CD Pipeline

### GitHub Actions Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:
    branches: [main, dev]

jobs:
  check:
    name: Check (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --target ${{ matrix.target }}

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --all

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --all --check
      - run: cargo clippy -- -D warnings

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Release Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            binary: sloth-tui
            archive: sloth-tui-linux-x86_64.tar.gz
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            binary: sloth-tui
            archive: sloth-tui-linux-aarch64.tar.gz
          - target: x86_64-apple-darwin
            os: macos-latest
            binary: sloth-tui
            archive: sloth-tui-macos-x86_64.tar.gz
          - target: aarch64-apple-darwin
            os: macos-latest
            binary: sloth-tui
            archive: sloth-tui-macos-aarch64.tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            binary: sloth-tui.exe
            archive: sloth-tui-windows-x86_64.zip
    # ... build + upload to GitHub Release
```

---

## 6. Source Integration Strategy

### What We Take From MovieBox-Tui (Verbatim)

| Module | Files | Action |
|---|---|---|
| MovieBox provider | `src/providers/moviebox/*` | Copy verbatim |
| 4KHDHub provider | `src/providers/fourkhdhub/*` | Copy verbatim |
| Stremio addons | `src/providers/addons/*` | Copy verbatim |
| BDIX providers | `src/providers/bdix/*` | Copy verbatim |
| IPTV TV provider | `src/providers/tv/*` | Copy verbatim |
| Image cache | `src/cache.rs` | Copy verbatim |
| Download engine | `src/download.rs` | Copy verbatim |
| Proxy support | `src/proxy.rs` | Copy verbatim |
| Auto-updater | `src/updater/*` | Copy verbatim |
| Theme system | `src/tui/theme.rs` | Copy, then extend |
| Widget library | `src/tui/widgets/*` | Copy, then extend |
| Action enum | `src/tui/action.rs` | Copy, then extend |
| App state | `src/tui/state.rs` | Copy, then extend |
| Event loop | `src/tui/app/run.rs` | Copy, then extend |
| All existing tests | `tests/*` | Copy verbatim, keep all passing |

### What We Rename/Refactor

| Old | New | Reason |
|---|---|---|
| `moviebox-tui` (crate name) | `sloth-tui` | New project identity |
| `src/favorites.rs` | `src/db/favorites.rs` | Centralize DB code |
| `src/history.rs` | `src/db/history.rs` | Centralize DB code |
| `src/player.rs` | `src/player/mpv.rs` | Split by player |

### What We Add (New Code)

- `src/providers/anime/` — all anime providers
- `src/providers/sports/` — streamed.pk + IPTV sports
- `src/providers/f1/` — F1 calendar + streams
- `src/providers/registry.rs` — unified provider registry with fallback
- `src/metadata/` — TMDB + AniList metadata
- `src/tracking/` — AniList sync, Trakt, Discord RPC
- `src/db/` — SQLite with sqlx
- `src/daemon/` — notification daemon
- `src/tui/screens/anime.rs` — anime tab
- `src/tui/screens/sports.rs` — sports tab
- `src/tui/screens/f1.rs` — F1 tab
- `src/tui/widgets/countdown.rs` — countdown widget
- `src/tui/widgets/track_bar.rs` — progress bar widget

---

## 7. Testing Strategy

### Test Pyramid

```
                   ┌───────────┐
                   │   E2E     │  (manual testing — player launch)
                   │   Tests   │
                 ┌─┴───────────┴─┐
                 │  Integration  │  (wiremock, temp SQLite)
                 │    Tests      │
               ┌─┴───────────────┴─┐
               │    Unit Tests     │  (parsers, cache, state logic)
               └───────────────────┘
```

### Running Tests

```bash
# All tests
cargo nextest run --all

# Specific module
cargo nextest run providers::anime

# With output
cargo nextest run -- --nocapture

# Snapshot tests review
cargo insta review

# Generate wiremock fixtures from live endpoints (during development)
# Set CAPTURE_FIXTURES=1 to record live responses
CAPTURE_FIXTURES=1 cargo test hianime_pipeline

# Database tests (requires DATABASE_URL)
DATABASE_URL="sqlite://./test.db" cargo test db::
```

### Test Coverage Targets

| Component | Target Coverage |
|---|---|
| Provider parsers | > 90% |
| Database queries | > 85% |
| Config parsing | > 95% |
| Error paths | > 80% |
| TUI state logic | > 75% |
| Theme rendering | Snapshot tested |

---

## 8. Debugging

### Debug Build

```bash
# Debug build with logging
RUST_LOG=debug cargo run

# Trace level (very verbose)
RUST_LOG=sloth_tui=trace cargo run

# Debug a specific module
RUST_LOG=sloth_tui::providers::hianime=trace cargo run
```

### Log File Location

- Linux/macOS: `~/.local/share/sloth-tui/sloth.log`
- Windows: `%LOCALAPPDATA%\sloth-tui\sloth.log`

### mpv IPC Debugging

```bash
# Manual IPC test while Sloth is running with mpv
echo '{"command": ["get_property", "time-pos"]}' | nc -U /tmp/sloth-mpv.sock
```

### Database Inspection

```bash
# Open the SQLite database
sqlite3 ~/.local/share/sloth-tui/sloth.db

# Useful queries:
.tables
SELECT * FROM watch_history ORDER BY watched_at DESC LIMIT 10;
SELECT * FROM anilist_entries WHERE dirty = 1;
```

---

## 9. Release Checklist

Before tagging a release:

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Run full test suite
cargo nextest run --all

# 4. Security audit
cargo audit

# 5. Check all platforms compile
cargo check --target x86_64-unknown-linux-musl
cargo check --target x86_64-pc-windows-msvc
cargo check --target aarch64-apple-darwin

# 6. Build release binary
cargo build --release

# 7. Test the binary manually
./target/release/sloth-tui

# 8. Tag and push
git tag v0.1.0
git push origin v0.1.0
```
