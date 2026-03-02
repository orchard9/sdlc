# Design: WebhookRoute Registration and Tick Dispatch

## Overview

This feature adds three things to the existing orchestrator:

1. A `WebhookRoute` data model and a `webhook_routes` redb table in `ActionDb`.
2. Two REST endpoints for route management (`POST` and `GET` on `/api/orchestrator/webhooks/routes`) plus one ingestion endpoint (`POST /api/orchestrator/webhooks/:path*`).
3. Extended tick dispatch: after scheduled actions run, pending webhook actions are matched against routes, rendered, and dispatched.

## Module Layout

```
crates/sdlc-core/src/orchestrator/
  mod.rs          (re-export WebhookRoute, add all_pending_webhooks, delete_webhook)
  action.rs       (unchanged)
  db.rs           (add webhook_routes table, CRUD methods, all_pending_webhooks)
  webhook.rs      (WebhookRoute struct + template rendering)

crates/sdlc-server/src/routes/
  orchestrator.rs (new file: REST handlers for routes + ingestion)

crates/sdlc-server/src/
  lib.rs          (register new routes)
```

The tick loop changes live in `crates/sdlc-cli/src/cmd/orchestrate.rs` — specifically in `run_one_tick`.

## Data Layer (`orchestrator/webhook.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRoute {
    pub id: Uuid,
    pub path: String,
    pub tool_name: String,
    pub input_template: String,
    pub created_at: DateTime<Utc>,
}

impl WebhookRoute {
    pub fn new(path: impl Into<String>, tool_name: impl Into<String>, input_template: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            path: path.into(),
            tool_name: tool_name.into(),
            input_template: input_template.into(),
            created_at: Utc::now(),
        }
    }

    /// Render input_template by substituting {{payload}} with the JSON-escaped webhook body.
    pub fn render_input(&self, raw_payload: &[u8]) -> Result<serde_json::Value> {
        let payload_str = String::from_utf8_lossy(raw_payload);
        let payload_json = serde_json::to_string(&payload_str)?; // JSON-escaped string
        let rendered = self.input_template.replace("{{payload}}", &payload_json);
        let value: serde_json::Value = serde_json::from_str(&rendered)
            .map_err(|e| SdlcError::OrchestratorDb(format!("template render failed: {e}")))?;
        Ok(value)
    }
}
```

## Storage Layer (`orchestrator/db.rs`)

### New table

```rust
const WEBHOOK_ROUTES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("webhook_routes");
```

Table is opened (created if absent) in `ActionDb::open()` alongside `ACTIONS`.

### New methods on `ActionDb`

| Method | Signature | Description |
|---|---|---|
| `insert_route` | `(&self, route: &WebhookRoute) -> Result<()>` | Insert a route; returns `OrchestratorDb` error if path already registered |
| `list_routes` | `(&self) -> Result<Vec<WebhookRoute>>` | Return all routes, sorted by `created_at` ascending |
| `find_route_by_path` | `(&self, path: &str) -> Result<Option<WebhookRoute>>` | Linear scan, returns first match |
| `all_pending_webhooks` | `(&self) -> Result<Vec<Action>>` | Full table scan, returns `Pending` actions with `Webhook` trigger |
| `delete_action` | `(&self, action: &Action) -> Result<()>` | Remove a record by key (used after webhook dispatch) |

The `insert_route` method checks for path uniqueness before inserting and returns an `OrchestratorDb` error with a message the HTTP handler maps to `409 Conflict`.

## Tick Loop Changes (`cmd/orchestrate.rs`)

`run_one_tick` gains a second phase after the existing scheduled-action loop:

```rust
pub fn run_one_tick(root: &Path, db: &ActionDb) -> Result<()> {
    // Phase 1: scheduled actions (unchanged)
    let now = Utc::now();
    let due = db.range_due(now)?;
    for action in due {
        dispatch(root, db, action)?;
    }

    // Phase 2: webhook actions
    let webhooks = db.all_pending_webhooks()?;
    for action in webhooks {
        dispatch_webhook(root, db, action)?;
    }
    Ok(())
}

fn dispatch_webhook(root: &Path, db: &ActionDb, action: Action) -> Result<()> {
    let raw_payload = match &action.trigger {
        ActionTrigger::Webhook { raw_payload, .. } => raw_payload.clone(),
        _ => unreachable!("all_pending_webhooks only returns Webhook triggers"),
    };

    let route = db.find_route_by_path(&action.label)?;

    let status = match route {
        None => {
            let reason = format!("no route registered for path: {}", action.label);
            eprintln!("orchestrate: [{}] webhook dispatch failed — {reason}", action.label);
            ActionStatus::Failed { reason }
        }
        Some(route) => {
            let tool_input = match route.render_input(&raw_payload) {
                Ok(v) => v,
                Err(e) => {
                    let reason = format!("template render error: {e}");
                    eprintln!("orchestrate: [{}] {reason}", action.label);
                    // delete action before returning error status
                    let _ = db.delete_action(&action);
                    db.set_status(action.id, ActionStatus::Failed { reason })?;
                    return Ok(()); // already handled
                }
            };
            let script = sdlc_core::paths::tool_script(root, &route.tool_name);
            if !script.exists() {
                let reason = format!("tool script not found: {}", script.display());
                eprintln!("orchestrate: [{}] {reason}", action.label);
                let _ = db.delete_action(&action);
                return Ok(());
            }
            let input_json = serde_json::to_string(&tool_input)?;
            match sdlc_core::tool_runner::run_tool(&script, "--run", Some(&input_json), root, None) {
                Ok(stdout) => {
                    let result = serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
                    eprintln!("orchestrate: [{}] webhook completed", action.label);
                    ActionStatus::Completed { result }
                }
                Err(e) => {
                    let reason = e.to_string();
                    eprintln!("orchestrate: [{}] webhook failed — {reason}", action.label);
                    ActionStatus::Failed { reason }
                }
            }
        }
    };

    // Delete action after dispatch regardless of outcome
    db.delete_action(&action)?;
    Ok(())
}
```

Note: `set_status` is NOT called on webhook actions after deletion — the record is removed from the DB entirely. Audit history for webhook dispatches is captured in the orchestrator's stderr log and (future work) a webhook run log.

## REST Layer (`routes/orchestrator.rs`)

```rust
// POST /api/orchestrator/webhooks/routes
pub async fn register_route(State(app): State<AppState>, Json(body): Json<RegisterRouteBody>)
    -> Result<(StatusCode, Json<WebhookRoute>), AppError>

