# Design: Server Startup Marks Orphaned Runs Failed

## Summary

This is a single-function patch to `load_run_history` in
`crates/sdlc-server/src/state.rs`. No new modules, no new types, no API changes.

## Change

**File:** `crates/sdlc-server/src/state.rs`

**Function:** `load_run_history` (lines ~62–95)

### Current behavior

```rust
if rec.status == "running" {
    rec.status = "stopped".to_string();
    rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
    // best-effort persist
    let _ = std::fs::write(e.path(), serde_json::to_string_pretty(&rec)...);
}
```

### New behavior

```rust
if rec.status == "running" {
    rec.status = "failed".to_string();
    rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
    rec.error = Some("server restarted".to_string());
    // best-effort persist
    let _ = std::fs::write(e.path(), serde_json::to_string_pretty(&rec)...);
}
```

The doc-comment on `load_run_history` is updated from:
> "marking any `running` as `stopped`"

to:
> "marking any `running` as `failed` (orphaned by a server restart)"

## Why not a new status?

`RunStatus` in the frontend already has four values: `running | completed | failed | stopped`.
`failed` is the correct semantic: the run did not complete normally, and it was not
stopped by user intent. Adding a new status (`crashed`, `orphaned`) would require
frontend changes and break all existing consumers. `failed` is the right fit.

## Test Plan

Add a unit test in `crates/sdlc-server/src/state.rs` (in the existing `#[cfg(test)]`
block) that:
1. Creates a temp dir and writes a `RunRecord` JSON with `status = "running"`.
2. Calls `load_run_history`.
3. Asserts returned record has `status == "failed"`.
4. Asserts returned record has `error == Some("server restarted")`.
5. Asserts the JSON file on disk reflects the updated status.
6. Asserts a second record with `status = "completed"` is unchanged.
