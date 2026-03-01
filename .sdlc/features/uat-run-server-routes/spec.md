# Spec: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Overview

The `uat-run-persistence` feature has already shipped `UatRun`, `UatVerdict`, `save_uat_run`, `list_uat_runs`, and `latest_uat_run` in `sdlc-core`. This feature exposes that data over HTTP and integrates the UAT completion lifecycle into the SSE event bus.

## Goals

1. Two new HTTP endpoints for querying milestone UAT run history.
2. A new `MilestoneUatCompleted` SSE variant so the frontend can react when a UAT agent finishes.
3. Frontend API client helpers and TypeScript types to match the new backend surface.

## Non-Goals

- No new data storage — the persistence layer already exists in `sdlc-core::milestone`.
- No UI changes beyond the client.ts and types.ts additions (UI will be a follow-on feature).
- No authentication changes.

## Deliverables

### 1. HTTP Routes

Add to `crates/sdlc-server/src/routes/milestones.rs`:

```
GET /api/milestones/{slug}/uat-runs
```
Returns a JSON array of `UatRun` records for the given milestone slug, sorted newest-first. Returns `[]` if no runs exist. Returns `404` if the slug is invalid or the milestone does not exist.

```
GET /api/milestones/{slug}/uat-runs/latest
```
Returns the most recent `UatRun` as JSON, or `404` if no runs have been recorded yet for the milestone.

Both routes delegate to `sdlc_core::milestone::list_uat_runs` and `sdlc_core::milestone::latest_uat_run` respectively, following the same `spawn_blocking` + `AppError` pattern used by all existing milestone handlers.

Register both routes in `crates/sdlc-server/src/lib.rs` (the `build_router_from_state` function).

### 2. SSE Variant

Add to `SseMessage` in `crates/sdlc-server/src/state.rs`:

```rust
/// A milestone UAT agent run completed — UatRun record written.
MilestoneUatCompleted { slug: String },
```

Serialization follows the existing pattern used by `PonderRunCompleted`, `ToolBuildCompleted`, etc. — the variant name becomes the `type` field in the SSE JSON payload, and the struct fields become top-level keys.

### 3. Emit the SSE Event

In `crates/sdlc-server/src/routes/runs.rs`, update `start_milestone_uat` to pass `Some(SseMessage::MilestoneUatCompleted { slug: slug.clone() })` as the `completion_event` argument to `spawn_agent_run` instead of `None`.

### 4. Frontend API Client

Add to `frontend/src/api/client.ts`:

```typescript
listMilestoneUatRuns: (slug: string) =>
  request<UatRun[]>(`/api/milestones/${encodeURIComponent(slug)}/uat-runs`),
getLatestMilestoneUatRun: (slug: string) =>
  request<UatRun | null>(`/api/milestones/${encodeURIComponent(slug)}/uat-runs/latest`),
```

Add to `frontend/src/lib/types.ts`:

```typescript
export type UatVerdict = 'pass' | 'pass_with_tasks' | 'failed'
export interface UatRun {
  id: string
  milestone_slug: string
  started_at: string
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

## Acceptance Criteria

- `GET /api/milestones/{slug}/uat-runs` returns `200` with an empty array when no runs exist.
- `GET /api/milestones/{slug}/uat-runs` returns `200` with a populated array sorted newest-first when runs exist.
- `GET /api/milestones/{slug}/uat-runs/latest` returns `404` when no runs exist.
- `GET /api/milestones/{slug}/uat-runs/latest` returns `200` with the most recent run when runs exist.
- `MilestoneUatCompleted { slug }` is emitted over SSE when a UAT agent run finishes.
- TypeScript types compile cleanly (`npx tsc --noEmit`).
- Rust builds cleanly (`SDLC_NO_NPM=1 cargo build --all`).
- No `unwrap()` in library or server code.

## Implementation Notes

- Both route handlers use `tokio::task::spawn_blocking` as is standard for all sync `sdlc-core` calls in the server.
- The `latest` endpoint returns `404` (via `AppError`) when `latest_uat_run` returns `Ok(None)`.
- Route registration order in `lib.rs` must place `uat-runs/latest` before `uat-runs/{id}` to avoid the `latest` segment being consumed as a run ID (though we have no `/{id}` route yet, good hygiene).
- The `UatRun` serialization from `sdlc-core` uses `serde_json` via `Json<Vec<UatRun>>` — no manual JSON construction needed.
