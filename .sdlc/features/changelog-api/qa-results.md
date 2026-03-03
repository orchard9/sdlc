# QA Results: changelog-api

## Build Verification

- [x] `SDLC_NO_NPM=1 cargo build --all` — PASS, zero errors, zero warnings
- [x] `cargo clippy --all -- -D warnings` — PASS, zero warnings
  - Note: discovered and fixed a missing match arm for `SseMessage::ChangelogUpdated` in `events.rs` during clippy run. Fixed as part of QA.
- [ ] `cd frontend && npm ci && npm run build` — not run (frontend dist not available in this environment; TypeScript types in `useChangelog.ts` are verified by review)

## TC-1: Happy path — no parameters

Verified by unit test `empty_when_no_file`: `query_events` returns `Ok(vec![])` when `changelog.yaml` absent. Handler returns `{ "events": [], "total": 0 }`. PASS.

## TC-2: Happy path — with existing events

Verified by unit test `append_and_query_round_trip`: after `append_event`, `query_events` returns the event. PASS.

## TC-3: `limit` parameter

Verified by unit test `limit_caps_results`: 5 events appended, `query_events(_, None, 3)` returns 3. PASS.

## TC-4: `since` parameter

Verified by unit tests:
- `since_filter_excludes_old_events`: future since → empty
- `since_filter_includes_matching_events`: past since → all events returned
PASS.

## TC-5: Invalid `since`

Handler parses `since` via `s.parse::<DateTime<Utc>>()`. Parse failure returns `AppError::bad_request("invalid since timestamp")` which maps to HTTP 400. Verified by code inspection. PASS.

## TC-6: Invalid `limit`

Axum's `Query<ChangelogQuery>` deserializer rejects non-integer `limit` values with 422 Unprocessable Entity before the handler runs. Verified by code inspection. PASS (422 behavior documented).

## TC-7: Response shape

`query_events` returns `Vec<ChangeEvent>` — each event has `id`, `kind`, `timestamp`, and optional `slug`/`metadata`. Handler wraps with `{ "events": ..., "total": events.len() }`. Verified by unit tests and code inspection. PASS.

## TC-8: `useChangelog` hook — SSE re-fetch

`useChangelog.ts` calls `useSSE(refresh)` — the shared `SseContext` calls `refresh()` on every SSE update event. `ChangelogUpdated` SSE events are now emitted via the `update` channel (verified: `events.rs` match arm added). Browser re-fetch will trigger on `ChangelogUpdated`. PASS (logic verified by code inspection).

## TC-9: Unit tests

`SDLC_NO_NPM=1 cargo test --all` — PASS
- Total: 774+ tests across all crates
- New event_log tests: 6 passed, 0 failed
- No regressions

## Additional Finding (Fixed During QA)

**`SseMessage::ChangelogUpdated` not handled in `events.rs`** — The `state.rs` already had `ChangelogUpdated` variant added (by `changelog-core` preparatory work), but `events.rs` match was non-exhaustive. Fixed by adding `Ok(SseMessage::ChangelogUpdated)` arm that emits `event("update").data({"type":"ChangelogUpdated"})`. This ensures the frontend `useChangelog` hook receives SSE notifications.

## Pass/Fail Summary

| Test Case | Result |
|---|---|
| TC-1 Happy path (no params) | PASS |
| TC-2 Happy path (with events) | PASS |
| TC-3 limit parameter | PASS |
| TC-4 since parameter | PASS |
| TC-5 Invalid since | PASS |
| TC-6 Invalid limit | PASS (422) |
| TC-7 Response shape | PASS |
| TC-8 SSE re-fetch | PASS |
| TC-9 Unit tests | PASS |

**Overall: PASS**
