# Spec: changelog-api

## Summary

Expose a REST endpoint `GET /api/changelog` that queries the changelog event log (written by `changelog-core`) and returns paginated, filtered events to API consumers. Also provide a `useChangelog` frontend hook that fetches the endpoint and re-fetches on `ChangelogUpdated` SSE events.

## Problem

The `changelog-core` feature appends events to `.sdlc/changelog.yaml`. Without a REST endpoint, neither the frontend nor external tools can query that event log. A dashboard banner, activity feed, or any UI component needs a stable API surface to read changelog events filtered by time range and capped at a configurable limit.

## Goals

1. `GET /api/changelog?since=<ISO8601>&limit=<N>` returns `{ events: [...], total: N }`.
2. `since` and `limit` are both optional query parameters with sensible defaults.
3. Events are returned in chronological order (oldest first within the requested window).
4. The hook `useChangelog` fetches `/api/changelog` and re-fetches whenever the `ChangelogUpdated` SSE event fires.
5. No new persistent state is introduced — the endpoint reads from the existing `changelog.yaml` managed by `changelog-core`.

## Non-Goals

- Writing or mutating changelog events (append-only is enforced by `changelog-core`).
- Pagination with cursors — `since` + `limit` is sufficient for v1.
- Authentication beyond what the existing `sdlc-server` auth middleware already provides.

## API Contract

### `GET /api/changelog`

Query parameters:

| Parameter | Type | Default | Description |
|---|---|---|---|
| `since` | ISO 8601 UTC string | None (all events) | Return only events with `timestamp >= since` |
| `limit` | unsigned integer | 100 | Maximum number of events to return |

Response (200 OK):

```json
{
  "events": [
    {
      "id": "...",
      "kind": "feature_merged",
      "slug": "my-feature",
      "timestamp": "2026-03-02T23:00:00Z",
      "metadata": {}
    }
  ],
  "total": 1
}
```

- `events` is always present; empty array when no events match.
- `total` reflects the count of events returned (not the full changelog size).
- Events are ordered by `timestamp` ascending.

### Error cases

- Invalid `since` value: `400 Bad Request` with `{ "error": "invalid since timestamp" }`.
- Invalid `limit` value (non-integer, negative): `400 Bad Request`.
- Changelog file does not exist yet: `200 OK` with `{ "events": [], "total": 0 }`.

## Frontend Hook: `useChangelog`

File: `frontend/src/hooks/useChangelog.ts`

```typescript
interface UseChangelogOptions {
  since?: string;   // ISO 8601 UTC
  limit?: number;   // default 100
}

interface ChangelogResult {
  events: ChangelogEvent[];
  total: number;
  loading: boolean;
  error: string | null;
}

function useChangelog(opts?: UseChangelogOptions): ChangelogResult
```

- On mount, fetches `GET /api/changelog` with the provided query params.
- Subscribes to the `ChangelogUpdated` SSE event via the existing `useSSE` hook (or equivalent) and re-fetches on each event.
- Returns `loading: true` while the initial fetch is in flight.
- Returns `error` string if the fetch fails; clears on successful re-fetch.

## Implementation Path

1. `crates/sdlc-server/src/routes/changelog.rs` — implement `get_changelog` handler.
2. `crates/sdlc-server/src/routes/mod.rs` — declare `pub mod changelog` and wire the route.
3. `crates/sdlc-server/src/main.rs` (or wherever routes are registered) — add `.route("/api/changelog", get(changelog::get_changelog))`.
4. `frontend/src/hooks/useChangelog.ts` — implement the hook.

## Dependencies

- `changelog-core` must be implemented first; this feature calls `sdlc_core::event_log::query_events()`.
- No new crate dependencies required.

## Acceptance Criteria

- `GET /api/changelog` returns 200 with empty events when changelog does not exist.
- `GET /api/changelog?limit=5` returns at most 5 events.
- `GET /api/changelog?since=<ts>` returns only events at or after `ts`.
- `GET /api/changelog?since=notadate` returns 400.
- `useChangelog` hook re-fetches after a `ChangelogUpdated` SSE event fires.