// GET /api/orchestrator/webhooks/routes  
pub async fn list_routes(State(app): State<AppState>)
    -> Result<Json<Vec<WebhookRoute>>, AppError>

// POST /api/orchestrator/webhooks/*path
pub async fn receive_webhook(
    State(app): State<AppState>,
    Path(path): Path<String>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError>
```

### Request body for `register_route`

```rust
#[derive(Deserialize)]
pub struct RegisterRouteBody {
    pub path: String,
    pub tool_name: String,
    pub input_template: String,
}
```

### `register_route` validation

- `path` must start with `/` and be non-empty — return `400` otherwise.
- `tool_name` must be non-empty — return `400` otherwise.
- `input_template` must be non-empty — return `400` otherwise.
- Duplicate `path` → `409 Conflict`.

### `receive_webhook`

- Constructs a route path as `/{path}` (the axum wildcard captures the suffix without the leading slash).
- Creates an `Action` with:
  - `label` = the normalized path (e.g. `/hooks/my-service`)
  - `tool_name` = `"_webhook"` (placeholder; actual tool resolved at dispatch via route table)
  - `tool_input` = `serde_json::Value::Null` (actual input rendered at dispatch)
  - `trigger` = `ActionTrigger::Webhook { raw_payload: body.to_vec(), received_at: Utc::now() }`
- Inserts the action into `ActionDb`.
- Returns `202 Accepted`.

Since `Action::new_scheduled` only supports scheduled triggers, a new constructor `Action::new_webhook` is added:

```rust
pub fn new_webhook(label: impl Into<String>, raw_payload: Vec<u8>) -> Self {
    let now = Utc::now();
    Self {
        id: Uuid::new_v4(),
        label: label.into(),
        tool_name: "_webhook".into(),
        tool_input: serde_json::Value::Null,
        trigger: ActionTrigger::Webhook { raw_payload, received_at: now },
        status: ActionStatus::Pending,
        recurrence: None,
        created_at: now,
        updated_at: now,
    }
}
```

## Router Registration (`lib.rs`)

```rust
.route(
    "/api/orchestrator/webhooks/routes",
    get(routes::orchestrator::list_routes).post(routes::orchestrator::register_route),
)
.route(
    "/api/orchestrator/webhooks/*path",
    post(routes::orchestrator::receive_webhook),
)
```

The wildcard `*path` route must be placed before any other `/api/orchestrator` routes in the router to avoid shadowing.

## AppState Changes

The `AppState` struct needs access to the `ActionDb`. Since the orchestrator is currently CLI-only (the daemon is a separate `sdlc orchestrate` process), the server routes open the DB on each request using `sdlc_core::paths::orchestrator_db_path`.

This avoids adding `ActionDb` to `AppState` (which would require making `Database` `Send + Sync` or wrapping it in `Arc<Mutex<...>>`). Each request opens, operates, and closes the DB. For the expected webhook volume (low-frequency automation), this is acceptable. Future optimization can pool the DB handle if needed.

## Error Mapping

The server `AppError` type already maps `anyhow::Error` to `500`. For this feature, route handlers return typed errors:
- `OrchestratorDb` errors from duplicate path → mapped to `409` in `register_route`.
- Validation errors → `400 Bad Request` with a message body.

The `AppError` mechanism (from `crates/sdlc-server/src/error.rs`) is extended with a small helper:

```rust
impl AppError {
    pub fn bad_request(msg: impl std::fmt::Display) -> (StatusCode, Self) {
        (StatusCode::BAD_REQUEST, Self(anyhow::anyhow!("{msg}")))
    }
    pub fn conflict(msg: impl std::fmt::Display) -> (StatusCode, Self) {
        (StatusCode::CONFLICT, Self(anyhow::anyhow!("{msg}")))
    }
}
```

(Check if these helpers already exist; add only if missing.)

## Testing

### Unit tests (in `db.rs`)

- Insert and retrieve a `WebhookRoute` by path.
- Duplicate path returns error.
- `list_routes` returns sorted results.
- `all_pending_webhooks` returns only `Pending` `Webhook` trigger actions.
- `delete_action` removes the record from the DB.

### Unit tests (in `webhook.rs`)

- `render_input` substitutes `{{payload}}` correctly.
- `render_input` returns an error when the rendered string is not valid JSON.

### Integration tests (in `orchestrate.rs` or a new `tests/orchestrator_webhook.rs`)

- Full round-trip: insert route → insert webhook action → `run_one_tick` → action deleted, tool called (mock tool via `TempDir` with a tool.ts that writes stdout).

## Non-Goals

- HMAC webhook signature verification.
- Multiple routes per path.
- Wildcard path matching (exact path match only).
- CLI subcommands for route CRUD.
