# QA Plan: orchestrator-tick-cli

## Automated Tests

### Unit tests (existing — in db.rs)
All 6 `ActionDb` unit tests must pass:
- `insert_and_range_due_returns_only_past_actions`
- `range_due_excludes_non_pending`
- `composite_key_ordering_is_by_timestamp`
- `startup_recovery_marks_old_running_as_failed`
- `startup_recovery_leaves_recent_running_alone`
- `empty_db_range_due_returns_empty`
- `startup_recovery_on_empty_db_returns_zero`

### Build + lint
```bash
SDLC_NO_NPM=1 cargo build -p sdlc-cli
cargo clippy -p sdlc-cli -- -D warnings
SDLC_NO_NPM=1 cargo test --all
```
All must pass with zero warnings.

## Manual Smoke Tests

### 1. Help output
```bash
./target/debug/sdlc orchestrate --help
./target/debug/sdlc orchestrate add --help
./target/debug/sdlc orchestrate list --help
```
Expected: each prints usage with correct flags, no panics.

### 2. Add and list
```bash
./target/debug/sdlc orchestrate add test-action --tool quality-check --input '{}'
./target/debug/sdlc orchestrate list
```
Expected: table row with `test-action`, `quality-check`, status `Pending`.

### 3. --at parsing
```bash
./target/debug/sdlc orchestrate add now-action --tool quality-check --input '{}' --at now
./target/debug/sdlc orchestrate add future-action --tool quality-check --input '{}' --at now+10s
./target/debug/sdlc orchestrate add hourly --tool quality-check --input '{}' --at now+1h --every 3600
./target/debug/sdlc orchestrate list
```
Expected: three rows with appropriate labels and statuses.

### 4. Daemon fires a due action
```bash
./target/debug/sdlc orchestrate add fire-test --tool quality-check --input '{}' --at now
./target/debug/sdlc orchestrate --tick-rate 5
# Wait one tick (~5s), then CTRL-C
./target/debug/sdlc orchestrate list
```
Expected: `fire-test` status is `Completed` (or `Failed` if quality-check has issues in the test env — not `Pending` or `Running`).

### 5. Restart recovery
```bash
# Start daemon, kill it mid-run (or force a Running action via direct DB manipulation)
./target/debug/sdlc orchestrate --tick-rate 60
# CTRL-C immediately after it starts
./target/debug/sdlc orchestrate list
```
Expected: no actions stuck in `Running` after daemon exits and restarts (startup_recovery clears stale Running).

## Acceptance Criteria

- `sdlc orchestrate --help` exits 0 and prints usage
- `sdlc orchestrate add` inserts a `Pending` action visible in `list`
- `sdlc orchestrate --tick-rate 5` fires a due action within one tick window
- Recurring actions (--every) re-queue after completion
- Daemon exits cleanly on CTRL-C
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy -p sdlc-cli -- -D warnings` passes
