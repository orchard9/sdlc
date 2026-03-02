# Code Review: knowledge-cli-ingest

## Summary

Implemented the full CLI and REST API surface for the knowledge base. All spec requirements are satisfied.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-cli/Cargo.toml` | Added `ureq = "2"` dependency |
| `crates/sdlc-cli/src/cmd/mod.rs` | Added `pub mod knowledge;` |
| `crates/sdlc-cli/src/cmd/knowledge.rs` | **Created** — full CLI module (~370 lines) |
| `crates/sdlc-cli/src/main.rs` | Added `Knowledge` variant + import + handler dispatch |
| `crates/sdlc-server/src/routes/mod.rs` | Added `pub mod knowledge;` |
| `crates/sdlc-server/src/routes/knowledge.rs` | **Created** — 8 REST handlers (~310 lines) |
| `crates/sdlc-server/src/lib.rs` | Registered 6 knowledge route groups |
| `crates/sdlc-server/src/error.rs` | Added `BacklogItemNotFound` to status match (pre-existing missing arm) |

## CLI Module (knowledge.rs)

Follows `investigate.rs` exactly:
- `KnowledgeSubcommand` enum with all 8 subcommands
- `run()` dispatcher with idiomatic `match` arms
- Private handler functions: `status`, `add`, `list`, `show`, `search`, `update`, `catalog_show`, `catalog_add`, `session_log`, `session_list`, `session_read`
- `slugify_title()` — pure function, no regex dependency
- `fetch_page_title()` — best-effort via ureq, never fails the command
- `entry_to_json_summary()` / `entry_to_json_full()` JSON helpers

### Key behaviors verified

- `--from-url`: fetches page title via ureq 10s timeout, stores `Source { source_type: Web, url }`, sets `origin: Web`. Failure is silent (best-effort per spec).
- `--from-file`: reads file content, stores `Source { source_type: LocalFile, path }`.
- `--content`: writes inline text. Origin stays Manual (create() default).
- `list` without entries → prints `EMPTY_STATE_MSG`.
- `search` with no results in empty base → prints `EMPTY_STATE_MSG`.
- `catalog add` — counts dots: zero → `add_class`, one → `add_division` with correct parent code extraction.
- `slugify_title` — lowercase, non-alnum→`-`, dedup, strip trailing, truncate at 40. No regex dep.

## Server Routes (knowledge.rs)

Follows `investigations.rs` exactly:
- All handlers are `pub async fn` with `State<AppState>`
- All blocking I/O wrapped in `tokio::task::spawn_blocking`
- Query params for `list_knowledge` via `axum::extract::Query<ListKnowledgeQuery>` (typed struct)
- All errors return `AppError`
- `slugify_title_server` is a private local copy (avoids cross-crate pub fn for simple util)

## Correctness

- Slug-only directories: never relies on code in the path (matches `knowledge-core-data` invariant)
- No clippy warnings
- All tests pass (SDLC_NO_NPM=1 cargo test --all → all green)

## Verdict

APPROVED — implementation is complete, correct, and follows established patterns.
