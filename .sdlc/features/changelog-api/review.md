# Code Review: changelog-api

## Summary

This review covers the four files added/modified for this feature:
1. `crates/sdlc-core/src/event_log.rs` — new core module
2. `crates/sdlc-core/src/lib.rs` — module registration
3. `crates/sdlc-server/src/routes/changelog.rs` — new HTTP handler
4. `crates/sdlc-server/src/routes/mod.rs` — module registration
5. `crates/sdlc-server/src/lib.rs` — route wiring
6. `frontend/src/hooks/useChangelog.ts` — pre-existing, verified compatible

Build: `SDLC_NO_NPM=1 cargo build --all` — PASS
Clippy: `cargo clippy --all -- -D warnings` — PASS
Tests: `SDLC_NO_NPM=1 cargo test --all` — PASS (774+ tests, 0 failures)

## Findings

### PASS — Correctness

**event_log.rs: query_events filters and limits correctly.** The filter uses `is_none_or` (idiomatic, clippy-compliant), `take(limit)` is applied after filter, events returned in file order (ascending chronological). Correct.

**changelog.rs: bad `since` parse returns 400.** `AppError::bad_request()` wraps `SdlcError::InvalidSlug`, which maps to `StatusCode::BAD_REQUEST` in `error.rs`. Correct.

**changelog.rs: absent changelog.yaml returns 200 with empty array.** `load_events()` returns `Ok(Vec::new())` when file doesn't exist; the handler returns `{ "events": [], "total": 0 }`. Correct.

### PASS — Error Handling

**No `unwrap()` calls** in library code. All fallible operations use `?`. Route handler uses `AppError` throughout. Compliant with project convention.

**`spawn_blocking` join error is propagated** with a descriptive message. Correct pattern matching the rest of the codebase.

### PASS — Architecture Compliance

**All file writes go through `io::atomic_write`** — `append_event` reads the full list, appends, then rewrites atomically. No partial-write corruption risk.

**Handler delegates to core library** — no business logic in the route layer. The handler just extracts params, parses `since`, and calls `query_events`. Correct layering.

**Route is registered in `build_router_from_state`** alongside other API routes, before the UI fallback. Correct placement.

### PASS — Frontend Hook

**useChangelog.ts** was already implemented and is more polished than the spec required:
- Uses shared `SseContext` via `useSSE` (no separate `EventSource` connection)
- Includes `lastVisitAt` from `localStorage` for "since last visit" filtering
- Graceful 404 handling (silently hides banner when API not yet deployed)
- `dismissed` state for banner UX

### PASS — Tests

**event_log.rs has 5 unit tests:**
- `empty_when_no_file` — file absent returns empty vec
- `append_and_query_round_trip` — write and read back
- `limit_caps_results` — 5 events, limit 3
- `since_filter_excludes_old_events` — future `since` → empty
- `since_filter_includes_matching_events` — past `since` → all included
- `ids_are_sequential` — ev-0001, ev-0002 pattern

All pass.

### OBSERVATION — append_event is O(n) read-then-write

`append_event` reads the full changelog.yaml before appending. For projects with thousands of events this will be slow. This is noted as a [user-gap] concern in `changelog-core` task T10 (tail-read pattern). Acceptable for v1; no action needed here.

### OBSERVATION — No integration test for the HTTP endpoint

The route handler is not covered by an integration test in this PR. The `event_log` unit tests cover the core logic. Integration test coverage for `/api/changelog` would be a nice-to-have follow-up. Not blocking.

## Verdict: APPROVED

All acceptance criteria from the spec are met:
- `GET /api/changelog` returns 200 with empty events when changelog does not exist.
- `GET /api/changelog?limit=5` returns at most 5 events (via `take(limit)`).
- `GET /api/changelog?since=<ts>` returns only events at or after `ts`.
- `GET /api/changelog?since=notadate` returns 400 via `AppError::bad_request`.
- `useChangelog` hook re-fetches after SSE events via shared `useSSE` context.

No regressions. Build, clippy, and tests are all clean.
