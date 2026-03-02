# Tasks: Webhook Event History

## T1 — Add `WebhookEvent` and `WebhookEventOutcome` to `webhook.rs`

**File:** `crates/sdlc-core/src/orchestrator/webhook.rs`

Add the `WebhookEvent` struct and `WebhookEventOutcome` enum as specified in the design. Export them from `mod.rs`.

Acceptance: `WebhookEvent` serializes/deserializes via serde. Unit tests for struct construction pass.

---

## T2 — Add `WEBHOOK_EVENTS` table and ring-buffer methods to `db.rs`

**File:** `crates/sdlc-core/src/orchestrator/db.rs`

1. Add `WEBHOOK_EVENTS: TableDefinition<&[u8], &[u8]>` constant.
2. Add `event_key(seq: u64, id: Uuid) -> [u8; 24]` private helper (same layout as `action_key`).
3. Extend `ActionDb::open()` to `open_table(WEBHOOK_EVENTS)` in the same write transaction.
4. Implement `insert_webhook_event(&self, event: &WebhookEvent) -> Result<()>`:
   - Read current max seq (last key in table, big-endian prefix), assign `event.seq = max_seq + 1`.
   - If count >= 500, delete the first (lowest-seq) entry.
   - Insert new entry under the composite key.
5. Implement `list_webhook_events(&self) -> Result<Vec<WebhookEvent>>`:
   - Full scan, collect all entries, reverse to newest-first.
6. Implement `webhook_event_count(&self) -> Result<u64>`.

Unit tests:
- `webhook_event_insert_and_list_round_trip`
- `webhook_event_list_empty_returns_empty`
- `webhook_event_ring_buffer_evicts_oldest_at_501`
- `webhook_event_seq_ordering_newest_first`
- `webhook_event_count_tracks_correctly`
- `existing_db_open_adds_webhook_events_table` (open on DB that predates WEBHOOK_EVENTS)

---

## T3 — Emit events from `receive_webhook` handler

**File:** `crates/sdlc-server/src/routes/webhooks.rs`

After `db.insert_webhook(&payload)` succeeds, call `db.insert_webhook_event()` with a `WebhookEvent` built from the payload (outcome: `Received`). Do not fail the HTTP response if the event insert fails — log a warning and continue (the payload is already stored; event logging is best-effort).

---

## T4 — Add `GET /api/orchestrator/webhooks/events` route handler

**File:** `crates/sdlc-server/src/routes/orchestrator.rs`

Add `list_webhook_events` handler mirroring `list_routes`. Returns 200 with JSON array (newest-first) or 503 if DB unavailable. Serialize each `WebhookEvent` as a flat JSON object (not nested serde tag — keep the `outcome` field as a tagged object with `kind`).

---

## T5 — Register the new route in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

Add:
```rust
.route(
    "/api/orchestrator/webhooks/events",
    get(routes::orchestrator::list_webhook_events),
)
```
Place this before the `/api/orchestrator/webhooks/routes` routes to avoid any future wildcard conflicts.

---

## T6 — Integration tests for the new endpoint

**File:** `crates/sdlc-server/tests/integration.rs`

Add:
- `get_webhook_events_no_db_returns_503`
- `get_webhook_events_empty_db_returns_empty_array`
- `receive_webhook_records_event`: POST `/webhooks/test` then GET `/api/orchestrator/webhooks/events`, assert one event with `route_path = "/test"` and `outcome.kind = "received"`.

---

## T7 — Run full test suite and fix any issues

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All tests must pass. Fix any compilation errors or clippy warnings before marking done.
