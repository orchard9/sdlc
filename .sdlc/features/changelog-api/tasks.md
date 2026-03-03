# Tasks: changelog-api

## T1 — Create `crates/sdlc-server/src/routes/changelog.rs`

Implement `get_changelog` handler:
- Accept `Query<ChangelogQuery>` with optional `since` (ISO 8601 string) and `limit` (usize, default 100).
- Parse `since` on the async thread; return `AppError` (which maps to a meaningful HTTP response) on parse failure.
- Delegate to `sdlc_core::event_log::query_events(&root, since, limit)` inside `tokio::task::spawn_blocking`.
- Return `Json({ "events": [...], "total": N })`.
- When `changelog.yaml` does not exist, `query_events` returns empty vec → return `{ "events": [], "total": 0 }`.

## T2 — Register route in `crates/sdlc-server/src/routes/mod.rs`

Add `pub mod changelog;` to mod.rs.

## T3 — Wire route in `crates/sdlc-server/src/lib.rs`

In `build_router_from_state`, add:
```rust
.route("/api/changelog", get(routes::changelog::get_changelog))
```
Place it near the state/backlog routes block, before the UI fallback.

## T4 — Create `frontend/src/hooks/useChangelog.ts`

Implement the `useChangelog(opts?)` hook:
- Fetch `GET /api/changelog` with `since`/`limit` query params.
- Return `{ events, total, loading, error }`.
- Check for an existing shared SSE subscription pattern (`useSSE` or equivalent) in the codebase; if present, use it. Otherwise open an `EventSource` for `/api/events` directly.
- Re-fetch on `ChangelogUpdated` SSE event (event type `update`, `msg.type === 'ChangelogUpdated'`).
- Clear `error` on successful re-fetch.
