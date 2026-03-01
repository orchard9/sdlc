# Code Review: uat-run-persistence

## Summary

Reviewed changes in `crates/sdlc-core/src/paths.rs` and `crates/sdlc-core/src/milestone.rs`.

## Changes Reviewed

### `crates/sdlc-core/src/paths.rs`

Added three path helpers:
- `milestone_uat_runs_dir(root, slug)` — `.sdlc/milestones/<slug>/uat-runs/`
- `uat_run_dir(root, milestone_slug, run_id)` — `.sdlc/milestones/<slug>/uat-runs/<id>/`
- `uat_run_manifest(root, milestone_slug, run_id)` — `.sdlc/milestones/<slug>/uat-runs/<id>/run.yaml`

Pattern is consistent with all existing path helpers. Names are clear and follow the established naming convention.

### `crates/sdlc-core/src/milestone.rs`

Added `UatVerdict` enum, `UatRun` struct, and three free functions.

**Correctness:**
- `save_uat_run` uses `crate::io::atomic_write` — correct, consistent with all other serialization in this crate.
- `list_uat_runs` correctly guards on `!dir.exists()` and returns empty Vec — matches the pattern of `Milestone::list`.
- `list_uat_runs` skips non-directory entries (`!entry.file_type()?.is_dir()`), defensive and correct.
- `list_uat_runs` skips missing `run.yaml` (`if manifest.exists()`) — tolerate partial/corrupt state.
- `latest_uat_run` correctly delegates to `list_uat_runs` and returns the first element.
- Sort is `b.started_at.cmp(&a.started_at)` — descending (newest first) as specified.

**Error handling:**
- All functions return `Result<_>` — no `unwrap()` anywhere in non-test code.
- Errors propagate via `?` as required.

**Serde annotations:**
- `UatVerdict` uses `#[serde(rename_all = "snake_case")]` — consistent with other enums in the crate.
- `UatRun` optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]` — consistent with `Milestone`.
- `tasks_created` uses `#[serde(default)]` to tolerate missing field during deserialization.

**Tests:**
- `uat_run_round_trip` — verifies full field round-trip through YAML serialization.
- `uat_run_list_sorted_newest_first` — verifies sort order with two runs.
- `uat_run_latest_none_when_empty` — verifies `None` when directory absent.
- All tests use `TempDir`, no shared state, no `unwrap()` in production paths.

**Quality gates:**
- `SDLC_NO_NPM=1 cargo test -p sdlc-core` → 225 tests pass, 0 failed.
- `cargo clippy -p sdlc-core -- -D warnings` → 0 warnings.

## Issues Found

None. The implementation is clean, minimal, and follows all established patterns.

## Verdict

Approved. Ready to advance to audit.
