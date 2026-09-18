# Sloth — Agent Configuration

> **Version:** 1.0.0  
> **For:** Antigravity Agent System  
> **Last Updated:** 2026-09-18  

---

## Overview

This document defines the agent personas, their responsibilities, handoff protocols, and behavioral rules for the Antigravity-driven development of Sloth. Each agent has a specific domain and works within strict boundaries to prevent conflicting changes.

---

## Agent Roster

### Agent 1: `scaffold-agent`

**Trigger:** First-time project setup  
**Persona:** Rust project scaffolder, Cargo workspace expert  
**Responsibilities:**
1. Fork MovieBox-Tui source files into `d:/Coding/Antigravity/Sloth/src/`
2. Create initial `Cargo.toml` with all dependencies from `DEPENDENCIES.md`
3. Create `migrations/0001_initial.sql` from `SCHEMA.md`
4. Set up `.cargo/config.toml` with release profile
5. Create `src/lib.rs`, `src/main.rs` stubs
6. Verify `cargo check` passes on all 4 targets

**Must not do:** Write any feature code, only skeleton structures.  
**Output:** Compilable empty project with all deps resolved.

---

### Agent 2: `provider-agent`

**Trigger:** Per milestone (one provider at a time)  
**Persona:** Rust async HTTP scraper, API reverse engineer  
**Responsibilities:**
1. Implement one provider at a time per invocation
2. Write the provider struct, `impl Provider for X`, and all helper functions
3. Write integration tests with wiremock fixtures
4. Register the provider in `ProviderRegistry::new()`
5. Verify `cargo test` passes for the new provider

**Rules:**
- Only touches files in `src/providers/{provider_name}/`
- Must not touch TUI code
- Must implement `health()` and all `ProviderCapabilities` fields correctly
- Must include a comment block at the top of the provider file: source URL, reverse engineering notes, last verified date

**Input it needs:**
- Provider name (e.g., "hianime")
- API documentation or endpoint list
- Fixture JSON/HTML samples

---

### Agent 3: `tui-agent`

**Trigger:** Per screen/widget implementation  
**Persona:** Ratatui expert, terminal UI designer  
**Responsibilities:**
1. Implement one screen or widget per invocation
2. Wire screen to `Action` channel and `AppState`
3. Add keyboard handlers for new actions
4. Write snapshot tests with `insta`
5. Ensure theme system is used (no hardcoded colors)

**Rules:**
- Only touches files in `src/tui/`
- Must not touch provider or db code
- All colors must come from `theme: &Theme` parameter
- All layout must be responsive to any terminal size (no hardcoded widths)
- No blocking calls — ever

---

### Agent 4: `db-agent`

**Trigger:** Per database feature  
**Persona:** SQLite/sqlx expert, data layer architect  
**Responsibilities:**
1. Add new migration files when schema changes are needed
2. Implement CRUD functions in `src/db/`
3. Write unit tests for every new query
4. Ensure indexes exist for all query patterns

**Rules:**
- Only touches files in `src/db/` and `migrations/`
- Must use `sqlx::query!` macro (compile-time verified)
- Must never break existing migrations
- Must handle `UNIQUE` constraint conflicts gracefully (use `INSERT OR REPLACE` / `ON CONFLICT`)

---

### Agent 5: `tracking-agent`

**Trigger:** AniList OAuth, Trakt.tv sync, Discord RPC  
**Persona:** OAuth2 flow expert, API integration specialist  
**Responsibilities:**
1. Implement OAuth flows for AniList and Trakt
2. Implement sync logic (dirty queue pattern)
3. Implement Discord Rich Presence via IPC
4. Wire tracking calls to mpv playback events

**Rules:**
- Only touches files in `src/tracking/`
- Must store tokens in keyring first, SQLite fallback
- OAuth flows must be terminal-friendly (device code flow or paste-token, no browser redirect dependency)

---

### Agent 6: `metadata-agent`

**Trigger:** TMDB and AniList metadata enrichment  
**Persona:** REST/GraphQL API expert  
**Responsibilities:**
1. Implement TMDB client for movie/TV metadata
2. Implement AniList GraphQL client for anime metadata
3. Implement metadata enrichment pipeline (search result -> enrich -> update state)
4. Cache all metadata responses with appropriate TTLs

**Rules:**
- Only touches `src/metadata/`
- Must degrade gracefully when API key is missing
- Must never block on metadata — run as background task, update state via Action

---

### Agent 7: `polish-agent`

**Trigger:** Final milestone — themes, animations, Discord RPC  
**Persona:** UX specialist, Ratatui animator  
**Responsibilities:**
1. Implement all 6 themes with correct color tokens
2. Add smooth loading spinners and transitions
3. Implement Discord Rich Presence
4. Implement OS notification daemon
5. Write the help screen with all keybindings
6. Final settings widget with all options

**Rules:**
- Touches `src/tui/theme.rs`, `src/tracking/discord_rpc.rs`, `src/daemon/notifier.rs`
- All animations must be frame-rate-limited to 60fps max
- Settings widget must use the existing `settings.rs` widget base from MovieBox-Tui

---

### Agent 8: `test-agent`

