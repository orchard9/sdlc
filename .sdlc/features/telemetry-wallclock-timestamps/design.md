# Design: Add Wall-Clock Timestamp to Every Telemetry Event

## Overview

This is a minimal, focused change to the event serialization layer. A single function — `message_to_event` in `crates/sdlc-server/src/routes/runs.rs` — is the sole code path that converts SDK `Message` values into the JSON objects that flow through the SSE broadcast, the `TelemetryStore`, and the `.events.json` sidecars. Adding the timestamp here propagates it everywhere automatically.

## Change Location

**File:** `crates/sdlc-server/src/routes/runs.rs`
**Function:** `message_to_event` (line ~643)

## Implementation

The function currently returns a `serde_json::Value` from a `match` on all `Message` variants. The change wraps that return value with a post-processing step that inserts the `timestamp` field:

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

Key properties:
- `chrono::SecondsFormat::Millis` produces millisecond precision, e.g. `"2026-03-02T07:15:00.123Z"`.
- The `use_z` flag is `true` so the suffix is `Z` (UTC) rather than `+00:00`.
- The existing `match` arms require zero edits — timestamps are injected uniformly after the match.
- The `if let Some(obj)` guard is a no-op safety check; all existing arms produce `Value::Object`.

## Timestamp Precision Rationale

Millisecond precision (`SecondsFormat::Millis`) is chosen because:
- Sub-millisecond precision adds noise without benefit for human-readable timelines.
- The `tool_progress` event already exposes `elapsed_seconds` as a float for fine-grained tool timing; `timestamp` is for calendar-time anchoring, not high-frequency profiling.
- Milliseconds are directly usable in JavaScript `Date.parse()` and standard log parsers.

## Data Flow

```
SDK Message stream
      |
      v
message_to_event()   <-- timestamp injected HERE
      |
      +---> SSE broadcast (live subscribers see timestamped events)
      |
      +---> accumulated_events Vec
                  |
                  +---> TelemetryStore.append_raw()  (redb)
                  |
                  +---> persist_run_events()  (.events.json sidecar)
```

## No Schema Migration Required

`.events.json` files are append-only logs consumed by the UI and the `GET /api/runs/:id/telemetry` endpoint. Older files without `timestamp` continue to work — the UI should treat a missing `timestamp` as `null` and fall back gracefully. No migration or versioning field is added; the presence or absence of `timestamp` is the version signal.

## Dependencies

`chrono` is already in `sdlc-server/Cargo.toml` (used for `generate_run_id` and `completed_at` in `state.rs`). No new dependencies are required.

## Testing

New unit tests are added in the `#[cfg(test)]` block in `runs.rs`:

1. **`message_to_event_has_timestamp`** — calls `message_to_event` with a `Message::Result` and asserts `event["timestamp"]` is a non-empty string parseable by `chrono::DateTime::parse_from_rfc3339`.
2. **`message_to_event_timestamp_is_utc`** — verifies the timestamp string ends with `Z`.
3. **`all_event_types_have_timestamp`** — parametrized over representative message variants (Init, Assistant, User, Result, ToolProgress) to confirm every arm produces a `timestamp` key.
