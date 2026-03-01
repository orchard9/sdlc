# Design: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Architecture Overview

This feature is a thin HTTP and SSE surface over the existing `sdlc-core` UAT persistence layer. There is no new domain logic — the design concern is routing, serialization, and event wiring.

```
Frontend
  │
  ├── GET /api/milestones/{slug}/uat-runs
  │     → milestones::list_milestone_uat_runs()
  │     → sdlc_core::milestone::list_uat_runs(root, slug)
  │     ← Vec<UatRun> as JSON
  │
  ├── GET /api/milestones/{slug}/uat-runs/latest
  │     → milestones::get_latest_milestone_uat_run()
  │     → sdlc_core::milestone::latest_uat_run(root, slug)
  │     ← UatRun as JSON   (or 404 if None)
  │
  └── /api/events (SSE stream)
        ← MilestoneUatCompleted { slug }  (emitted by spawn_agent_run on completion)
```

## Route Handlers

Both handlers live in `crates/sdlc-server/src/routes/milestones.rs` alongside the existing milestone handlers, keeping milestone-related routes co-located.

### `list_milestone_uat_runs`

```
GET /api/milestones/{slug}/uat-runs
```

Pattern: identical to `list_milestones` and `get_milestone`.

1. Extract `Path(slug)`.
2. Clone `app.root`.
3. `tokio::task::spawn_blocking`: call `sdlc_core::milestone::list_uat_runs(&root, &slug)`.
4. Return `Json(runs)` where `runs: Vec<UatRun>`.
5. On error, propagate via `AppError`.

Empty list (no runs yet) → 200 with `[]`.

### `get_latest_milestone_uat_run`

```
GET /api/milestones/{slug}/uat-runs/latest
```

1. Extract `Path(slug)`.
2. Clone `app.root`.
3. `tokio::task::spawn_blocking`: call `sdlc_core::milestone::latest_uat_run(&root, &slug)`.
4. If `Ok(Some(run))` → `Json(run)`.
5. If `Ok(None)` → `AppError::not_found("no UAT runs for this milestone")`.
6. If `Err(e)` → propagate `AppError`.

## Route Registration

In `crates/sdlc-server/src/lib.rs`, inside `build_router_from_state`:

```rust
.route(
    "/api/milestones/{slug}/uat-runs",
    get(routes::milestones::list_milestone_uat_runs),
)
.route(
    "/api/milestones/{slug}/uat-runs/latest",
    get(routes::milestones::get_latest_milestone_uat_run),
)
```

Both routes are added after the existing milestone routes.

## SSE Variant

### Enum addition (state.rs)

```rust
/// A milestone UAT agent run completed — UatRun record may have been written.
MilestoneUatCompleted { slug: String },
```

The `SseMessage` enum is used via the `events.rs` SSE serializer that pattern-matches all variants. The serializer needs a new arm for `MilestoneUatCompleted`.

### events.rs serialization arm

Following the pattern of `PonderRunCompleted`, `ToolBuildCompleted`:

```rust
SseMessage::MilestoneUatCompleted { slug } => {
    serde_json::json!({ "type": "MilestoneUatCompleted", "slug": slug })
}
```

### Emission in runs.rs

Change the last argument of `spawn_agent_run` in `start_milestone_uat` from `None` to:

```rust
Some(SseMessage::MilestoneUatCompleted { slug: slug.clone() })
```

The slug must be cloned before the `format!` call that moves it into the prompt string, or restructured to ensure the slug is still accessible at the `spawn_agent_run` call site.

## Frontend Types (types.ts)

```typescript
export type UatVerdict = 'pass' | 'pass_with_tasks' | 'failed'

export interface UatRun {
  id: string
  milestone_slug: string
  started_at: string       // ISO 8601
  completed_at: string | null
  verdict: UatVerdict
  tests_total: number
  tests_passed: number
  tests_failed: number
  playwright_report_path: string | null
  tasks_created: string[]
  summary_path: string
}
```

The `UatVerdict` string values match the `#[serde(rename_all = "snake_case")]` on the Rust enum.

## Frontend API Client (client.ts)

Added to the `api` object, grouped with other milestone methods:

```typescript
listMilestoneUatRuns: (slug: string) =>
  request<UatRun[]>(`/api/milestones/${encodeURIComponent(slug)}/uat-runs`),
getLatestMilestoneUatRun: (slug: string) =>
  request<UatRun | null>(`/api/milestones/${encodeURIComponent(slug)}/uat-runs/latest`),
```

`getLatestMilestoneUatRun` returns `UatRun | null` in TypeScript because the caller must handle the 404 case gracefully (it may catch and return null, or the caller handles the thrown error).

## Error Handling

- `AppError::not_found` is used for the `latest` endpoint when no run exists.
- All other `SdlcError` variants propagate as 500 via `AppError`.
- No `unwrap()` anywhere; all fallible calls use `?`.

## No Breaking Changes

- No existing routes are modified.
- No existing SSE variants are changed.
- The new SSE variant is additive and opt-in for frontend consumers.

## File Change Summary

| File | Change |
|------|--------|
| `crates/sdlc-server/src/routes/milestones.rs` | Add `list_milestone_uat_runs`, `get_latest_milestone_uat_run` |
| `crates/sdlc-server/src/state.rs` | Add `MilestoneUatCompleted { slug }` to `SseMessage` |
| `crates/sdlc-server/src/routes/events.rs` | Add serialization arm for `MilestoneUatCompleted` |
| `crates/sdlc-server/src/routes/runs.rs` | Pass `MilestoneUatCompleted` completion event in `start_milestone_uat` |
| `crates/sdlc-server/src/lib.rs` | Register two new routes |
| `frontend/src/lib/types.ts` | Add `UatVerdict`, `UatRun` |
| `frontend/src/api/client.ts` | Add `listMilestoneUatRuns`, `getLatestMilestoneUatRun` |
