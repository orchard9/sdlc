# QA Plan: changelog-api

## Scope

Verify that `GET /api/changelog` behaves correctly across the full parameter matrix, that the `useChangelog` hook re-fetches on SSE events, and that the build passes without warnings.

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` compiles with zero errors and zero warnings.
- [ ] `cargo clippy --all -- -D warnings` produces no warnings.
- [ ] `cd frontend && npm ci && npm run build` produces no TypeScript errors.

## TC-1: Happy path — no parameters

```
curl http://localhost:4444/api/changelog
```
Expected: `200 OK`, `{ "events": [], "total": 0 }` (empty when changelog.yaml absent).

## TC-2: Happy path — with existing events

Precondition: `changelog.yaml` contains ≥ 3 events.

```
curl http://localhost:4444/api/changelog
```
Expected: `200 OK`, `events` array with all events, `total` matches count.

## TC-3: `limit` parameter

```
curl http://localhost:4444/api/changelog?limit=2
```
Expected: `200 OK`, `events` array has at most 2 entries, `total <= 2`.

## TC-4: `since` parameter

```
curl "http://localhost:4444/api/changelog?since=2026-01-01T00:00:00Z"
```
Expected: `200 OK`, all returned events have `timestamp >= 2026-01-01T00:00:00Z`.

## TC-5: Invalid `since`

```
curl "http://localhost:4444/api/changelog?since=notadate"
```
Expected: `400 Bad Request`, body contains `"error"` key.

## TC-6: Invalid `limit`

```
curl "http://localhost:4444/api/changelog?limit=abc"
```
Expected: `400 Bad Request` or graceful fallback to default (document which behavior is implemented).

## TC-7: Response shape

Verify each event in the response contains at minimum: `id`, `kind`, `timestamp`. Verify `total` equals `events.length`.

## TC-8: `useChangelog` hook — SSE re-fetch (manual browser test)

1. Open the frontend application in a browser.
2. Mount any component that uses `useChangelog()`.
3. Append an event to `changelog.yaml` via CLI or curl.
4. Observe: the hook re-fetches and the UI updates without a page reload.

## TC-9: Unit tests (if applicable)

Run `SDLC_NO_NPM=1 cargo test --all` and verify no new test failures are introduced.

## Pass Criteria

All TC-1 through TC-9 pass with no regressions. Build and clippy are clean.
