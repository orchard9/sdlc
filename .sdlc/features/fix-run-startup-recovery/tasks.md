# Tasks: Server Startup Marks Orphaned Runs Failed

## T1 — Patch `load_run_history` to use `"failed"` status

**File:** `crates/sdlc-server/src/state.rs`

Change the startup recovery block in `load_run_history`:
- Set `rec.status = "failed".to_string()` (was `"stopped"`)
- Set `rec.error = Some("server restarted".to_string())`
- Keep `rec.completed_at = Some(chrono::Utc::now().to_rfc3339())`
- Keep the best-effort `std::fs::write` persist call
- Update the function's doc-comment to say "marking any `running` as `failed`
  (orphaned by a server restart)"

## T2 — Add unit test for orphaned-run recovery

**File:** `crates/sdlc-server/src/state.rs` (existing `#[cfg(test)]` block)

Test name: `orphaned_runs_marked_failed_on_startup`

Steps:
1. Create a `TempDir`.
2. Create the `.sdlc/.runs/` subdirectory.
3. Write two `RunRecord` JSON files:
   - One with `status = "running"` (the orphan)
   - One with `status = "completed"` (should be unchanged)
4. Call `load_run_history(&temp_dir.path())`.
5. Assert the orphaned record has `status == "failed"`.
6. Assert the orphaned record has `error == Some("server restarted")`.
7. Assert the completed record has `status == "completed"` and `error == None`.
8. Assert the orphaned run's JSON file on disk has `status = "failed"`.

## T3 — Verify build and tests pass

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must pass with zero errors.
