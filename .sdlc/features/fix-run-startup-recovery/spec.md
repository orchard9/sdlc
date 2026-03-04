# Spec: Server Startup Marks Orphaned Runs Failed

## Problem

When the `sdlc-server` process restarts (crash, OS reboot, `kill`, deployment), any
agent runs that were in the `"running"` state at shutdown are left in an ambiguous
terminal state. The current `load_run_history` implementation marks these orphaned
runs as `"stopped"` — the same status used when a user deliberately cancels a run via
the stop button.

This conflation means:
- The UI shows crashed runs as if the user stopped them on purpose.
- Monitoring, analytics, and the activity feed cannot distinguish intentional stops
  from unexpected process deaths.
- The quota/cost panel treats crashed runs identically to user-stopped ones, hiding
  reliability signals.

## Goal

On server startup, any `RunRecord` with `status == "running"` that survived the
restart is an orphaned (crashed) run. It must be marked `"failed"` (not `"stopped"`)
so that:
1. The UI renders orphaned runs in red (the existing `failed` badge color) rather than
   the neutral `stopped` treatment.
2. Operators can distinguish crash-derived failures from deliberate stops in the run
   history and logs.
3. The distinction is stored persistently in `.sdlc/.runs/*.json` so it survives
   future restarts.

## Scope

- **In scope:** Change `load_run_history` in `crates/sdlc-server/src/state.rs` to
  set `status = "failed"` (and a non-null `completed_at`) for any record that is
  still `"running"` at load time.
- **In scope:** Add an `error` field value of `"server restarted"` (or similar) to
  the persisted record so the reason is machine-readable.
- **In scope:** Add a unit test in `state.rs` or the integration test suite that
  verifies orphaned-run recovery sets `status = "failed"`.
- **Out of scope:** Changes to the `"stopped"` status path used by `stop_run_by_key`
  (user-initiated stops must remain `"stopped"`).
- **Out of scope:** Frontend changes — the existing `failed` badge color is already
  correct for this status.
- **Out of scope:** SSE events — no broadcast is needed because the server has no
  connected clients at startup time.

## Acceptance Criteria

1. After a simulated crash (write a `RunRecord` with `status = "running"` to disk,
   then call `load_run_history`), the returned record has `status = "failed"`.
2. The `error` field of the returned record contains a human-readable crash reason
   (e.g., `"server restarted"`).
3. The persisted JSON file on disk is updated to reflect `status = "failed"`.
4. Records with `status != "running"` are unaffected.
5. A unit test covers all of the above.
6. `cargo test --all` passes with `SDLC_NO_NPM=1`.
7. `cargo clippy --all -- -D warnings` passes.

## Implementation Notes

The change is localized to `load_run_history` in
`crates/sdlc-server/src/state.rs` lines 79–88:

```rust
// Before
if rec.status == "running" {
    rec.status = "stopped".to_string();
    // ...
}

// After
if rec.status == "running" {
    rec.status = "failed".to_string();
    rec.error = Some("server restarted".to_string());
    // ...
}
```

The doc-comment on `load_run_history` must be updated to reflect the new behavior.
