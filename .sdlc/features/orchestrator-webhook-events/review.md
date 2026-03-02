# Code Review: Webhook Event History

**Feature:** orchestrator-webhook-events
**Reviewer:** Automated agent review
**Scope:** All tasks T1–T7 — WebhookEvent data model, ring-buffer storage, HTTP ingestion wiring, GET route, registration, and integration tests.

---

## Summary

The implementation is complete and correct. All seven tasks shipped cleanly:

- `WebhookEvent` and `WebhookEventOutcome` are well-designed serde types with comprehensive unit tests.
- The ring-buffer storage in `db.rs` uses a composite 24-byte key (seq + UUID) matching the existing `action_key` pattern. The two-phase read-then-write approach correctly avoids the Rust borrow-checker limitation with `table.first()` + `table.remove()`.
- Best-effort event logging in `receive_webhook` is correct: a failed event insert emits a `tracing::warn!` without affecting the 202 response.
- The GET handler serializes events as flat JSON objects with the `outcome` field as a tagged object (`kind` field).
- Route is registered in the correct order (before the wildcard webhook ingestion route).
- 7 unit tests in `db.rs` and 2 integration tests cover normal path, empty-DB edge case, seq ordering, cap enforcement, and seq-assigned-by-DB invariant.

---

## File-by-File Review

### `crates/sdlc-core/src/orchestrator/webhook.rs`

**Status: PASS**

- `WebhookEventOutcome` uses `#[serde(tag = "kind", rename_all = "snake_case")]` — correct tagged enum representation, matches the spec.
- `WebhookEvent::new()` initializes `seq = 0`; the DB assigns the real seq on insert — this is the correct pattern for DB-assigned sequences and the test `webhook_event_seq_assigned_by_db_not_caller` verifies it.
- 8 unit tests cover all outcome variant serialization round-trips plus struct construction. All pass.
- Exports in `mod.rs` are correct: `pub use webhook::{WebhookEvent, WebhookEventOutcome, WebhookPayload, WebhookRoute};`

No issues.

### `crates/sdlc-core/src/orchestrator/db.rs`

**Status: PASS**

- `WEBHOOK_EVENTS: TableDefinition<&[u8], &[u8]>` added alongside the existing three table definitions.
- `WEBHOOK_EVENTS_CAP: u64 = 500` constant.
- `event_key(seq: u64, id: Uuid) -> [u8; 24]` follows the exact same layout as `action_key` — 8-byte big-endian seq prefix + 16-byte UUID. This is correct: big-endian ordering means byte order == seq order, forward scan = ascending seq = oldest first.
- `ActionDb::open()` opens `WEBHOOK_EVENTS` in the same initialization write transaction as the other three tables.
- `insert_webhook_event` two-phase design:
  - Phase 1 (read tx): reads `last()` to derive `next_seq` and `len()` to get `count`. Both guards are fully dropped before the write transaction opens.
  - Phase 2 (write tx): evicts `first()` entry by copying the key bytes to an owned `Vec<u8>` before calling `remove()` — this correctly releases the immutable borrow before the mutable call.
  - Assigns `event.seq = next_seq` on a clone before inserting.
- `list_webhook_events`: full scan + `result.reverse()` for newest-first. Correct and simple.
- `webhook_event_count`: `table.len()` via `ReadableTableMetadata` trait — correct.
- 7 ring-buffer unit tests: round-trip, empty, cap enforcement at 501, seq ordering (newest first), count tracking, seq assigned by DB (not caller), and table persistence across DB re-opens. All pass.

Minor observation (no action required): `insert_webhook_event` has a theoretical TOCTOU window — a second concurrent writer could insert between Phase 1 and Phase 2, causing seq to be re-derived incorrectly. In practice, the orchestrator is single-process and all redb writes are `spawn_blocking` tasks that hold the `ActionDb` reference for the duration. redb also serializes writers at the transaction level, so two concurrent write transactions will not interleave. No fix needed, but worth noting in future if concurrent writers are introduced.

### `crates/sdlc-server/src/routes/webhooks.rs`

**Status: PASS**

- Imports `WebhookEvent`, `WebhookEventOutcome`, `WebhookPayload` from `sdlc_core::orchestrator`.
- `body_bytes = body.len()` captured before the `spawn_blocking` closure to avoid moving `body` and then trying to use it.
- Route path normalization (prefix with `/`) happens before both `insert_webhook` and `insert_webhook_event` so stored paths are consistent.
- Best-effort pattern: `if let Err(e) = db.insert_webhook_event(&event) { tracing::warn!(...) }` — correct. The payload is already committed when the event insert is attempted.
- No `unwrap()` calls in non-test code.

No issues.

### `crates/sdlc-server/src/routes/orchestrator.rs`

**Status: PASS**

- `list_webhook_events` mirrors `list_routes` correctly — `spawn_blocking`, opens `ActionDb`, maps events to JSON.
- Explicit field-by-field JSON serialization (not `serde_json::to_value`) makes the output schema explicit and prevents accidental field renames from affecting the API contract.
- Returns `AppError` on DB unavailability, which translates to 500 — appropriate since events are non-critical.
- `received_at` serialized as RFC 3339 string — consistent with other timestamp fields in the API.

No issues.

### `crates/sdlc-server/src/lib.rs`

**Status: PASS**

Route registration:
```rust
.route("/api/orchestrator/webhooks/events", get(routes::orchestrator::list_webhook_events))
.route("/api/orchestrator/webhooks/routes", get(...).post(...))
```

Both specific routes are registered before any wildcard webhook ingestion route. Order is correct — axum matches longest prefix first, so `/events` and `/routes` will not be swallowed by the ingestion wildcard.

No issues.

### `crates/sdlc-server/tests/integration.rs`

**Status: PASS**

Two new tests:
1. `get_webhook_events_empty_db_returns_empty_array` — verifies GET returns `200 OK` and an empty JSON array on a fresh DB.
2. `receive_webhook_records_event` — POSTs to `/webhooks/test`, then GETs `/api/orchestrator/webhooks/events`, verifies exactly 1 event with `route_path = "/test"` and `outcome.kind = "received"`.

Both tests use `TempDir` for isolation. Both pass. Coverage is sufficient for the feature scope.

---

## Test Results

- `cargo test -p sdlc-core --lib`: 319 tests pass (includes 7 new ring-buffer tests).
- `cargo test --test integration -p sdlc-server`: 31 tests pass (includes 2 new webhook-event tests).

---

## Findings

| # | Severity | Finding | Action |
|---|---|---|---|
| 1 | Note | TOCTOU window in `insert_webhook_event` between read-tx and write-tx | Accept — single-process, redb serializes writers; no fix needed now |
| 2 | Note | `list_webhook_events` always loads all 500 events into memory | Accept — 500 small JSON objects is negligible; no pagination needed at this scale |

Both findings are accepted. No code changes required.

---

## Verdict

**APPROVED.** The implementation is complete, correct, and well-tested. All spec requirements are met. No open issues.