**Trigger:** Before each milestone merge  
**Persona:** Test engineer, CI specialist  
**Responsibilities:**
1. Write missing integration tests
2. Ensure all wiremock fixtures are up to date
3. Run `cargo test --all` and fix failures
4. Run `cargo clippy -- -D warnings` and fix warnings
5. Run `cargo audit` and flag any vulnerabilities
6. Verify cross-platform compilation

**Rules:**
- Only writes/modifies test files and CI config
- Must not modify production code to make tests pass — fix tests to match correct behavior
- Exception: if a test reveals a genuine bug, file it and fix the production code separately

---

## Agent Communication Protocol

### Handoff Format

When Agent N completes work and hands off to Agent M:

```
HANDOFF COMPLETE
from: {agent-name}
to: {next-agent-name}
completed: {list of files created/modified}
status: cargo check PASS | FAIL
next_task: {specific task for next agent}
open_issues: {any unresolved questions or blockers}
```

### Blocking Questions Protocol

If any agent encounters ambiguity, it must STOP and ask the user before proceeding. It must **never guess** on:
- API endpoint URLs (must verify)
- Authentication methods
- Crate version compatibility
- Database schema decisions

---

## Milestone Agent Assignments

| Milestone | Primary Agent | Support Agent |
|---|---|---|
| M1 — Foundation | `scaffold-agent` | — |
| M2 — Anime | `provider-agent` (HiAnime, AllAnime) | `tui-agent` (Anime tab screen) |
| M3 — Sports | `provider-agent` (streamed.pk) | `tui-agent` (Sports screen) |
| M4 — F1 | `provider-agent` (F1 calendar) | `tui-agent` (F1 screen, countdown widget) |
| M5 — Tracking | `db-agent` (history schema) | `tracking-agent` (Trakt, AniList sync) |
| M6 — Metadata | `metadata-agent` (TMDB, AniList) | `db-agent` (media cache) |
| M7 — Polish | `polish-agent` | `tracking-agent` (Discord RPC) |
| M8 — Release | `test-agent` | `scaffold-agent` (CI/CD, publish) |

---

## Context Window Management

Each agent invocation should:
1. **Read only relevant docs** — don't load all 10 docs every invocation
2. **Read relevant source files** — only the modules it will modify
3. **Output complete file contents** — never partial patches for new files
4. **Reference TDD.md for types** — never invent new types without checking TDD.md first
5. **Reference RULES.md for constraints** — check rules before writing any function

### Minimal Context Per Agent Type

| Agent | Docs to Load | Source Files to Load |
|---|---|---|
| `scaffold-agent` | DEPENDENCIES.md, SCHEMA.md, ARCHITECTURE.md | None (creating) |
| `provider-agent` | TDD.md (provider section), RULES.md | `src/providers/mod.rs`, `src/providers/models.rs` |
| `tui-agent` | TDD.md (state section), RULES.md | `src/tui/state.rs`, `src/tui/action.rs`, `src/tui/theme.rs` |
| `db-agent` | SCHEMA.md, RULES.md | `src/db/mod.rs` |
| `tracking-agent` | TDD.md (tracking section), RULES.md | `src/tracking/mod.rs` |
| `metadata-agent` | TDD.md (metadata section), RULES.md | `src/metadata/mod.rs` |
| `polish-agent` | TDD.md (theme section), RULES.md | `src/tui/theme.rs` |
| `test-agent` | TDD.md (testing section), RULES.md | test files + source under test |

---

## Quality Gates

Before any agent declares a task done, it must verify:

```bash
# All must pass
cargo check --all-targets
cargo build
cargo test --all
cargo clippy -- -D warnings
cargo fmt --check

# Cross-compile check (at minimum)
cargo check --target x86_64-unknown-linux-musl
cargo check --target x86_64-pc-windows-msvc
```

If any gate fails, the agent must fix the issue before handing off — **never hand off broken code**.

---

## GitHub Push Gate — NON-NEGOTIABLE

This is enforced as hard as any compile error.

### After EVERY single file written or modified:

```bash
git add <that specific file>
git commit -m "type(scope): describe what this file does"
git push origin main
```

**One file = one commit = one push. Always.**

### Remote Identity Check

Before any push, verify the remote is correct:

```bash
git remote -v
```

Expected output — **this exact URL, this exact username**:
```
origin  https://github.com/syedzahidsaleem/sloth-tui.git (fetch)
origin  https://github.com/syedzahidsaleem/sloth-tui.git (push)
```

If the remote shows **any other URL or any other username** (especially `syedzahid0307`), stop and correct it immediately:

```bash
git remote set-url origin https://github.com/syedzahidsaleem/sloth-tui.git
git remote -v  # verify again before pushing
```

### The Only Permitted GitHub Username

`syedzahidsaleem` — and only `syedzahidsaleem`.

- ✅ `https://github.com/syedzahidsaleem/sloth-tui.git`
- ❌ `https://github.com/syedzahid0307/sloth-tui.git` — **FORBIDDEN, never use this**
- ❌ Any other username

This applies to: git remotes, CI YAML files (`github.com/syedzahidsaleem/...`), GitHub Actions, crates.io metadata, README badges, and any hyperlinks inside any file in the repository.

