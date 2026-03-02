# Design: Webhook Event History

## Architecture Overview

This feature extends the existing orchestrator data layer with a new `WEBHOOK_EVENTS` redb table and supporting types. The change is additive — no existing tables or APIs are modified, only new methods and a new route are added.

```
External Sender
      │
      │ POST /webhooks/{route}
      ▼
routes/webhooks.rs::receive_webhook()
      │
      ├── db.insert_webhook()           [WEBHOOKS table — existing, unchanged]
      │
      └── db.insert_webhook_event()     [WEBHOOK_EVENTS table — NEW]
               outcome: Received

Tick loop (sdlc orchestrate)
      │
      ├── db.all_pending_webhooks()     [existing]
      ├── match route → dispatch tool   [existing]
      │
      └── db.insert_webhook_event()     [NEW — per dispatch outcome]
               outcome: Routed | NoRoute | DispatchError

GET /api/orchestrator/webhooks/events
      │
      └── db.list_webhook_events()      [WEBHOOK_EVENTS, newest-first]
```

## Key Design Decisions

### 1. Append-only events, not in-place updates

Rather than updating an existing record's outcome field, each state transition is recorded as a new `WebhookEvent`. The `received_at` timestamp and `route_path` are sufficient to correlate arrival and dispatch events for display purposes.

Rationale: redb has no row-level update — updating requires remove + reinsert with the same key. Since the key encodes the sequence number (assigned at write time), there is no stable key to reinsert under without renumbering. Append-only avoids this complexity entirely.

### 2. Composite key: seq (u64 big-endian) + UUID (16 bytes)

Key layout matches the `ACTIONS` table pattern:

```
[ seq: u64 big-endian (8 bytes) | uuid: 16 bytes ]
```

Byte ordering = sequence ordering. The full table scan for `list_webhook_events` iterates in key order (ascending seq), which is reversed in application code to produce newest-first output.

### 3. Ring buffer enforced at insert time

On `insert_webhook_event`:
1. Count entries in `WEBHOOK_EVENTS`.
2. If count >= 500, read and delete the first (lowest-seq) entry.
3. Insert the new entry.

All three operations occur within a single write transaction, making the eviction atomic.

### 4. Sequence number assignment at insert time

The `ActionDb::insert_webhook_event` method assigns `seq`. It reads the last key in `WEBHOOK_EVENTS` (highest seq), extracts the 8-byte prefix, interprets it as u64, and increments by 1. If the table is empty, seq = 1.

This is safe under the single-writer constraint (redb write transactions are exclusive by the `Mutex<ActionDb>` wrapper in `AppState`).

### 5. No raw body storage in `WebhookEvent`

The event log records metadata only: route_path, content_type, body_bytes (size), received_at, outcome. This keeps the ring buffer size predictable regardless of payload size.

Raw bodies remain available in the `WEBHOOKS` table until dispatched.

## File Changes

### `crates/sdlc-core/src/orchestrator/webhook.rs`

Add:
- `WebhookEvent` struct
- `WebhookEventOutcome` enum (Received, Routed, NoRoute, DispatchError)
- `pub use webhook::{..., WebhookEvent, WebhookEventOutcome}` in `mod.rs`

### `crates/sdlc-core/src/orchestrator/db.rs`

Add:
- `WEBHOOK_EVENTS: TableDefinition<&[u8], &[u8]>` constant
- `event_key(seq: u64, id: Uuid) -> [u8; 24]` private helper
- `ActionDb::insert_webhook_event(&self, event: &WebhookEvent) -> Result<()>`
- `ActionDb::list_webhook_events(&self) -> Result<Vec<WebhookEvent>>`
- `ActionDb::webhook_event_count(&self) -> Result<u64>`
- `open()` — ensure `WEBHOOK_EVENTS` table is created on first open
- Unit tests for all new methods

### `crates/sdlc-server/src/routes/orchestrator.rs`

Add:
- `list_webhook_events` handler: `GET /api/orchestrator/webhooks/events`

### `crates/sdlc-server/src/lib.rs`

Add route:
```rust
.route(
    "/api/orchestrator/webhooks/events",
    get(routes::orchestrator::list_webhook_events),
)
```
(before the existing `/api/orchestrator/webhooks/routes` routes)

### `crates/sdlc-server/src/routes/webhooks.rs`

Extend `receive_webhook` to call `db.insert_webhook_event()` after `db.insert_webhook()`.

### `crates/sdlc-server/tests/integration.rs`

Add integration tests for `GET /api/orchestrator/webhooks/events`.

### `crates/sdlc-core/src/orchestrator/mod.rs`

Export new types:
```rust
pub use webhook::{WebhookEvent, WebhookEventOutcome, WebhookPayload, WebhookRoute};
```

## API Response Shape

`GET /api/orchestrator/webhooks/events` → 200 OK:

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "seq": 42,
    "route_path": "/hooks/github",
    "content_type": "application/json",
    "body_bytes": 1234,
    "received_at": "2026-03-02T10:00:00Z",
    "outcome": {
      "kind": "received"
    }
  },
  {
    "id": "...",
    "seq": 41,
    "route_path": "/hooks/stripe",
    "content_type": "application/json",
    "body_bytes": 512,
    "received_at": "2026-03-02T09:55:00Z",
    "outcome": {
      "kind": "routed",
      "route_id": "...",
      "tool_name": "stripe-handler"
    }
  }
]
```

Error (503):
```json
{ "error": "orchestrator DB not available" }
```

## Testing Strategy

### Unit tests (in `db.rs`)

- `webhook_event_insert_and_list_round_trip` — insert one event, list returns it.
- `webhook_event_list_empty_returns_empty` — fresh DB returns empty vec.
- `webhook_event_ring_buffer_evicts_oldest_at_501` — insert 501 events, list length = 500, oldest (seq=1) is gone.
- `webhook_event_seq_ordering_newest_first` — insert 3 events, verify list order.
- `webhook_event_count_tracks_correctly` — count after inserts and after ring-buffer eviction.

### Integration tests (in `crates/sdlc-server/tests/integration.rs`)

- `get_webhook_events_no_db_returns_503` — request without DB.
- `get_webhook_events_empty_db_returns_empty_array` — DB present but no events.
- `receive_webhook_records_event` — POST to `/webhooks/test`, then GET `/api/orchestrator/webhooks/events`, verify one event present.

## Sequence Diagram: Webhook Arrival

```
Client          Server (receive_webhook)     ActionDb         WEBHOOK_EVENTS table
  │                       │                     │                     │
  │ POST /webhooks/github  │                     │                     │
  │──────────────────────►│                     │                     │
  │                       │ insert_webhook()     │                     │
  │                       │─────────────────────►│                     │
  │                       │                     │ write WEBHOOKS       │
  │                       │ insert_webhook_event(outcome=Received)     │
  │                       │─────────────────────►│                     │
  │                       │                     │ count, evict if ≥500 │
  │                       │                     │─────────────────────►│
  │                       │                     │ write WEBHOOK_EVENTS │
  │                       │                     │◄─────────────────────│
  │                       │ Ok(id)               │                     │
  │                       │◄─────────────────────│                     │
  │ 202 { "id": "..." }   │                     │                     │
  │◄──────────────────────│                     │                     │
```
