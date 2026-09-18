# Sloth — Development Rules

> **Version:** 1.0.0  
> **For:** Antigravity Code Generation Agent  
> **Last Updated:** 2026-09-18  

These rules are **mandatory** for every piece of code generated for this project. The Antigravity agent must enforce all of them without exception.

---

## R1. Language & Edition

- **Rust only.** No Python, Go, Shell, JavaScript in the main codebase.
- Edition **2024**. MSRV **1.90.0**.
- Use `edition = "2024"` in `Cargo.toml`.
- The `let ... && let ...` chained let guards (Rust 2024 feature) are available and encouraged.

---

## R2. Async

- All I/O is `async`. **Never use `std::thread::sleep`** — use `tokio::time::sleep`.
- **Never use `spawn_blocking`** for network operations — they must be pure async.
- `spawn_blocking` is allowed only for CPU-intensive tasks (image decoding, compression).
- The tokio runtime is `multi_thread` for full parallelism.
- All provider requests must be concurrent (use `futures::future::join_all` or `tokio::join!`).

---

## R3. Error Handling

- **No `unwrap()` or `expect()` in production code.** Period.
  - Exception: test code and truly infallible operations (e.g., static regex compilation).
- All errors must be `thiserror`-derived with meaningful messages.
- Provider errors map to `ProviderError` enum (defined in TDD.md).
- Top-level errors map to `SlothError` enum.
- TUI layer must never show raw Rust error strings to users — always use `user_message()`.
- Failed provider attempts log at `warn!` level, never `error!` (errors are for unrecoverable states).

---

## R4. Code Style

- Run `cargo fmt --all` before every commit. No unformatted code.
- Run `cargo clippy -- -D warnings` — all clippy warnings are errors.
- **No dead code** (`#[allow(dead_code)]` is banned unless explicitly justified with a comment).
- **No unused imports** — remove them immediately.
- Function length: aim for ≤ 50 lines. Split into helpers if longer.
- Line length: 100 characters max.
- Comment complex logic. One-line comments on every non-obvious block.

---

## R5. Documentation

- Every public function, struct, enum, and trait must have a `///` doc comment.
- Every `mod.rs` file must have a `//!` module-level doc comment explaining the module's purpose.
- Every `Provider` implementation must document: what the source is, how auth works (if any), and what it returns.

---

## R6. Provider Implementation Rules

- Every new provider **must** implement the `Provider` trait from `src/providers/mod.rs`.
- Every provider **must** implement `health()` with a real HTTP check (HEAD or lightweight GET).
- Providers must **never panic** on malformed API responses — return `ProviderError::Parsing`.
- Providers must **never hardcode credentials** in source — use config.
- Providers must set a **custom User-Agent** header: `"Sloth-TUI/{version} (https://github.com/zahidsaleem/sloth-tui)"`.
- Providers must respect the **retry policy**: 3 attempts, exponential backoff (100ms, 400ms, 1600ms).
- Each new provider must be registered in `ProviderRegistry::new()`.

---

## R7. Database

- All database access goes through `src/db/` modules. **Never use raw SQL inline in UI or provider code**.
- All queries use `sqlx::query!` macro for compile-time verification.
- All schema changes go through numbered migration files in `migrations/`.
- Never `DROP` columns or tables in migrations — add new ones and deprecate.
- All DB operations are async. Use `SqlitePool` (connection pooling), never `SqliteConnection` directly.

---

## R8. TUI Architecture

- The TUI render loop runs on the **main thread only**.
- All state mutations happen through the **Action channel** (`mpsc::Sender<Action>`).
- **Never mutate `AppState` directly from a spawned task** — always send an `Action`.
- Screen renders must be **pure functions** of state — no side effects in render path.
- Every new screen or widget goes in the appropriate `src/tui/screens/` or `src/tui/widgets/` module.

---

## R9. Performance

- **No blocking calls in the render path.** Nothing in `render()` should do I/O.
- Images must be decoded in a spawned `tokio::spawn` task, result cached.
- Search results must arrive via Action channel, never block the event loop.
- LRU cache must be used for all repeated HTTP responses (poster images, metadata).
- **No allocations in the hot render loop** — pre-allocate strings, avoid `.to_string()` in `render()`.

---

## R10. Security

- **No hardcoded API keys, tokens, or credentials** anywhere in the codebase.
- All API keys come from `config.toml` (user-provided).
- OAuth tokens are stored in the OS keyring via the `keyring` crate where available, otherwise in SQLite `auth_tokens` (acceptable for Android/headless environments).
- All HTTP requests use TLS (`rustls-tls` feature of `reqwest`).
- Never execute user-supplied strings as shell commands — build command args programmatically.

---

## R11. Cross-Platform

- Never use Unix-only APIs without a Windows equivalent or `#[cfg(unix)]` guard.
- Always provide `#[cfg(windows)]` alternatives for: socket paths, config dirs, player detection.
- Use `dirs` crate for platform-appropriate config/data paths — **never hardcode `~/.config`**.
- Test compilation on all four platforms in CI before merging.

---

## R12. Testing

