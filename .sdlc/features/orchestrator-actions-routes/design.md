# Design: Orchestrator Actions REST API

## Overview

This feature adds four HTTP endpoints to `sdlc-server` that expose CRUD operations for orchestrator `Action` records. It extends the existing `orchestrator.rs` route module and adds two new methods to `ActionDb` in `sdlc-core`.

The design follows the exact pattern established by the existing webhook route handlers (`list_routes`, `register_route`) in `crates/sdlc-server/src/routes/orchestrator.rs`.

---

## Component Map

```
sdlc-core/src/orchestrator/
  db.rs              ← add: delete(), update_label_and_recurrence()
  action.rs          ← unchanged (Action, ActionTrigger, ActionStatus already complete)

sdlc-server/src/routes/
  orchestrator.rs    ← add: list_actions(), create_action(), delete_action(), patch_action()

sdlc-server/src/
  lib.rs             ← register 4 new routes under /api/orchestrator/actions

sdlc-server/tests/
  integration.rs     ← add integration tests for all 4 endpoints
```

---

## Data Layer Changes (`sdlc-core/src/orchestrator/db.rs`)

### `ActionDb::delete(id: Uuid) -> Result<()>`

Deletes an action by ID. Idempotent — silently succeeds if the action does not exist.

**Algorithm:**
1. Full scan `list_all()` to find the action with matching `id`.
2. If not found, return `Ok(())`.
3. Compute the action's redb key via `action_key(action.trigger.key_ts(), action.id)`.
4. Open write transaction, remove the key, commit.

This matches the existing `delete_route` and `delete_webhook` pattern.

### `ActionDb::update_label_and_recurrence(id: Uuid, label: Option<String>, recurrence: Option<Option<Duration>>) -> Result<Action>`

Updates mutable fields of an action in-place. Returns the updated `Action`.

**Parameters:**
- `label: Option<String>` — if `Some(s)`, set `action.label = s`; if `None`, leave unchanged
- `recurrence: Option<Option<Duration>>` — outer `None` = leave unchanged; `Some(None)` = clear; `Some(Some(d))` = set

