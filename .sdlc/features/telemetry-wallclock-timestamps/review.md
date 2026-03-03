# Code Review: Add Wall-Clock Timestamp to Every Telemetry Event

## Summary

This review covers the implementation of `telemetry-wallclock-timestamps` — a focused change
that stamps every telemetry event produced by `message_to_event` with a UTC RFC-3339
wall-clock timestamp.

## Change Location

**File:** `crates/sdlc-server/src/routes/runs.rs`
**Function:** `message_to_event` (lines 648–794)

## Implementation Review

### Core Change

```rust
fn message_to_event(msg: &Message) -> serde_json::Value {
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut event = match msg {
        // ... all existing arms unchanged ...
    };
    if let Some(obj) = event.as_object_mut() {
        obj.insert("timestamp".to_string(), serde_json::json!(ts));
    }
    event
}
```

**Assessment: Correct.** The approach captures the timestamp before the match (minimizing
any drift between the SDK event arrival and the timestamp call), then injects it uniformly
after all match arms. This is exactly the pattern described in the spec and design artifacts.

### Timestamp Format

- `chrono::SecondsFormat::Millis` — millisecond precision, e.g. `"2026-03-02T07:15:00.123Z"`
- `use_z = true` — UTC suffix is `Z`, not `+00:00`
- Parseable by `chrono::DateTime::parse_from_rfc3339`, JavaScript `Date.parse()`, and
  standard log parsers

**Assessment: Correct.** The format matches the spec requirements.

### Safety

- `if let Some(obj) = event.as_object_mut()` — defensive guard that is a no-op in practice
  (all 13 existing arms produce `serde_json::Value::Object`). Safe to have; no correctness
  concern if a future arm accidentally returns a non-object.
- No new `unwrap()` calls — compliant with project coding convention.

**Assessment: Clean.**

### Dependencies

- `chrono` is already in `sdlc-server/Cargo.toml` — no new dependencies introduced.

**Assessment: No change to dependency graph.**

## Test Coverage

Four new unit tests were added in the existing `#[cfg(test)]` block:

| Test | Asserts |
|---|---|
| `message_to_event_result_has_timestamp` | `timestamp` is a non-empty string parseable as RFC-3339 |
| `message_to_event_timestamp_ends_with_z` | `timestamp` ends with `Z` (UTC) |
| `message_to_event_assistant_has_timestamp` | `assistant` variant carries RFC-3339 timestamp |
| `message_to_event_tool_progress_has_timestamp` | `tool_progress` variant carries RFC-3339 timestamp |

All four tests pass. Together they cover:
- Acceptance criteria 4 (tests verify `message_to_event` produces `timestamp`)
- The three representative message variants called out in the tasks artifact

**Assessment: Adequate coverage for the change scope.**

## Acceptance Criteria Verification

| # | Criterion | Status |
|---|---|---|
| 1 | Every event object from `message_to_event` has `"timestamp"` (valid RFC-3339 UTC) | PASS — injected uniformly after match; tested |
| 2 | Newly written `.events.json` files contain `timestamp` on every event | PASS — `append_raw` path receives pre-stamped values |
| 3 | `GET /api/runs/:id/telemetry` returns events with `timestamp` for runs after this change | PASS — same code path |
| 4 | All existing tests pass; new tests verify `timestamp` field and RFC-3339 parsability | PASS — 4 new tests, full test suite green |
| 5 | `cargo clippy --all -- -D warnings` passes with no new warnings | PASS |

## Findings

No blockers or issues found. The implementation is minimal, focused, and matches the
specification and design exactly. Pre-commit hooks (fmt + clippy + full test suite) passed
on commit.

## Verdict

APPROVED — no issues. The change is correct, safe, and well-tested.
