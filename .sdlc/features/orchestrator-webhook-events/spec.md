# Spec: Webhook Event History

## Overview

The orchestrator currently stores raw `WebhookPayload` records in the `WEBHOOKS` redb table. These payloads are consumed (dispatched) and deleted by the tick loop. Once dispatched, there is no historical record of what happened — what arrived, when, which route matched, whether dispatch succeeded or failed.

This feature adds a `WEBHOOK_EVENTS` table to the `ActionDb` backed by a fixed-size ring buffer (capped at 500 entries). Every webhook arrival and dispatch outcome is appended as a `WebhookEvent`. A `GET /api/orchestrator/webhooks/events` route exposes the event log to the frontend and to API consumers.

## Goals

1. Record every webhook arrival with its route path, content type, body size, and timestamp.
2. Record the dispatch outcome (routed / no_route / dispatch_error) with error detail when applicable.
3. Enforce a 500-record ring buffer: when the buffer is full, the oldest event is evicted before inserting the new one.
4. Expose a `GET /api/orchestrator/webhooks/events` endpoint returning events sorted newest-first.
5. Integrate event writes into the existing `receive_webhook` handler and the tick-loop dispatch path.

## Non-Goals

- No full replay of raw bodies (body bytes are NOT stored in `WebhookEvent`; only metadata is stored).
- No streaming/SSE push of events (GET poll is sufficient for v1).
- No per-route filtering in the query endpoint for v1.
- No persistence of events across DB deletion (events live with the redb file).

## Data Model

### `WebhookEvent` struct (in `crates/sdlc-core/src/orchestrator/webhook.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Unique identifier — also used as the redb sort key (u64 sequence number + UUID).
    pub id: Uuid,
    /// Monotonically incrementing sequence number, used for ring-buffer eviction.
    pub seq: u64,
    /// The normalized route path from the incoming request (e.g. `/hooks/github`).
    pub route_path: String,
    /// Content-Type from the request, if present.
    pub content_type: Option<String>,
    /// Byte size of the raw body received.
    pub body_bytes: usize,
    /// RFC 3339 timestamp when the payload was received.
    pub received_at: DateTime<Utc>,
    /// Outcome of dispatch.
    pub outcome: WebhookEventOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WebhookEventOutcome {
    /// Payload was matched against a route and dispatched successfully.
    Routed {
        route_id: String,
        tool_name: String,
    },
    /// No route registered for the path.
    NoRoute,
    /// Route matched but dispatch failed.
    DispatchError { reason: String },
    /// Payload arrived — dispatch outcome not yet recorded (intermediate state).
    Received,
}
```

### `WEBHOOK_EVENTS` redb table

Key: 24-byte composite — `seq: u64` big-endian (8 bytes) + `id: Uuid` (16 bytes).

This mirrors the `ACTIONS` table design. Because `seq` occupies the high bytes in big-endian encoding, byte ordering equals sequence ordering. This enables:

- O(1) oldest-record lookup (first entry in table).
- O(1) newest-record lookup (last entry in table).
- Ring-buffer eviction: on insert, if `count >= 500`, remove the first (lowest-seq) entry before inserting the new one.

Value: JSON-encoded `WebhookEvent`.

## API

### `GET /api/orchestrator/webhooks/events`

Returns all `WebhookEvent` records sorted by `seq` descending (newest first), as a JSON array.

Response shape:
```json
[
  {
    "id": "...",
    "seq": 42,
    "route_path": "/hooks/github",
    "content_type": "application/json",
    "body_bytes": 1234,
    "received_at": "2026-03-02T10:00:00Z",
    "outcome": {
      "kind": "routed",
      "route_id": "...",
      "tool_name": "ci-notify"
    }
  }
]
```

Status codes:
- `200 OK` — success (may return empty array).
- `503 Service Unavailable` — orchestrator DB not available.

### Integration with `receive_webhook`

The existing `POST /webhooks/{route}` handler (`routes/webhooks.rs`) currently calls `db.insert_webhook()`. After this feature:

1. On successful `insert_webhook`: also call `db.insert_webhook_event()` with `outcome: Received`.
2. On `insert_webhook` failure: still try to record an event with `outcome: DispatchError { reason }`.

### Integration with the tick loop dispatch path

When the tick loop dispatches a webhook (in `sdlc orchestrate`):

1. After successful dispatch: call `db.update_webhook_event_outcome(id, Routed { ... })`.
2. If no route found: call `db.update_webhook_event_outcome(id, NoRoute)`.
3. If dispatch fails: call `db.update_webhook_event_outcome(id, DispatchError { reason })`.

However, updating in-place is awkward with the seq-based key. Instead, the simpler design: insert a new event for each outcome transition rather than updating. The `received_at` timestamp links the arrival event to the dispatch event via shared `route_path` + proximity in time. This avoids the complexity of a mutable key-value store.

Revised approach: each `WebhookEvent` represents a single moment with a single `outcome`. The arrival event has `outcome: Received`. The dispatch event is a separate record with `outcome: Routed | NoRoute | DispatchError`.

## `ActionDb` additions

```rust
impl ActionDb {
    /// Insert a webhook event, enforcing the 500-record ring buffer.
    pub fn insert_webhook_event(&self, event: &WebhookEvent) -> Result<()>;

    /// Return all webhook events sorted by seq descending (newest first).
    pub fn list_webhook_events(&self) -> Result<Vec<WebhookEvent>>;

    /// Return the current event count in WEBHOOK_EVENTS.
    pub fn webhook_event_count(&self) -> Result<u64>;
}
```

The `WebhookEvent::seq` field is assigned by `ActionDb::insert_webhook_event` — it reads the current max seq from the table (or 0 if empty), increments by 1, and uses that value.

Ring buffer eviction: before inserting, check count. If `count >= 500`, iterate to the first entry and remove it.

## Acceptance Criteria

1. `db.insert_webhook_event()` inserts a record and `db.list_webhook_events()` returns it.
2. After 501 inserts, `db.list_webhook_events().len() == 500` (oldest evicted).
3. `GET /api/orchestrator/webhooks/events` returns `200` with an array of events when DB is available.
4. `GET /api/orchestrator/webhooks/events` returns `503` when no DB is attached.
5. `receive_webhook` handler writes a `WebhookEvent` with `outcome: Received` on every successful `insert_webhook`.
6. Seq ordering is preserved: events are returned newest-first.
7. All existing tests continue to pass.
8. New unit tests cover: insert+list round trip, ring buffer overflow eviction, seq ordering, `list_webhook_events` empty case.
