# QA Plan: Webhook Event History

## Scope

Verify the `WEBHOOK_EVENTS` redb table, the `WebhookEvent` / `WebhookEventOutcome` types, the ring-buffer eviction logic, and the `GET /api/orchestrator/webhooks/events` endpoint.

## Test Layers

### Layer 1 — Unit Tests (in `crates/sdlc-core/src/orchestrator/db.rs`)

| # | Test | Pass Criteria |
|---|------|--------------|
| U1 | `webhook_event_insert_and_list_round_trip` | Insert one event; `list_webhook_events()` returns exactly that event with all fields preserved. |
| U2 | `webhook_event_list_empty_returns_empty` | Fresh DB; `list_webhook_events()` returns `Ok(vec![])`. |
| U3 | `webhook_event_ring_buffer_evicts_oldest_at_501` | Insert 501 events; `list_webhook_events().len() == 500`; the event with `seq == 1` is absent. |
| U4 | `webhook_event_seq_ordering_newest_first` | Insert 3 events; verify list is ordered newest-first (descending `seq`). |
| U5 | `webhook_event_count_tracks_correctly` | After 5 inserts: count == 5. After 501 inserts: count == 500. |
| U6 | `existing_db_open_adds_webhook_events_table` | Open a DB, insert event, close, reopen — data survives and no error is returned. |
| U7 | `webhook_event_outcome_serialization` | Each `WebhookEventOutcome` variant round-trips through JSON correctly with expected `kind` tag. |

### Layer 2 — Integration Tests (in `crates/sdlc-server/tests/integration.rs`)

| # | Test | Pass Criteria |
|---|------|--------------|
| I1 | `get_webhook_events_no_db_returns_503` | `GET /api/orchestrator/webhooks/events` without DB → 503 with `{"error": ...}`. |
| I2 | `get_webhook_events_empty_db_returns_empty_array` | DB present, no events → 200, body is `[]`. |
| I3 | `receive_webhook_records_event` | `POST /webhooks/test` with JSON body → 202; then `GET /api/orchestrator/webhooks/events` → 200, array contains one event with `route_path = "/test"`, `outcome.kind = "received"`, correct `body_bytes`. |
| I4 | `webhook_events_newest_first_order` | POST two webhooks; GET events → first element has higher `seq` than second. |

### Layer 3 — Compilation and Static Analysis

| # | Check | Pass Criteria |
|---|-------|--------------|
| C1 | `SDLC_NO_NPM=1 cargo test --all` | Zero test failures. |
| C2 | `cargo clippy --all -- -D warnings` | Zero warnings. |
| C3 | No `unwrap()` in new library code | Grep `crates/sdlc-core/src/orchestrator/` for `.unwrap()` — none in non-test code. |

## Edge Cases

| Scenario | Expected Behaviour |
|----------|--------------------|
| Ring buffer boundary: exactly 500 events | No eviction; count stays at 500. |
| Ring buffer boundary: 500 → 501 | Evict seq=1; count stays at 500. |
| Multiple rapid inserts with same `route_path` | Each gets a unique UUID and incrementing `seq`. |
| DB mutex poisoned (test-only scenario) | `insert_webhook_event` returns `Err(OrchestratorDb("mutex poisoned"))`. |
| `receive_webhook` with DB unavailable | Handler returns 503 before reaching event insert — no panic. |
| `receive_webhook` where event insert fails | Handler still returns 202 (event logging is best-effort); warning logged. |

## Regression

Run the full existing test suite before and after the change. All previously passing tests must continue to pass. Pay special attention to:

- `webhook_insert_and_retrieve_round_trip`
- `webhook_delete_removes_record`
- `existing_db_open_adds_webhooks_table`
- `existing_db_open_adds_webhook_routes_table`

These existing tests must not be broken by the addition of `WEBHOOK_EVENTS`.

## Sign-off Criteria

QA passes when:
1. All Layer 1 unit tests (U1–U7) pass.
2. All Layer 2 integration tests (I1–I4) pass.
3. All Layer 3 checks (C1–C3) pass.
4. All pre-existing tests in `db.rs` and `integration.rs` continue to pass.
