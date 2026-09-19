# Contributing to Sloth

Zero-cost terminal streaming for movies, anime, sports, F1, and live TV — built in Rust.

Thank you for your interest in contributing. This document covers everything you need to get the project building locally and how to submit quality changes.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building](#building)
- [Running Tests](#running-tests)
- [Code Standards](#code-standards)
- [Commit Convention](#commit-convention)
- [Branch Naming](#branch-naming)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Project References](#project-references)

---

## Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust (stable) | >= 1.90.0 | Compiler (MSRV set in `Cargo.toml`) |
| cargo | Bundled with Rust | Build tool |
| sqlx-cli | Any recent | Only for schema migrations -- **not** required for building |

Install Rust via [rustup](https://rustup.rs/). The project's `Cargo.toml` declares `rust-version = "1.90.0"`, so any stable toolchain at or above that version works.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install `sqlx-cli` only if you need to add or apply database migrations:

```sh
cargo install sqlx-cli --no-default-features --features native-tls,sqlite
```

---

## Building

Clone the repository and run:

```sh
git clone https://github.com/syedzahidsaleem/sloth-tui.git
cd sloth-tui
cargo check            # fast type and borrow-check pass
cargo build            # full debug build
cargo build --release  # optimised release binary
```

The project uses [sqlx](https://github.com/launchbakery/sqlx) with **offline query verification**. The `.sqlx/` directory containing pre-compiled query metadata is committed to the repository, so `DATABASE_URL` is **not** required to build or check the project.

### Active database development

If you are adding or modifying SQL queries, you need a live database connection so sqlx can verify your queries at compile time:

```sh
# 1. Create the database file
sqlx database create

# 2. Apply all migrations
sqlx migrate run

# 3. Regenerate the offline query cache after changing any sqlx::query! call
cargo sqlx prepare
```

The `.env` file is gitignored. Create it locally when needed:

```
DATABASE_URL=sqlite://sloth.db
```

---

## Running Tests

```sh
cargo test --all           # run all unit and integration tests
cargo test --lib           # unit tests only (no network, no filesystem)
cargo test --test <name>   # specific integration test file in tests/
```

Integration tests use [wiremock](https://github.com/LukeMathWalker/wiremock-rs) to mock HTTP -- no live network calls are made. Database tests create isolated temporary SQLite files via the `tempfile` crate, so they are safe to run anywhere.

---

## Code Standards

All of the following must pass before a PR can be merged:

```sh
cargo fmt --all                            # format all code
cargo clippy --all-targets -- -D warnings  # all warnings treated as errors
cargo check --all-targets                  # full type-check including tests and examples
cargo test --all                           # all tests pass
```

**Rules enforced in every PR:**

- **No `unwrap()` or `expect()`** in production code. Use `?`, `map_err`, or explicit match.
- **No `println!`** in production code. Use the `tracing` macros (`debug!`, `info!`, `warn!`, `error!`).
- Every public item (function, struct, enum, trait) must have a `///` doc comment.
- Every `mod.rs` must have a `//!` module-level doc comment.
- Line length: 100 characters maximum.
- Function length: aim for <= 50 lines.

The full list of enforced rules is in [`docs/RULES.md`](docs/RULES.md).

---

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

```
type(scope): short description
```

**Types:**

| Type | Use for |
|---|---|
| `feat` | New functionality |
| `fix` | Bug fixes |
| `refactor` | Internal restructuring without behaviour change |
| `docs` | Documentation only |
| `test` | New or updated tests |
| `chore` | Build config, dependencies, tooling |
| `perf` | Performance improvements |

**Examples from this repository:**

```
feat(anime): add HiAnime provider
fix(mpv): handle IPC timeout on Windows
docs(contributing): add contributor guide
test(f1): add fixture parsing unit tests
chore(cargo): update sqlx to 0.8
```

Scope should match the affected module path (e.g., `player`, `tui/screens`, `providers/moviebox`).

---

## Branch Naming

```
<type>/<short-description>
```

Examples:

```
feat/anilist-auth
fix/mpv-ipc-windows
docs/contributing-guide
ci/github-actions
test/sports-pipeline
```

---

## Pull Request Guidelines

- **One logical change per PR.** Do not bundle unrelated fixes.
- Reference the milestone the change belongs to if applicable (M1-M8 from [`docs/PROMPTS.md`](docs/PROMPTS.md)).
- Ensure `cargo fmt --all`, `cargo clippy -- -D warnings`, and `cargo test --all` all pass locally before opening a PR.
- Breaking changes to the `Provider` trait require a major version bump.
- PRs that add new providers must include integration tests using `wiremock`.

---

## Project References

| Document | Purpose |
|---|---|
| [`docs/RULES.md`](docs/RULES.md) | Mandatory coding rules for all contributors |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System design and module overview |
| [`docs/TDD.md`](docs/TDD.md) | Type definitions and data model |
| [`docs/SCHEMA.md`](docs/SCHEMA.md) | SQLite schema documentation |
| [`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md) | Dependency rationale and versions |
| [`docs/PROMPTS.md`](docs/PROMPTS.md) | Development milestones (M1-M8) |
