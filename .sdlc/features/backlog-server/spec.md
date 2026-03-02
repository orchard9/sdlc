# Spec: backlog-server

## Overview

Expose the existing `sdlc_core::backlog::BacklogStore` data layer as REST API endpoints in `sdlc-server`, enabling the frontend and external agents to read and mutate backlog items via HTTP. The backlog is a project-level parking lot for out-of-scope concerns discovered during autonomous agent runs.

## Background

`sdlc_core::backlog` (at `crates/sdlc-core/src/backlog.rs`) already implements the full data layer:
- `BacklogStore::load` / `save` — `.sdlc/backlog.yaml`
- `BacklogStore::add` — create with `title`, `kind`, `description`, `evidence`, `source_feature`
- `BacklogStore::list` — filter by `status` and/or `source_feature`
- `BacklogStore::get` — fetch single item by B-prefixed ID
- `BacklogStore::park` — transition to `Parked` with required non-empty `park_reason`
- `BacklogStore::mark_promoted` — transition to `Promoted`, setting `promoted_to` slug

Enums: `BacklogKind` (`concern | idea | debt`), `BacklogStatus` (`open | parked | promoted`).

`SdlcError::BacklogItemNotFound` is already handled as HTTP 404 in `src/error.rs`.
`SdlcError::InvalidTransition` maps to HTTP 422.

## API Design

### GET /api/backlog

List backlog items. Optional query parameters:
- `?status=open|parked|promoted` — filter by status (omit for all)
- `?source_feature=<slug>` — filter by originating feature (omit for all)

Both filters compose as AND when both are present.

Response `200 OK`:
```json
[
  {
    "id": "B1",
    "title": "...",
    "kind": "concern",
    "status": "open",
    "description": null,
    "evidence": null,
    "source_feature": null,
    "park_reason": null,
    "promoted_to": null,
    "created_at": "...",
    "updated_at": "..."
  }
]
```

### POST /api/backlog

Create a new backlog item. ID is assigned by the server.

Request body:
```json
{
  "title": "auth.rs: token race under concurrent requests",
  "kind": "concern",
  "description": "optional detail",
  "evidence": "optional grounding ref",
  "source_feature": "optional-feature-slug"
}
```

Required fields: `title`, `kind`.
Optional: `description`, `evidence`, `source_feature`.

Response `201 Created`:
```json
{
  "id": "B1",
  "title": "...",
  "kind": "concern",
  "status": "open",
  "description": null,
  "evidence": null,
  "source_feature": null,
  "park_reason": null,
  "promoted_to": null,
  "created_at": "...",
  "updated_at": "..."
}
```

Validation error (missing required field) → `400 Bad Request`.

### POST /api/backlog/:id/park

Park an item. Requires a non-empty `park_reason`.

Request body:
```json
{
  "park_reason": "revisit after v14, not urgent"
}
```

Response `200 OK`: the updated item JSON (same shape as list item).

Errors:
- `404` if item not found
- `422` if `park_reason` is empty or if item is already `promoted`
- `400` if `park_reason` field is missing from body

### POST /api/backlog/:id/promote

Mark an item as promoted to a feature. The feature itself is created by the CLI; the server only records the transition.

Request body:
```json
{
  "slug": "auth-race-fix",
  "milestone_slug": "v15-layout-foundation"
}
```

Both `slug` and `milestone_slug` are optional. If `slug` is omitted, the `promoted_to` field on the item will remain null (the promotion is still recorded). The `milestone_slug` field is informational only — it is not stored on the backlog item but may be used by future routing logic.

Response `200 OK`: the updated item JSON.

Errors:
- `404` if item not found
- `422` if already promoted

## Implementation Plan

1. Create `crates/sdlc-server/src/routes/backlog.rs` with four handlers: `list_backlog`, `create_backlog_item`, `park_backlog_item`, `promote_backlog_item`.
2. Add `pub mod backlog;` to `crates/sdlc-server/src/routes/mod.rs`.
3. Register all four routes in `crates/sdlc-server/src/lib.rs` `build_router_from_state`.
4. Write integration tests in `crates/sdlc-server/tests/integration.rs`.

## Handler Pattern

All handlers follow the established pattern:

```rust
pub async fn list_backlog(
    State(app): State<AppState>,
    Query(params): Query<ListBacklogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        // call BacklogStore::list(...)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}
```

## Testing Requirements

Integration tests cover:
1. `GET /api/backlog` returns empty array when no backlog exists
2. `POST /api/backlog` creates an item with the correct ID (B1) and fields
3. `GET /api/backlog` returns the created item
4. `GET /api/backlog?status=open` filters correctly
5. `GET /api/backlog?source_feature=feat-x` filters by source feature
6. `POST /api/backlog/:id/park` transitions item to parked with reason
7. `POST /api/backlog/:id/park` with empty reason returns 422
8. `POST /api/backlog/:id/park` on promoted item returns 422
9. `POST /api/backlog/:id/promote` transitions item to promoted
10. `POST /api/backlog/:id/promote` on already-promoted item returns 422
11. `POST /api/backlog/:id/park` with unknown ID returns 404
12. `POST /api/backlog/:id/promote` with unknown ID returns 404

## Out of Scope

- Feature creation during promote (CLI responsibility)
- SSE event emission (no UI subscription needed at this phase)
- Authentication (handled by existing auth middleware, not per-route)
- Pagination (backlog is a flat short list, typically < 200 items)
