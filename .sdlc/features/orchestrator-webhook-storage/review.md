# Code Review: HTTP Webhook Receiver and Raw Payload Storage in redb

## Summary

Implementation is complete and correct. All six tasks delivered. `cargo test --all` and `cargo clippy --all -- -D warnings` both pass clean.

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/orchestrator/webhook.rs` | New — `WebhookPayload` struct with `new()` constructor |
| `crates/sdlc-core/src/orchestrator/mod.rs` | Added `pub mod webhook` + re-export |
| `crates/sdlc-core/src/orchestrator/db.rs` | Added `WEBHOOKS` table, `insert_webhook`, `all_pending_webhooks`, `delete_webhook`, 6 unit tests |
| `crates/sdlc-server/src/routes/webhooks.rs` | New — `receive_webhook` handler |
| `crates/sdlc-server/src/routes/mod.rs` | Added `pub mod webhooks` |
| `crates/sdlc-server/src/lib.rs` | Registered `POST /webhooks/{route}` route |
| `crates/sdlc-server/tests/integration.rs` | Added `post_webhook_returns_202_with_id` and `post_webhook_preserves_raw_body_bytes` |

---

## Correctness

**Data model:** `WebhookPayload` fields match the spec exactly — `id: Uuid`, `route_path: String`, `raw_body: Vec<u8>`, `received_at: DateTime<Utc>`, `content_type: Option<String>`. UUID generated at construction, timestamp set to `Utc::now()`.

**Storage key:** 16-byte raw UUID bytes, consistent with the design. Not a composite timestamp key — correct for lookup-by-ID delete-after-dispatch semantics.

**Table initialization:** `ActionDb::open()` now opens both `ACTIONS` and `WEBHOOKS` in the same write transaction. Existing databases gain the `WEBHOOKS` table on first open without data loss — verified by `existing_db_open_adds_webhooks_table` test.

**Raw body preservation:** `Bytes` from axum is converted to `Vec<u8>` via `.to_vec()` — no encoding, no decoding. The `post_webhook_preserves_raw_body_bytes` integration test confirms binary content (including null bytes and `0xff`) round-trips intact.

**HTTP status:** Returns `202 Accepted` (not `200 OK`) on success — correct for an asynchronous ingestion endpoint.

**Error handling:** Both DB open failure and insert failure map to `500 Internal Server Error` with a JSON `{ "error": "..." }` body. Task join panics also produce `500`. Consistent with server patterns.

**Idempotency of delete:** `delete_webhook` uses `table.remove()` which silently succeeds on missing keys — confirmed by `webhook_delete_nonexistent_is_idempotent` test.

---

## Quality

**No `unwrap()` in library code:** All `db.rs` methods use `map_err(|e| SdlcError::OrchestratorDb(e.to_string()))` + `?`. The `webhook.rs` handler uses `spawn_blocking` + explicit `match` on the result.

**Clippy clean:** Zero warnings at `-D warnings` level.

**Test coverage:**
- 6 unit tests in `db.rs` covering insert/retrieve round-trip, delete, ordering, empty DB, re-open, and idempotent delete
- 2 integration tests covering HTTP 202 response and raw body fidelity

---

## Spec Compliance

| Acceptance Criterion | Status |
|---|---|
| `POST /webhooks/github` returns 202 with UUID | Verified by `post_webhook_returns_202_with_id` |
| `insert_webhook` → `all_pending_webhooks` returns payload with exact raw bytes | Verified by unit + integration tests |
| `delete_webhook(id)` removes the payload | Verified by `webhook_delete_removes_record` |
| `cargo test --all` passes | Passed (27 integration tests ok) |
| `cargo clippy --all -- -D warnings` passes | Passed (0 warnings) |

---

## No Issues Found

The implementation is minimal, correct, and consistent with existing patterns. No tech debt introduced. The `WEBHOOKS` table is fully isolated from `ACTIONS` — no risk of interference with the existing tick loop.

**Recommendation: Approve.**
