# Tasks: WebhookRoute Registration and Tick Dispatch

## T1 — Add `WebhookRoute` struct and `webhook.rs` module

Create `crates/sdlc-core/src/orchestrator/webhook.rs` with:
- `WebhookRoute` struct (`id`, `path`, `tool_name`, `input_template`, `created_at`) with `Serialize`/`Deserialize`.
- `WebhookRoute::new(path, tool_name, input_template)` constructor.
- `WebhookRoute::render_input(&self, raw_payload: &[u8]) -> Result<serde_json::Value>` — replaces `{{payload}}` with the JSON-escaped payload string, then parses the result as JSON.
- Unit tests for `render_input`: happy path and parse-error path.

Export from `mod.rs`: `pub use webhook::WebhookRoute;`

## T2 — Add `Action::new_webhook` constructor

In `crates/sdlc-core/src/orchestrator/action.rs`, add:
```rust
pub fn new_webhook(label: impl Into<String>, raw_payload: Vec<u8>) -> Self
```
Sets `tool_name = "_webhook"`, `tool_input = Null`, trigger = `ActionTrigger::Webhook { raw_payload, received_at: Utc::now() }`, status = `Pending`.

## T3 — Extend `ActionDb` with webhook route table

In `crates/sdlc-core/src/orchestrator/db.rs`:

1. Define `const WEBHOOK_ROUTES: TableDefinition<&[u8], &[u8]>` keyed by UUID bytes.
2. In `ActionDb::open()`, open (create if absent) `WEBHOOK_ROUTES` alongside `ACTIONS` in the same write transaction.
3. Add methods:
   - `insert_route(&self, route: &WebhookRoute) -> Result<()>` — checks for duplicate path, returns error if exists.
   - `list_routes(&self) -> Result<Vec<WebhookRoute>>` — returns all routes sorted by `created_at` ascending.
   - `find_route_by_path(&self, path: &str) -> Result<Option<WebhookRoute>>` — linear scan.
   - `all_pending_webhooks(&self) -> Result<Vec<Action>>` — full `ACTIONS` scan, returns `Pending` `Webhook`-triggered actions.
   - `delete_action(&self, action: &Action) -> Result<()>` — removes the action by its composite key.

Unit tests:
- Insert and retrieve a route.
- Duplicate path returns an error.
- `list_routes` is sorted by `created_at`.
- `all_pending_webhooks` returns only `Pending` `Webhook` actions.
- `delete_action` removes the record.

## T4 — Extend tick loop with webhook dispatch phase

In `crates/sdlc-cli/src/cmd/orchestrate.rs`:

1. Add `dispatch_webhook(root: &Path, db: &ActionDb, action: Action) -> Result<()>`:
   - Extract `raw_payload` from the `Webhook` trigger.
   - Look up the route by `action.label`.
   - If no route: log + delete action (no status update since record is removed).
   - If route found: render template → call `run_tool()` → log result → delete action.
2. In `run_one_tick`, after the existing scheduled loop, call `db.all_pending_webhooks()` and dispatch each.

Integration test: create a temp DB with a route and a webhook action, call `run_one_tick`, verify the action is deleted from the DB (tool invocation can be mocked by pointing `root` at a temp dir with a valid tool script).

## T5 — Add REST routes for webhook management

Create `crates/sdlc-server/src/routes/orchestrator.rs` with:

- `RegisterRouteBody` struct (`path`, `tool_name`, `input_template`).
- `register_route` handler: validate inputs, open DB, call `insert_route`, return `201` + `WebhookRoute` JSON.
  - `400` for empty/invalid path or tool_name.
  - `409` for duplicate path (detect via error message from `insert_route`).
- `list_routes` handler: open DB, call `list_routes`, return `200` + JSON array.
- `receive_webhook` handler:
  - Accept `*path` wildcard.
  - Read body bytes.
  - Build normalized path (prefix with `/`).
  - Create `Action::new_webhook(path, body.to_vec())`.
  - Open DB, insert action.
  - Return `202 Accepted`.

Add `pub mod orchestrator;` to `crates/sdlc-server/src/routes/mod.rs`.

## T6 — Register routes in `lib.rs`

In `crates/sdlc-server/src/lib.rs`, add to `build_router_from_state`:

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

Place these before any other `/api/orchestrator` fallback routes.

## T7 — Update `paths.rs` for webhook route path helper (optional)

If the server route opens `orchestrator_db_path` on each request, verify `orchestrator_db_path(root)` is already exported and accessible from `sdlc_core::paths`. If not, add or re-export it. (This may already exist — confirm and skip if so.)

## T8 — Clippy and tests pass

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`. Fix any warnings or test failures. Confirm all new unit tests pass.
