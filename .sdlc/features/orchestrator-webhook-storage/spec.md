# Spec: HTTP Webhook Receiver and Raw Payload Storage in redb

## Summary

Add a webhook ingestion layer to the orchestrator. When an external service sends a POST request to `POST /webhooks/:route`, the server stores the raw body bytes, receipt timestamp, and route path in a dedicated redb table. The stored payload becomes a `Webhook`-triggered `Action` that the orchestrator tick loop can dispatch.

No payload transformation on ingress — store exactly what arrived.

---

## Problem

The orchestrator currently supports only `Scheduled` action triggers. There is no way for external services (GitHub, CI systems, other tools) to push events into the orchestrator. A webhook endpoint + raw storage layer is the ingress primitive required before any webhook-driven automation can be built.

---

## Scope

### In scope

- `WebhookPayload` struct in `sdlc-core` with: `id: Uuid`, `route_path: String`, `raw_body: Vec<u8>`, `received_at: DateTime<Utc>`, `content_type: Option<String>`
- A `WEBHOOKS` redb table in `ActionDb` (key: 16-byte UUID, value: JSON-encoded `WebhookPayload`)
- Three new `ActionDb` methods:
  - `insert_webhook(payload: &WebhookPayload) -> Result<()>`
  - `all_pending_webhooks() -> Result<Vec<WebhookPayload>>`
  - `delete_webhook(id: Uuid) -> Result<()>`
- `POST /webhooks/:route` handler in `sdlc-server` that:
  - Accepts any `Content-Type`
  - Reads the raw body bytes
  - Constructs a `WebhookPayload` and calls `insert_webhook`
  - Returns `202 Accepted` with a JSON body `{ "id": "<uuid>" }`
- The route must be registered in `build_router_from_state` in `lib.rs`
- The `ActionDb` is opened from `orchestrator_db_path(root)` — the same `.sdlc/orchestrator.db` already used by the tick loop

### Out of scope

- Authentication / HMAC signature verification (future feature)
- Routing webhooks to specific tools (consumed by tick loop in a future feature)
- Payload size limits (future hardening)
- UI to inspect stored webhooks

---

## Data Model

```rust
/// A raw webhook payload received via POST /webhooks/:route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: Uuid,
    /// The `:route` path segment from the URL (e.g. "github" from /webhooks/github).
    pub route_path: String,
    /// Raw bytes as received — not decoded or validated.
    pub raw_body: Vec<u8>,
    /// RFC 3339 timestamp when the payload arrived.
    pub received_at: DateTime<Utc>,
    /// The Content-Type header from the request, if present.
    pub content_type: Option<String>,
}
```

The `WEBHOOKS` table key is the 16-byte UUID (raw bytes), not a composite timestamp key, because webhooks don't need ordered range scans — they need lookup by ID for deletion after dispatch.

---

## API

### `POST /webhooks/:route`

Accept a webhook from any external sender.

**Request:**
- Method: `POST`
- Path: `/webhooks/{route}` where `route` is a single path segment (e.g. `github`, `stripe`, `ci`)
- Body: any bytes
- Headers: `Content-Type` is captured but not enforced

**Response (success):**
```json
HTTP 202 Accepted
Content-Type: application/json

{ "id": "550e8400-e29b-41d4-a716-446655440000" }
```

**Response (error):**
```json
HTTP 500 Internal Server Error
Content-Type: application/json

{ "error": "failed to store webhook: <reason>" }
```

---

## ActionDb Extension

Three new methods on `ActionDb`:

```rust
impl ActionDb {
    /// Store a raw webhook payload in the WEBHOOKS table.
    pub fn insert_webhook(&self, payload: &WebhookPayload) -> Result<()>;

    /// Return all stored webhook payloads (for tick loop dispatch).
    pub fn all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>>;

    /// Delete a webhook payload by ID (after successful dispatch).
    pub fn delete_webhook(&self, id: Uuid) -> Result<()>;
}
```

---

## Server Integration

The webhook route uses the existing `AppState` to access `ActionDb`. `AppState` already holds the project root (`root: PathBuf`); the handler opens `ActionDb` at `orchestrator_db_path(&root)`.

Since `ActionDb::open()` is cheap (redb opens quickly), it is called per-request. If performance becomes an issue, `ActionDb` can be added to `AppState` in a future pass.

---

## File Locations

| File | Change |
|---|---|
| `crates/sdlc-core/src/orchestrator/mod.rs` | Export `WebhookPayload` |
| `crates/sdlc-core/src/orchestrator/db.rs` | Add `WEBHOOKS` table + 3 methods |
| A new file: `crates/sdlc-core/src/orchestrator/webhook.rs` | `WebhookPayload` struct |
| `crates/sdlc-server/src/routes/webhooks.rs` | New route handler |
| `crates/sdlc-server/src/routes/mod.rs` | `pub mod webhooks;` |
| `crates/sdlc-server/src/lib.rs` | Register `POST /webhooks/{route}` |

---

## Error Handling

- `SdlcError::OrchestratorDb(String)` is already defined — reuse it for all webhook storage errors
- The HTTP handler maps `SdlcError` to `500` with a JSON error body (consistent with other server error handling patterns)

---

## Tests

- Unit: `insert_webhook` / `all_pending_webhooks` / `delete_webhook` round-trip in `db.rs` (using `TempDir`)
- Integration: HTTP `POST /webhooks/test` returns `202` with a valid UUID in the body (using existing `build_router` test harness in `sdlc-server`)

---

## Acceptance Criteria

1. `POST /webhooks/github` with any body returns `202` and a UUID
2. After calling `insert_webhook`, `all_pending_webhooks()` returns the stored payload with exact raw bytes preserved
3. After calling `delete_webhook(id)`, `all_pending_webhooks()` no longer includes that payload
4. `cargo test --all` passes with `SDLC_NO_NPM=1`
5. `cargo clippy --all -- -D warnings` passes
