# Tasks: Add Wall-Clock Timestamp to Every Telemetry Event

## T1 — Inject timestamp in `message_to_event`

**File:** `crates/sdlc-server/src/routes/runs.rs`

Capture `chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)` before the `match`, bind it as `ts`, then after the match insert `"timestamp"` into the returned `serde_json::Value::Object`. No changes to the per-arm `serde_json::json!` literals.

Estimated effort: 15 minutes.

## T2 — Add unit tests for `message_to_event` timestamp

**File:** `crates/sdlc-server/src/routes/runs.rs` (existing `#[cfg(test)]` block)

Add tests:
- `message_to_event_result_has_timestamp` — construct a minimal `Message::Result`, call `message_to_event`, assert `event["timestamp"]` is a string and parses as RFC-3339.
- `message_to_event_timestamp_ends_with_z` — verify UTC suffix `Z`.
- `message_to_event_assistant_has_timestamp` — verify the `assistant` variant also carries the field (covers the most complex arm).

Estimated effort: 20 minutes.

## T3 — Verify CI passes

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`. Fix any issues. Confirm all tests green.

Estimated effort: 5 minutes.
