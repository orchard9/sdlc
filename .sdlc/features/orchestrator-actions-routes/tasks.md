# Tasks: Orchestrator Actions REST API

## T1 — Add `delete` method to `ActionDb`

**File:** `crates/sdlc-core/src/orchestrator/db.rs`

Add `pub fn delete(&self, id: Uuid) -> Result<()>`:
- Scan `list_all()` to find the action
- If not found, return `Ok(())` (idempotent)
- Compute key via `action_key(action.trigger.key_ts(), action.id)`
- Open write transaction, remove key, commit

Add unit test `action_delete_removes_record` and `action_delete_nonexistent_is_idempotent` in the `#[cfg(test)]` block.

---

## T2 — Add `update_label_and_recurrence` method to `ActionDb`

**File:** `crates/sdlc-core/src/orchestrator/db.rs`

Add:
```rust
pub fn update_label_and_recurrence(
    &self,
    id: Uuid,
    label: Option<String>,
    recurrence: Option<Option<Duration>>,
) -> Result<Action>
```

- Scan `list_all()` to find action
- Return `SdlcError::OrchestratorDb(format!("action not found: {id}"))` if missing
- Apply updates: if `label.is_some()`, set `action.label`; if `recurrence.is_some()`, set `action.recurrence`
- Set `action.updated_at = Utc::now()`
- Remove old key, insert updated action at same key, commit
- Return updated action

Add unit tests:
- `update_label_changes_label`
- `update_recurrence_sets_and_clears`
- `update_not_found_returns_error`

---

## T3 — Add `not_found` helper to `AppError`

**File:** `crates/sdlc-server/src/error.rs`

Check if `AppError::not_found()` exists. If not, add it following the same pattern as `AppError::bad_request()` and `AppError::conflict()` — returns `404 Not Found` with a message body.

---

## T4 — Add `list_actions` handler

**File:** `crates/sdlc-server/src/routes/orchestrator.rs`

```rust
pub async fn list_actions(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError>
```

- Return `503` if `action_db` is `None`
- `spawn_blocking`: call `db.list_all()`
- Map each `Action` to JSON object:
  - `id`, `label`, `tool_name`, `tool_input`, `status` (string), `recurrence_secs` (null or u64)
  - `trigger_type`: `"scheduled"` or `"webhook"`
  - `next_tick_at`: RFC 3339 string when scheduled, absent when webhook
  - `created_at`, `updated_at`: RFC 3339 strings
- Return `200 OK` with JSON array

---

## T5 — Add `create_action` handler

**File:** `crates/sdlc-server/src/routes/orchestrator.rs`

Add `CreateActionBody` struct with fields `label`, `tool_name`, `tool_input`, `next_tick_at`, `recurrence_secs`.

```rust
pub async fn create_action(
    State(app): State<AppState>,
    Json(body): Json<CreateActionBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError>
```

- Return `503` if `action_db` is `None`
- Validate: non-empty `label`, non-empty `tool_name` passing `validate_slug()`, `tool_input` is JSON object
- `spawn_blocking`: `Action::new_scheduled(...)`, `db.insert(&action)`, return action JSON
- Return `201 Created`

---

## T6 — Add `delete_action` handler

**File:** `crates/sdlc-server/src/routes/orchestrator.rs`

```rust
pub async fn delete_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError>
```

- Return `503` if `action_db` is `None`
- Parse `id` as `Uuid`; return `400` if invalid
- `spawn_blocking`: call `db.delete(uuid)`
- Return `204 No Content`

---

## T7 — Add `patch_action` handler

**File:** `crates/sdlc-server/src/routes/orchestrator.rs`

Add `PatchActionBody` struct with `label: Option<String>` and `recurrence_secs: Option<serde_json::Value>`.

```rust
pub async fn patch_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PatchActionBody>,
) -> Result<Json<serde_json::Value>, AppError>
```

- Return `503` if `action_db` is `None`
- Parse `id` as `Uuid`; return `400` if invalid
- Validate: if `label` is `Some(s)`, `s` must be non-empty
- Parse `recurrence_secs`: `None` field → no change; `Value::Null` → `Some(None)`; `Value::Number(n)` → `Some(Some(Duration::from_secs(n)))`
- `spawn_blocking`: call `db.update_label_and_recurrence(...)`
- If error message contains "not found" → return `404`
- Return `200 OK` with updated action JSON

---

## T8 — Register routes in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

Add after the existing orchestrator webhook routes block:

```rust
.route(
    "/api/orchestrator/actions",
    get(routes::orchestrator::list_actions).post(routes::orchestrator::create_action),
)
.route(
    "/api/orchestrator/actions/{id}",
    delete(routes::orchestrator::delete_action).patch(routes::orchestrator::patch_action),
)
```

---

## T9 — Integration tests

**File:** `crates/sdlc-server/tests/integration.rs`

Add tests using `build_router_with_db`:

- `list_actions_empty` — GET returns 200 with `[]`
- `create_action_success` — POST returns 201 with action object
- `list_actions_returns_created` — GET after POST returns `[action]`
- `create_action_missing_db` — POST without DB returns 503
- `create_action_empty_label` — POST with `""` label returns 400
- `create_action_invalid_tool_name` — POST with `"../evil"` returns 400
- `delete_action_success` — DELETE existing returns 204; GET returns `[]`
- `delete_action_idempotent` — DELETE non-existent UUID returns 204
- `delete_action_invalid_uuid` — DELETE `/api/orchestrator/actions/not-a-uuid` returns 400
- `patch_action_label` — PATCH `{"label": "new"}` returns 200 with updated label
- `patch_action_recurrence_set` — PATCH `{"recurrence_secs": 3600}` returns 200 with `recurrence_secs: 3600`
- `patch_action_recurrence_clear` — PATCH `{"recurrence_secs": null}` returns 200 with `recurrence_secs: null`
- `patch_action_not_found` — PATCH with random UUID returns 404
- `patch_action_empty_label` — PATCH `{"label": ""}` returns 400

---

## T10 — Build and test verification

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Fix any warnings or test failures before marking complete.
