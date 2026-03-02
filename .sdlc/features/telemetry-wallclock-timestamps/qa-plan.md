# QA Plan: Add Wall-Clock Timestamp to Every Telemetry Event

## Unit Tests (automated, `cargo test`)

### TC-1: `message_to_event` produces `timestamp` on result events
- Construct `Message::Result` with minimal fields.
- Call `message_to_event`.
- Assert `event["timestamp"]` is a `Value::String`.
- Parse with `chrono::DateTime::parse_from_rfc3339` — must succeed without error.

### TC-2: Timestamp ends with `Z` (UTC)
- Same event from TC-1.
- Assert `event["timestamp"].as_str().unwrap().ends_with('Z')`.

### TC-3: `message_to_event` produces `timestamp` on assistant events
- Construct a minimal `Message::Assistant` with one text content block and one tool use.
- Call `message_to_event`.
- Assert `event["timestamp"]` present and parses as RFC-3339.

### TC-4: Timestamp is monotonically non-decreasing across sequential calls
- Call `message_to_event` twice in quick succession.
- Parse both timestamps as `DateTime<Utc>`.
- Assert `t2 >= t1`.

### TC-5: Existing `append_and_retrieve` test still passes
- The existing test in `telemetry.rs` must continue to pass — confirms no regressions in the store layer.

## Integration / Smoke Test (manual, single run)

### TC-6: New run events contain timestamp in sidecar
1. Start `sdlc ui` (or ensure it's running).
2. Trigger any agent run (e.g., start a ponder chat).
3. After run completes, inspect `.sdlc/.runs/<run-id>.events.json`.
4. Assert every object in the array contains a `"timestamp"` key.
5. Assert all values are valid ISO-8601 strings ending in `Z`.

### TC-7: `GET /api/runs/:id/telemetry` returns timestamped events
1. Use the run ID from TC-6.
2. `curl http://localhost:7777/api/runs/<id>/telemetry | jq '.events[].timestamp'`
3. Assert all values are non-null strings.

## Regression Checks

### RC-1: Clippy clean
- `cargo clippy --all -- -D warnings` produces zero warnings or errors.

### RC-2: Full test suite
- `SDLC_NO_NPM=1 cargo test --all` all tests pass.

### RC-3: Old events files not broken
- Load an existing `.events.json` file (pre-change) via `GET /api/runs/:id/telemetry`.
- Confirm the endpoint returns 200 and the events array is present (missing `timestamp` is acceptable for historical records — UI must not crash on null).

## Pass Criteria

All TC-1 through TC-5 automated tests pass. RC-1 and RC-2 pass. TC-6 and TC-7 confirmed in a single manual smoke run. RC-3 confirmed by loading one historical events file.
