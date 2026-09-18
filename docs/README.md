# Sloth — Documentation Index

> Zero-cost, always-fast terminal streaming for movies, anime, sports, Formula 1, and live TV.
> Built in Rust on Ratatui. Fork of MovieBox-Tui, massively extended.

---

## Documents

| Document | Purpose | Read When |
|---|---|---|
| [PRD.md](PRD.md) | Product Requirements — what we're building and why | Planning, scope decisions |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design, module map, data flow diagrams | Understanding the codebase |
| [TDD.md](TDD.md) | Technical Design — exact types, traits, signatures | Writing any code |
| [SCHEMA.md](SCHEMA.md) | SQLite database schema, all tables and queries | Database work |
| [DEPENDENCIES.md](DEPENDENCIES.md) | Cargo.toml with full rationale for every crate | Adding/changing deps |
| [DESIGN.md](DESIGN.md) | Visual layouts, color system, icon system, responsive rules | UI implementation |
| [RULES.md](RULES.md) | Mandatory coding rules for Antigravity agents | Every code generation task |
| [AGENTS.md](AGENTS.md) | Agent personas, responsibilities, handoff protocol | Multi-agent orchestration |
| [WORKFLOW.md](WORKFLOW.md) | Git workflow, CI/CD, testing strategy, release checklist | Dev process |
| [IMPLEMENTATION.md](IMPLEMENTATION.md) | Milestone-by-milestone file-level implementation plan | Executing each milestone |
| [PROMPTS.md](PROMPTS.md) | 18 complete Antigravity prompts — run in order | Actually building Sloth |

---

## Quick Start for Building

1. Read [PROMPTS.md](PROMPTS.md) — it contains everything needed
2. Run **Prompt 1** in Antigravity to scaffold the project
3. Run each subsequent prompt in order, verifying after each one
4. Reference other docs when Antigravity needs more context

---

## Architecture in One Sentence

Sloth is a Tokio-async Ratatui TUI where every content source implements a `Provider` trait with a declared capability set, wired through a `ProviderRegistry` that runs parallel searches and sequential fallback resolution, all mutations flowing to a single Action channel that drives the render loop.

---

## Content Sources

| Content | Primary Source | Fallback |
|---|---|---|
| Movies/TV | MovieBox API | 4KHDHub, Stremio addons |
| Anime | HiAnime (AniWatch) | AllAnime |
| Sports | streamed.su API | iptv-org sports M3U |
| Formula 1 | Ergast/Jolpica API | iptv-org F1 channels |
| Live TV | iptv-org M3U bundles | Custom M3U URL |

---

## Technology Stack

- **Language:** Rust (edition 2024, MSRV 1.90.0)
- **TUI:** Ratatui 0.30 + crossterm
- **Async:** Tokio multi-thread
- **HTTP:** reqwest with rustls-tls (pure-Rust, no OpenSSL)
- **Database:** SQLite via sqlx (async, compile-time query checking)
- **Images:** ratatui-image (Kitty, Sixel, Unicode block)
- **Tracking:** AniList GraphQL OAuth, Trakt.tv OAuth
- **Metadata:** TMDB API v3
- **Player:** mpv (primary, with IPC resume), VLC, IINA
- **Allocator:** mimalloc (non-Android)
- **Platforms:** Windows, macOS, Linux, Android (Termux)
