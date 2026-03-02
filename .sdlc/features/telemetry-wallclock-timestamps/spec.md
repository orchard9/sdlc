# Spec: Add Wall-Clock Timestamp to Every Telemetry Event

## Problem

Every telemetry event stored by the `TelemetryStore` and persisted in `.sdlc/.runs/*.events.json` sidecars lacks a wall-clock timestamp. The only ordering information available is the event's sequence number within a run (`seq`). This means:

- The activity feed UI cannot display absolute timestamps per event (only relative sequence order).
- Duration calculations between events (e.g., "how long did this tool call take?") require external correlation — the `elapsed_seconds` field on `tool_progress` events is the only time signal and it counts from the start of that tool call, not from epoch.
- Log replay and post-mortem analysis cannot place events on an absolute timeline.

## Goal

Stamp every telemetry event with a UTC ISO-8601 wall-clock timestamp at the moment the event is emitted by `message_to_event`. The timestamp is injected unconditionally into the JSON object for every event type.

## Scope

### In scope

- Add a `timestamp` field (RFC-3339 / ISO-8601 UTC, e.g. `"2026-03-02T07:15:00.123Z"`) to the output of `message_to_event` in `crates/sdlc-server/src/routes/runs.rs`.
- The timestamp is set to `chrono::Utc::now()` at the time `message_to_event` is called, which is the moment the agent SDK delivers the event to the streaming loop.
- The change applies to all event variants: `init`, `status`, `subagent_started`, `subagent_progress`, `subagent_completed`, `assistant`, `user`, `result`, `tool_progress`, `tool_summary`, `stream_event`, `auth_status`, and the catch-all `system`.
- New events persisted to `.events.json` sidecars will carry the timestamp.
- The existing `TelemetryStore.append_raw` path in `telemetry.rs` already stores the full JSON value — no changes needed there; the timestamp arrives pre-stamped.

### Out of scope

- Back-filling timestamps on existing `.events.json` files.
- Exposing timestamp-based filtering APIs.
- UI changes — the timestamp will be present in the JSON and is available to UI consumers, but no UI work is required for this feature.
- Changes to `RunRecord` (which already records `started_at` / `completed_at` in RFC-3339).

## Acceptance Criteria

1. Every event object returned by `message_to_event` contains a `"timestamp"` key whose value is a valid RFC-3339 UTC string.
2. Newly written `.events.json` sidecar files contain `timestamp` on every event object.
3. The `GET /api/runs/:id/telemetry` endpoint returns events with `timestamp` for any run that started after this change was deployed.
4. All existing tests pass. New tests verify that `message_to_event` produces a `timestamp` field and that its value parses as a valid RFC-3339 datetime.
5. `cargo clippy --all -- -D warnings` passes with no new warnings.

## Design Notes

The timestamp is injected in `message_to_event` rather than in `append_raw` because:
- `message_to_event` is the single source of truth for the event shape consumed by the SSE broadcast, the sidecar persister, and the activity feed.
- Stamping at `message_to_event` time makes the timestamp as accurate as possible (the moment the SDK message arrives in the streaming loop).
- Injecting in `append_raw` would stamp only the persisted copy, not the live SSE stream, so real-time subscribers would still see events without timestamps.

The implementation approach for `message_to_event` is:

```rust
fn message_to_event(msg: &Message) -> serde_json::Value {
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut event = match msg { /* existing match */ };
    if let Some(obj) = event.as_object_mut() {
        obj.insert("timestamp".to_string(), serde_json::json!(ts));
    }
    event
}
```

This approach requires zero changes to the per-arm `serde_json::json!` literals and handles the catch-all arms automatically.