**Algorithm:**
1. `list_all()` to find action with matching `id`.
2. Return `SdlcError::OrchestratorDb("action not found: {id}")` if missing.
3. Apply field updates; set `action.updated_at = Utc::now()`.
4. Compute old key via `action_key(action.trigger.key_ts(), action.id)`.
5. Open write transaction: remove old key, insert new value at same key (key is unchanged since trigger timestamp and ID don't change).
6. Return the updated `Action`.

---

## Route Handler Changes (`sdlc-server/src/routes/orchestrator.rs`)

All four handlers follow the same `spawn_blocking` pattern as the existing `list_routes` / `register_route` handlers.

### `list_actions` — `GET /api/orchestrator/actions`

```rust
pub async fn list_actions(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError>
```

- Returns `503` if `action_db` is `None`.
- Calls `db.list_all()` (already sorted newest-first by `created_at` descending).
- Maps each `Action` to a JSON object (see shape below).
- Returns `200 OK` with JSON array.

**JSON shape for one action:**
```json
{
  "id": "uuid-string",
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
```

Notes on serialization:
- `trigger_type`: `"scheduled"` or `"webhook"` (derived from `ActionTrigger` variant)
- `next_tick_at`: present and RFC 3339 when `trigger_type == "scheduled"`; absent when webhook
- `status`: `"pending"` | `"running"` | `"completed"` | `"failed"` (string, not the full enum)
- `recurrence_secs`: `null` or a positive integer (seconds)

### `create_action` — `POST /api/orchestrator/actions`

```rust
#[derive(Deserialize)]
pub struct CreateActionBody {
    pub label: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub next_tick_at: DateTime<Utc>,
    pub recurrence_secs: Option<u64>,
}

pub async fn create_action(
    State(app): State<AppState>,
    Json(body): Json<CreateActionBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError>
```

**Validation (400 on failure):**
- `label` must be non-empty
- `tool_name` must be non-empty and pass `validate_slug()` (path traversal guard)
- `tool_input` must be a JSON object (not array or primitive)

**On success:**
- Constructs `Action::new_scheduled(label, tool_name, tool_input, next_tick_at, recurrence)`
- Calls `db.insert(&action)`
- Returns `201 Created` with the action JSON object

### `delete_action` — `DELETE /api/orchestrator/actions/{id}`

```rust
pub async fn delete_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError>
```

**Validation:**
- Parse `id` as `Uuid`; return `400` if invalid

**On success:**
- Calls `db.delete(uuid)`
- Returns `204 No Content` (idempotent — always 204 if DB is available)

### `patch_action` — `PATCH /api/orchestrator/actions/{id}`

```rust
#[derive(Deserialize)]
pub struct PatchActionBody {
    pub label: Option<String>,
    pub recurrence_secs: Option<serde_json::Value>,  // null or u64
}

pub async fn patch_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PatchActionBody>,
) -> Result<Json<serde_json::Value>, AppError>
```

Using `serde_json::Value` for `recurrence_secs` allows distinguishing `null` (clear) from absent (leave unchanged). The field is always present in the PATCH body if the client intends to change it.

**Validation (400 on failure):**
- Parse `id` as `Uuid`
- If `label` is `Some(s)`, `s` must be non-empty

**On success:**
- Parse `recurrence_secs` from JSON: `Value::Null` → `Some(None)`, `Value::Number(n)` → `Some(Some(Duration::from_secs(n)))`, field absent → `None` (leave unchanged)
- Calls `db.update_label_and_recurrence(uuid, label, recurrence)`
- On `SdlcError::OrchestratorDb` containing "not found" → return `404`
- On success → return `200 OK` with updated action JSON object

---

## Router Registration (`sdlc-server/src/lib.rs`)

Add after the existing orchestrator webhook route block:

```rust
// Orchestrator actions CRUD
.route(
    "/api/orchestrator/actions",
    get(routes::orchestrator::list_actions).post(routes::orchestrator::create_action),
)
.route(
    "/api/orchestrator/actions/{id}",
    delete(routes::orchestrator::delete_action).patch(routes::orchestrator::patch_action),
)
```

The ordering in `build_router_from_state` places this immediately after the existing orchestrator routes, keeping them grouped.

---

## Integration Tests (`sdlc-server/tests/integration.rs`)

Tests use `build_router_with_db` (already used in existing webhook tests) to inject a live `ActionDb`.

**Test cases:**

| Test | Method | Path | Expected |
|---|---|---|---|
| `list_actions_empty` | GET | `/api/orchestrator/actions` | 200, `[]` |
| `create_action_success` | POST | `/api/orchestrator/actions` | 201, action object |
| `list_actions_returns_created` | GET after POST | `/api/orchestrator/actions` | 200, `[action]` |
| `create_action_missing_db` | POST | `/api/orchestrator/actions` | 503 |
| `create_action_empty_label` | POST | `/api/orchestrator/actions` | 400 |
| `create_action_invalid_tool_name` | POST | `/api/orchestrator/actions` | 400 (path traversal) |
| `delete_action_success` | DELETE | `/api/orchestrator/actions/{id}` | 204 |
| `delete_action_idempotent` | DELETE nonexistent | `/api/orchestrator/actions/{id}` | 204 |
| `delete_action_invalid_uuid` | DELETE | `/api/orchestrator/actions/bad` | 400 |
| `patch_action_label` | PATCH | `/api/orchestrator/actions/{id}` | 200, updated label |
| `patch_action_recurrence_set` | PATCH | `/api/orchestrator/actions/{id}` | 200, recurrence_secs set |
| `patch_action_recurrence_clear` | PATCH (null) | `/api/orchestrator/actions/{id}` | 200, recurrence_secs null |
| `patch_action_not_found` | PATCH | `/api/orchestrator/actions/{uuid}` | 404 |
| `patch_action_empty_label` | PATCH | `/api/orchestrator/actions/{id}` | 400 |

---

## Error Mapping

| Condition | HTTP Status |
|---|---|
| `action_db` is `None` | 503 Service Unavailable |
| Invalid UUID in path | 400 Bad Request |
| Empty `label` | 400 Bad Request |
| Invalid `tool_name` (slug check) | 400 Bad Request |
| `tool_input` not a JSON object | 400 Bad Request |
| Action not found (PATCH only) | 404 Not Found |
| DB mutex poisoned | 500 Internal Server Error (via AppError) |

---

## Consistency with Existing Patterns

- All handlers use `spawn_blocking` wrapping the synchronous `ActionDb` methods (same as `list_routes`, `register_route`).
- The `AppError::bad_request()` and `AppError::conflict()` helpers are already defined in `crates/sdlc-server/src/error.rs`. A `AppError::not_found()` helper may need to be added (check `error.rs` during implementation).
- No new crate dependencies. `chrono::DateTime<Utc>`, `uuid::Uuid`, `serde_json::Value` are already in scope.
- `validate_slug` is already imported in `orchestrator.rs` for the existing `register_route` handler.
