# Design: changelog-api

## Overview

This feature adds one Rust route handler and one React hook. No new data structs are needed — the handler delegates entirely to `sdlc_core::event_log::query_events()` provided by `changelog-core`.

## Architecture

```
Frontend (React)
  └── useChangelog(opts)
        ├── fetch GET /api/changelog?since=<ts>&limit=<n>
        └── subscribe to ChangelogUpdated SSE → re-fetch

sdlc-server (Axum)
  └── GET /api/changelog
        └── routes/changelog.rs :: get_changelog()
              └── tokio::task::spawn_blocking
                    └── sdlc_core::event_log::query_events(root, since, limit)
                          └── reads .sdlc/changelog.yaml
```

## Rust Handler — `crates/sdlc-server/src/routes/changelog.rs`

```rust
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChangelogQuery {
    pub since: Option<String>,   // ISO 8601 UTC
    pub limit: Option<usize>,
}

/// GET /api/changelog — return changelog events filtered by since/limit.
pub async fn get_changelog(
    Query(params): Query<ChangelogQuery>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Parse `since` upfront on the async thread to return 400 quickly.
    let since: Option<DateTime<Utc>> = match params.since {
        None => None,
        Some(s) => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|_| AppError(anyhow::anyhow!("invalid since timestamp")))?,
        ),
    };
    let limit = params.limit.unwrap_or(100);
    let root = app.root.clone();

    let events = tokio::task::spawn_blocking(move || {
        sdlc_core::event_log::query_events(&root, since, limit)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    let total = events.len();
    Ok(Json(serde_json::json!({ "events": events, "total": total })))
}
```

Key decisions:
- `since` parsing happens on the async thread before `spawn_blocking` — bad parse returns 400 without blocking a thread.
- Default `limit` is 100 — generous enough for a dashboard feed, small enough to be safe.
- The handler returns `AppError`, which the existing `error.rs` converts to a JSON error response.

## Route Registration — `crates/sdlc-server/src/lib.rs`

Add `pub mod changelog;` to `routes/mod.rs`, then in `build_router_from_state` in `lib.rs`:

```rust
.route("/api/changelog", get(routes::changelog::get_changelog))
```

Placement: insert near the state/backlog routes block (early in the router), before the wildcard UI fallback.

## Frontend Hook — `frontend/src/hooks/useChangelog.ts`

```typescript
import { useEffect, useState, useCallback } from 'react';

export interface ChangelogEvent {
  id: string;
  kind: string;
  slug?: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

interface UseChangelogOptions {
  since?: string;
  limit?: number;
}

interface ChangelogResult {
  events: ChangelogEvent[];
  total: number;
  loading: boolean;
  error: string | null;
}

export function useChangelog(opts?: UseChangelogOptions): ChangelogResult {
  const [state, setState] = useState<ChangelogResult>({
    events: [],
    total: 0,
    loading: true,
    error: null,
  });

  const fetchEvents = useCallback(async () => {
    const params = new URLSearchParams();
    if (opts?.since) params.set('since', opts.since);
    if (opts?.limit != null) params.set('limit', String(opts.limit));
    const url = `/api/changelog${params.size ? `?${params}` : ''}`;

    try {
      const res = await fetch(url);
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        setState(prev => ({
          ...prev,
          loading: false,
          error: body.error ?? `HTTP ${res.status}`,
        }));
        return;
      }
      const data: { events: ChangelogEvent[]; total: number } = await res.json();
      setState({ events: data.events, total: data.total, loading: false, error: null });
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : 'fetch error',
      }));
    }
  }, [opts?.since, opts?.limit]);

  // Initial fetch
  useEffect(() => {
    fetchEvents();
  }, [fetchEvents]);

  // Re-fetch on ChangelogUpdated SSE event
  useEffect(() => {
    const evs = new EventSource('/api/events');
    const handler = (e: MessageEvent) => {
      try {
        const msg = JSON.parse(e.data);
        if (msg.type === 'ChangelogUpdated') fetchEvents();
      } catch { /* ignore parse errors */ }
    };
    evs.addEventListener('update', handler);
    return () => { evs.close(); };
  }, [fetchEvents]);

  return state;
}
```

Note: If the project already has a shared SSE hook (`useSSE`), the second `useEffect` should delegate to it instead of opening a second `EventSource`. The implementation task will check for an existing pattern.

## Error Handling

| Scenario | HTTP status | Response body |
|---|---|---|
| `since` not parseable as ISO 8601 | 400 | `{ "error": "invalid since timestamp" }` |
| `changelog.yaml` absent | 200 | `{ "events": [], "total": 0 }` |
| Internal I/O error | 500 | `{ "error": "..." }` |

The existing `AppError` implementation in `crates/sdlc-server/src/error.rs` already converts `anyhow::Error` to a 500 JSON response. For the 400 case, we return `AppError` with a "invalid since timestamp" message — the error.rs `IntoResponse` impl will need to distinguish 400 vs 500, or we return `(StatusCode::BAD_REQUEST, Json(...))` directly. The task will choose the appropriate pattern matching the existing code.

## Sequence: Dashboard Banner Consuming the Hook

```
1. DashboardBanner mounts → useChangelog({ limit: 10 }) called
2. Hook fetches GET /api/changelog?limit=10
3. Server reads changelog.yaml → returns last 10 events
4. Banner renders event list
5. New event appended to changelog.yaml → mtime watcher fires ChangelogUpdated SSE
6. Hook receives SSE → re-fetches → banner updates
```

## Files Changed

| File | Action |
|---|---|
| `crates/sdlc-server/src/routes/changelog.rs` | Create |
| `crates/sdlc-server/src/routes/mod.rs` | Add `pub mod changelog;` |
| `crates/sdlc-server/src/lib.rs` | Add route binding |
| `frontend/src/hooks/useChangelog.ts` | Create |
