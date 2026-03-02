# Tasks: HTTP Webhook Receiver and Raw Payload Storage in redb

## T1: Add `WebhookPayload` struct to sdlc-core

Create `crates/sdlc-core/src/orchestrator/webhook.rs` with the `WebhookPayload` struct:
- Fields: `id: Uuid`, `route_path: String`, `raw_body: Vec<u8>`, `received_at: DateTime<Utc>`, `content_type: Option<String>`
- `WebhookPayload::new(route_path, raw_body, content_type)` constructor that generates a UUID and sets `received_at = Utc::now()`
- Derives: `Debug, Clone, Serialize, Deserialize`

Update `crates/sdlc-core/src/orchestrator/mod.rs` to:
- Add `pub mod webhook;`
- Re-export `pub use webhook::WebhookPayload;`

## T2: Add WEBHOOKS table and methods to ActionDb

In `crates/sdlc-core/src/orchestrator/db.rs`:
- Add `const WEBHOOKS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("webhooks");`
- In `ActionDb::open()`: open the `WEBHOOKS` table in the initial write transaction (alongside `ACTIONS`) so both tables exist on every open
- Add `insert_webhook(&self, payload: &WebhookPayload) -> Result<()>` — key is `payload.id.as_bytes()`, value is JSON-encoded payload
- Add `all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>>` — full table scan, sorted by `received_at` ascending
- Add `delete_webhook(&self, id: Uuid) -> Result<()>` — remove entry by UUID key

## T3: Add unit tests for webhook storage methods

In the `#[cfg(test)]` block in `crates/sdlc-core/src/orchestrator/db.rs`:
- `webhook_insert_and_retrieve_round_trip` — insert one payload, verify `all_pending_webhooks` returns it with the exact same `raw_body`, `route_path`, `id`, and `content_type`
- `webhook_delete_removes_record` — insert, then delete, verify `all_pending_webhooks` is empty
- `webhook_multiple_payloads_sorted_by_received_at` — insert two payloads with different `received_at` values, verify ordering
- `empty_db_all_pending_webhooks_returns_empty` — fresh DB returns empty vec
- `existing_db_open_adds_webhooks_table` — open a DB, close it, reopen it, verify `all_pending_webhooks` works (exercises the table-creation-on-open path for existing DBs)

## T4: Add webhook HTTP route to sdlc-server

Create `crates/sdlc-server/src/routes/webhooks.rs`:
- `pub async fn receive_webhook(State(state): State<AppState>, Path(route): Path<String>, headers: HeaderMap, body: Bytes) -> impl IntoResponse`
- Extract `Content-Type` from headers (capture as `Option<String>`)
- Construct `WebhookPayload::new(route, body.to_vec(), content_type)`
- Open `ActionDb` at `orchestrator_db_path(&state.root)`
- Call `insert_webhook(&payload)`
- Return `202 Accepted` with `{ "id": "<uuid>" }` on success
- Return `500 Internal Server Error` with `{ "error": "<message>" }` on failure

Update `crates/sdlc-server/src/routes/mod.rs`: add `pub mod webhooks;`

Update `crates/sdlc-server/src/lib.rs`: register `.route("/webhooks/{route}", post(routes::webhooks::receive_webhook))`

## T5: Add integration test for the webhook HTTP endpoint

In `crates/sdlc-server/tests/integration.rs`:
- Test `post_webhook_returns_202_with_id`:
  - Build router with `build_router` using a `TempDir` as root
  - POST `/webhooks/test` with a body (e.g. `b"hello world"`) and `Content-Type: text/plain`
  - Assert response status is `202`
  - Assert response body is valid JSON with an `"id"` field containing a valid UUID string

## T6: Verify build and tests pass

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`. Fix any compilation errors or warnings.
