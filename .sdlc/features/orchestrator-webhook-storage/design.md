# Design: HTTP Webhook Receiver and Raw Payload Storage in redb

## Overview

This design adds the minimal ingress layer needed for webhook-driven orchestration. The work spans two crates: `sdlc-core` (data model and storage) and `sdlc-server` (HTTP handler).

The design is deliberately thin — no routing logic, no signature verification, no fanout. Store exactly what arrived, indexed by UUID, and expose the stored payloads for the tick loop to consume later.

---

## Component Map

```
External sender
     │
     │  POST /webhooks/{route}
     ▼
┌─────────────────────────────────┐
│  sdlc-server                    │
│  routes/webhooks.rs             │
│  - extract raw body bytes       │
│  - extract Content-Type header  │
│  - construct WebhookPayload     │
│  - call ActionDb::insert_webhook│
│  - return 202 + { "id": "..." } │
└──────────────┬──────────────────┘
               │
               │  insert_webhook()
               ▼
┌─────────────────────────────────┐
│  sdlc-core                      │
│  orchestrator/db.rs             │
│  WEBHOOKS table (redb)          │
│  key:   16-byte UUID            │
│  value: JSON WebhookPayload     │
└─────────────────────────────────┘
               │
               │  all_pending_webhooks() / delete_webhook()
               ▼
┌─────────────────────────────────┐
│  Orchestrator tick loop         │
│  (future: consumes payloads)    │
└─────────────────────────────────┘
```

---

## Data Model

### `WebhookPayload` (new file: `orchestrator/webhook.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: Uuid,
    pub route_path: String,
    pub raw_body: Vec<u8>,
    pub received_at: DateTime<Utc>,
    pub content_type: Option<String>,
}

impl WebhookPayload {
    pub fn new(route_path: String, raw_body: Vec<u8>, content_type: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            route_path,
            raw_body,
            received_at: Utc::now(),
            content_type,
        }
    }
}
```

`raw_body` is stored as a JSON byte array (`Vec<u8>` serialized as base64 via serde). The entire struct is JSON-encoded and stored as the redb value.

---

## Storage Design

### Table Layout

```
WEBHOOKS table
  Key:   [uuid: 16 bytes]
  Value: JSON-encoded WebhookPayload
```

Key choice: 16-byte UUID (not a composite timestamp key) because:
- Webhooks are consumed by delete-after-dispatch — need O(1) lookup by ID
- No ordered range scans needed (unlike `ACTIONS` which is keyed by timestamp for range_due queries)
- UUID is already the natural identity of a payload

### WEBHOOKS Table Definition

```rust
const WEBHOOKS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("webhooks");
```

Added alongside the existing `ACTIONS` table definition in `db.rs`. Both tables are created in `ActionDb::open()` so the database is always ready before first use.

---

## ActionDb Extension

Three new methods in `db.rs`:

### `insert_webhook`

```rust
pub fn insert_webhook(&self, payload: &WebhookPayload) -> Result<()> {
    let key = payload.id.as_bytes().as_slice();
    let value = serde_json::to_vec(payload)?;
    let wt = self.db.begin_write()?;
    {
        let mut table = wt.open_table(WEBHOOKS)?;
        table.insert(key, value.as_slice())?;
    }
    wt.commit()?;
    Ok(())
}
```

### `all_pending_webhooks`

```rust
pub fn all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>> {
    let rt = self.db.begin_read()?;
    let table = rt.open_table(WEBHOOKS)?;
    let mut result = Vec::new();
    for entry in table.iter()? {
        let (_, v) = entry?;
        let payload: WebhookPayload = serde_json::from_slice(v.value())?;
        result.push(payload);
    }
    result.sort_by(|a, b| a.received_at.cmp(&b.received_at));
    Ok(result)
}
```

### `delete_webhook`

```rust
pub fn delete_webhook(&self, id: Uuid) -> Result<()> {
    let key = id.as_bytes().as_slice();
    let wt = self.db.begin_write()?;
    {
        let mut table = wt.open_table(WEBHOOKS)?;
        table.remove(key)?;
    }
    wt.commit()?;
    Ok(())
}
```

---

## HTTP Handler

### `routes/webhooks.rs`

```rust
pub async fn receive_webhook(
    State(state): State<AppState>,
    Path(route): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let payload = WebhookPayload::new(route, body.to_vec(), content_type);
    let id = payload.id;

    let db_path = orchestrator_db_path(&state.root);
    match ActionDb::open(&db_path).and_then(|db| db.insert_webhook(&payload)) {
        Ok(()) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "id": id.to_string() })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ).into_response(),
    }
}
```

### Route Registration in `lib.rs`

```rust
.route("/webhooks/{route}", post(routes::webhooks::receive_webhook))
```

Note: `/webhooks/*` is a public endpoint (no auth middleware bypass needed for now — it sits behind the same auth layer as `/api/*`). This matches the intended integration pattern where the sender knows the tunnel URL and token.

---

## Module Additions

### `orchestrator/mod.rs`

```rust
pub mod action;
pub mod db;
pub mod webhook;

pub use action::{Action, ActionStatus, ActionTrigger};
pub use db::ActionDb;
pub use webhook::WebhookPayload;
```

### `routes/mod.rs`

```rust
pub mod webhooks;
// ... all existing entries ...
```

---

## Error Handling

All redb errors are mapped through `SdlcError::OrchestratorDb(String)` — same pattern as the existing `ActionDb` methods. The HTTP handler converts `SdlcError` to `500 + JSON`.

No new error variants needed.

---

## Testing Strategy

### Unit tests in `db.rs`

```
webhook_insert_and_retrieve_round_trip
  — insert one payload, all_pending_webhooks returns it with exact raw_body

webhook_delete_removes_record
  — insert then delete, all_pending_webhooks returns empty

webhook_multiple_payloads_sorted_by_received_at
  — insert two payloads, verify order

empty_db_all_pending_webhooks_returns_empty
  — baseline
```

### Integration test in `sdlc-server/tests/integration.rs`

```
post_webhook_returns_202_with_id
  — POST /webhooks/test, assert 202, parse UUID from response body
```

---

## Backwards Compatibility

- `ActionDb::open()` creates both `ACTIONS` and `WEBHOOKS` tables. Existing databases without the `WEBHOOKS` table will have it added on first open — redb handles this gracefully (creating a missing table in an existing database succeeds).
- No changes to `Action`, `ActionTrigger`, or `ActionStatus`.
- No changes to existing routes or middleware.
