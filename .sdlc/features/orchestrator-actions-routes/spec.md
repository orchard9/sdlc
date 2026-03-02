# Spec: Orchestrator Actions REST API

## Feature
`orchestrator-actions-routes`

## Title
Orchestrator Actions REST API — GET/POST/DELETE actions routes + PATCH for label and recurrence editing

## Problem Statement

The orchestrator data layer (`ActionDb` in `sdlc-core`) already has complete CRUD for `Action` records: `insert`, `list_all`, `set_status`, and `delete`. However, no HTTP endpoints expose actions to the UI or external consumers. Currently the server only exposes webhook route management (`GET/POST /api/orchestrator/webhooks/routes`). The Actions page (`orchestrator-actions-page` feature) needs REST endpoints to list, create, delete, and edit actions before it can render anything meaningful.

## Goals

1. Expose `GET /api/orchestrator/actions` — list all actions, newest first.
2. Expose `POST /api/orchestrator/actions` — create a new scheduled action.
3. Expose `DELETE /api/orchestrator/actions/{id}` — delete an action by UUID.
4. Expose `PATCH /api/orchestrator/actions/{id}` — update `label` and/or `recurrence_secs` on an action.
5. Return `503` on all four routes when `action_db` is not available (consistent with existing webhook route handlers).
6. Register all four routes in `lib.rs`.
7. Add integration tests covering the happy-path and validation edge-cases.

## Non-Goals

- Triggering action execution (that is the orchestrator tick loop's job).
- Exposing `set_status` directly — status is managed by the tick loop, not the UI.
- Filtering or pagination — list returns all records; the UI can sort/filter client-side.
- Exposing webhook payloads via REST (separate `orchestrator-webhook-events` feature).

## Data Model

The `Action` struct (from `sdlc-core/src/orchestrator/action.rs`) has these fields relevant to the API:

| Field | Type | Notes |
|---|---|---|
| `id` | `Uuid` | Unique identifier |
| `label` | `String` | Human-readable name (e.g. "nightly-audit") |
| `tool_name` | `String` | Slug under `.sdlc/tools/<name>/` |
| `tool_input` | `serde_json::Value` | JSON passed to the tool at runtime |
| `trigger` | `ActionTrigger` | `Scheduled { next_tick_at }` or `Webhook { ... }` |
| `status` | `ActionStatus` | `Pending`, `Running`, `Completed { result }`, `Failed { reason }` |
| `recurrence` | `Option<Duration>` | Serialized as seconds in API |
| `created_at` | `DateTime<Utc>` | |
| `updated_at` | `DateTime<Utc>` | |

## API Contract

### GET /api/orchestrator/actions

Returns all actions sorted by `created_at` descending (newest first).

**Response 200:**
```json
[
  {
    "id": "uuid",
    "label": "nightly-audit",
    "tool_name": "quality-check",
    "tool_input": {},
    "trigger_type": "scheduled",
    "next_tick_at": "2026-03-03T00:00:00Z",
    "status": "pending",
    "recurrence_secs": 86400,
    "created_at": "2026-03-02T00:00:00Z",
    "updated_at": "2026-03-02T00:00:00Z"
  }
]
```

**Response 503** if `action_db` not available.

### POST /api/orchestrator/actions

Create a new scheduled action.

**Request body:**
```json
{
  "label": "nightly-audit",
  "tool_name": "quality-check",
  "tool_input": {},
  "next_tick_at": "2026-03-03T00:00:00Z",
  "recurrence_secs": 86400
}
```

Fields:
- `label` — required, non-empty string
- `tool_name` — required, non-empty, validated as slug (no path traversal)
- `tool_input` — required, any JSON object
- `next_tick_at` — required, ISO 8601 datetime
- `recurrence_secs` — optional, u64 (seconds)

**Response 201:** Same shape as one element from the GET list.

**Response 400:** Validation errors (empty label, invalid slug, etc.).

**Response 503:** DB not available.

### DELETE /api/orchestrator/actions/{id}

Delete a single action by UUID. Idempotent — returns 204 even if the action does not exist.

**Response 204** on success.

**Response 400** if `id` is not a valid UUID.

**Response 503** if DB not available.

### PATCH /api/orchestrator/actions/{id}

Update mutable fields: `label` and/or `recurrence_secs`. All fields are optional; omitted fields are left unchanged.

**Request body:**
```json
{
  "label": "renamed-audit",
  "recurrence_secs": 3600
}
```

- `label` — optional, non-empty string if provided
- `recurrence_secs` — optional, `null` to clear recurrence, positive integer to set

**Response 200:** Updated action in same shape as GET list element.

**Response 400:** Validation errors.

**Response 404:** Action not found.

**Response 503:** DB not available.

## Implementation Plan

1. **`crates/sdlc-core/src/orchestrator/db.rs`**: Add `delete` method (delete by UUID, idempotent) and `update_label_and_recurrence` method (find by id, update fields, reinsert with same key).

2. **`crates/sdlc-server/src/routes/orchestrator.rs`**: Add four new handler functions:
   - `list_actions` — calls `db.list_all()`
   - `create_action` — validates body, calls `Action::new_scheduled`, calls `db.insert()`
   - `delete_action` — parses UUID path param, calls `db.delete()`
   - `patch_action` — parses UUID path param, validates body, calls `db.update_label_and_recurrence()`

3. **`crates/sdlc-server/src/lib.rs`**: Register the four new routes under `/api/orchestrator/actions` and `/api/orchestrator/actions/{id}`.

4. **`crates/sdlc-server/tests/integration.rs`**: Add integration tests for all four endpoints.

## Acceptance Criteria

- `GET /api/orchestrator/actions` returns `[]` on an empty DB, and returns actions sorted newest-first after inserts.
- `POST /api/orchestrator/actions` with valid body returns `201` and the created action; invalid body returns `400`; missing DB returns `503`.
- `DELETE /api/orchestrator/actions/{uuid}` returns `204` for both existing and non-existing IDs; invalid UUID returns `400`.
- `PATCH /api/orchestrator/actions/{uuid}` returns `200` with updated fields; returns `404` if not found; empty label returns `400`.
- All handlers return `503` when `action_db` is `None`.
- `SDLC_NO_NPM=1 cargo test --all` passes.
- `cargo clippy --all -- -D warnings` passes.

## Dependencies

- `sdlc-core`: `orchestrator::ActionDb`, `orchestrator::Action`, `orchestrator::ActionTrigger`, `orchestrator::ActionStatus`
- `sdlc-server`: `AppState.action_db`, `AppError`, existing `orchestrator.rs` module pattern
- No new crate dependencies required
