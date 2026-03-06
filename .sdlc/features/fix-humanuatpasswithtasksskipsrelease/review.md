# Review: Fix Human UAT PassWithTasks Skips Release

## Changes

### `crates/sdlc-server/src/routes/runs.rs` (line 1235)
- **Before**: `if verdict == UatVerdict::Pass {`
- **After**: `if matches!(verdict, UatVerdict::Pass | UatVerdict::PassWithTasks) {`
- Uses idiomatic `matches!` macro instead of chained `==` comparisons

### `crates/sdlc-server/tests/integration.rs`
- Added `human_uat_pass_with_tasks_releases_milestone` test
- Submits `pass_with_tasks` verdict with notes, asserts `released_at` is set
- Mirrors existing `human_uat_submit_pass` test structure

## Verification

- `cargo clippy --all -- -D warnings` — clean
- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- New test specifically validates the bug fix scenario

## Findings

No issues found. The fix is minimal, correct, and well-tested.