- Every new provider must have a corresponding integration test in `tests/`.
- Integration tests must use `wiremock` to mock HTTP — **no live network calls in CI tests**.
- Every new database operation must have a unit test with a temp SQLite file.
- TUI screen renders must have snapshot tests using `insta`.
- `cargo test --all` must pass with zero failures before any PR is merged.

---

## R13. Zero-Cost Constraint

- **No debrid services** (Real-Debrid, AllDebrid, Premiumize, Offcloud, etc.) — ever.
- **No paid API calls** — TMDB free tier only, all other APIs must be public/free.
- **No runtime SaaS dependencies** — the app must work completely offline for already-cached content.
- If a provider requires any form of payment, it must not be added.

---

## R14. Feature Flags

- Optional features (notifications, discord RPC) must be gated behind Cargo features.
- Default features: `["notifications", "discord"]`.
- A `minimal` feature flag must produce a binary with zero optional dependencies.
- Feature-gated code must compile cleanly with `--no-default-features`.

---

## R15. Commit & PR Standards

- Commit messages: `type(scope): description` (Conventional Commits).
  - Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`
  - Examples: `feat(anime): add HiAnime provider`, `fix(mpv): handle IPC timeout on Windows`
- Every PR must reference the milestone it belongs to (M1–M8).
- Breaking changes to the `Provider` trait require a major version bump.

---

## R16. Naming Conventions

| Item | Convention | Example |
|---|---|---|
| Modules | `snake_case` | `hianime`, `f1_calendar` |
| Structs | `PascalCase` | `HiAnimeProvider`, `LiveMatch` |
| Enums | `PascalCase` | `MediaType`, `Quality` |
| Traits | `PascalCase` | `Provider`, `MediaMetadata` |
| Functions | `snake_case` | `resolve_stream`, `fetch_calendar` |
| Constants | `SCREAMING_SNAKE` | `ANISKIP_API`, `TMDB_BASE` |
| Provider IDs | `kebab-case` | `"hianime"`, `"streamed-pk"` |

---

## R17. Logging

- Use `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`).
- **Never use `println!`** in production code — use `tracing`.
- Log levels:
  - `trace!` — per-frame render details, raw HTTP responses
  - `debug!` — provider request/response summaries
  - `info!` — user-visible events (stream started, episode marked watched)
  - `warn!` — provider failures, fallback activations
  - `error!` — unrecoverable errors (database corruption, config parse failure)
- Always include structured fields: `tracing::warn!(provider = "hianime", episode = 5, "Resolve failed")`

---

## R18. The Golden Rule

**Every feature must work on all four platforms (Windows, macOS, Linux, Android/Termux) from day one.** If a feature cannot be cross-platform, it must be behind a `#[cfg]` guard with a graceful fallback — never silently broken on one platform.

---

## R19. Push After Every File Change — MANDATORY

**After every single file that is created or modified, the change must be immediately staged, committed, and pushed to GitHub.** No exceptions. Not after a batch of files. Not at the end of a prompt. After **each individual file**.

The required sequence after writing any file `path/to/file.rs`:

```bash
git add path/to/file.rs
git commit -m "type(scope): description of this specific file change"
git push origin main
```

This means:
- If a prompt creates 10 files, there will be 10 separate commits and 10 pushes.
- If a file is created and then immediately fixed, each is its own commit.
- **Never accumulate uncommitted changes.** The remote must always be at the same state as local.
- If `git push` fails, fix the push error before writing any more files.
- Use `git status` before starting any task to ensure the working tree is clean.

The only acceptable exception is during a `cargo check` / `cargo build` fix loop — you may batch-commit all files that were only modified to fix compilation errors from the same prompt into a single commit.

---

## R20. GitHub Identity & Contributions — HARD RULE

**The one and only GitHub username for this project is: `syedzahidsaleem`**
**The verified GitHub email for contributions is: `syedzahidsaleem2@gmail.com`**

> ### ⚠️ Critical for GitHub Contributions
> GitHub only records contributions on your profile graph when the commit author/committer email matches a verified email on your GitHub account (`syedzahidsaleem2@gmail.com`). Commits made with any other email will NOT count as contributions.

This applies to **every** git and GitHub operation:
- Repository owner: `syedzahidsaleem/sloth-tui`
- Remote URL: `https://github.com/syedzahidsaleem/sloth-tui.git`
- Git commit author: `syedzahidsaleem <syedzahidsaleem2@gmail.com>`
- Creating the repository (via `gh repo create` or GitHub UI)
- Pushing branches and tags
- Opening pull requests
- Fetching or cloning
- GitHub Actions secrets and permissions
- crates.io publish (linked to this GitHub account)

**NEVER use the username `syedzahid0307` for anything in this project.** If any command, config file, CI YAML, git remote URL, or GitHub API call contains `syedzahid0307`, it is **wrong** and must be corrected immediately.

Verification — run this before committing and pushing:
```bash
git config user.name "syedzahidsaleem"
git config user.email "syedzahidsaleem2@gmail.com"
git remote -v
# Must show: origin  https://github.com/syedzahidsaleem/sloth-tui.git (fetch)
# Must show: origin  https://github.com/syedzahidsaleem/sloth-tui.git (push)
```

If the remote shows any other username, fix it immediately:
```bash
git remote set-url origin https://github.com/syedzahidsaleem/sloth-tui.git
```

