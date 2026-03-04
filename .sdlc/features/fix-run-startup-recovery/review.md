# Code Review: Server Startup Marks Orphaned Runs Failed

## Summary

The implementation correctly patches `load_run_history` in
`crates/sdlc-server/src/state.rs` to mark runs that are still `"running"` at server
startup as `"failed"` (not `"stopped"`), and populates the `error` field with
`"server restarted"` so the reason is machine-readable. A unit test verifies both the
in-memory result and the on-disk persistence.

## Diff reviewed

`crates/sdlc-server/src/state.rs` — 4 changes:

1. **Doc comment updated** on `load_run_history`: now says "marking any `running` as
   `failed` (orphaned by a server restart)" — accurately describes the new behavior.

2. **`rec.status`** changed from `"stopped"` to `"failed"` in the startup recovery
   branch — correct; crashes are not user-initiated stops.

3. **`rec.error`** set to `Some("server restarted")` — added the machine-readable
   reason the spec requires. Previously this field was left `None` for orphaned runs,
   which gave no diagnostic signal.

4. **Unit test `orphaned_runs_marked_failed_on_startup`** added to the existing
   `#[cfg(test)]` block — covers:
   - Orphaned run gets `status == "failed"` in returned `Vec`
   - `error == Some("server restarted")` in returned `Vec`
   - `completed_at` is `Some(...)` (non-null timestamp)
   - Orphaned run's `.json` file on disk is updated to `failed`
   - A completed run is left completely unchanged (status, error)

## Quality checks

| Check | Result |
|---|---|
| `SDLC_NO_NPM=1 cargo test -p sdlc-server orphaned_runs_marked_failed_on_startup` | PASS |
| `cargo clippy --all -- -D warnings` | PASS (zero warnings) |

## Findings

No issues found. The change is:
- **Minimal** — two lines changed in the recovery branch plus one `error` assignment
- **Correct** — semantically, crashed runs are failures, not user-initiated stops
- **Safe** — best-effort `fs::write` pattern is unchanged; I/O errors are discarded
  (consistent with existing behavior)
- **Backward-compatible** — `error` field is already `Option<String>` with
  `#[serde(skip_serializing_if = "Option::is_none")]` so existing `.json` files without
  `error` still deserialize correctly
- **Well-tested** — new unit test exercises all acceptance criteria from the spec

## Verdict

APPROVED — ready to merge.
