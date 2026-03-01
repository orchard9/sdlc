# Review: orchestrator-action-model

## Summary

Adds `crates/sdlc-core/src/orchestrator/` — the complete data layer for the
tick-rate orchestrator. Three files: `action.rs` (data model), `db.rs` (redb
persistence), `mod.rs` (re-exports). 7 unit tests, all passing. Zero clippy
warnings.

## What was built

- **`Action`** struct with all specified fields including `recurrence: Option<Duration>`
- **`ActionTrigger`** enum: `Scheduled { next_tick_at }` and `Webhook { raw_payload, received_at }` with `key_ts()` method
- **`ActionStatus`** enum: `Pending | Running | Completed { result } | Failed { reason }`
- **`ActionDb`**: redb wrapper with `open`, `insert`, `set_status`, `range_due`, `startup_recovery`, `list_all`
- Composite key design: 24-byte array (timestamp_ms big-endian ++ UUID) enabling O(1) range scans for due actions
- `SdlcError::OrchestratorDb(String)` variant added to `error.rs`
- `redb = "2"` added to workspace and sdlc-core Cargo.toml

## Key correctness property verified

The `startup_recovery` test backdates `updated_at` by manipulating the redb
record directly — this is the only way to test the recovery path without
sleeping for minutes. The test confirms stale `Running` actions become
`Failed { reason: "recovered from restart" }` and recent `Running` actions
are left untouched.

## Trade-offs noted

- `set_status` does a full scan to find the action by UUID — acceptable for
  current scale (< 10k actions). At larger scale, add a UUID → key secondary
  index or a status field in the key.
- `list_all` returns all records sorted in application code — same trade-off.
- `ReadableTable` trait import required (redb API design) — documented in code.

## Tests

| Test | Verifies |
|---|---|
| `insert_and_range_due_returns_only_past_actions` | Only due actions returned |
| `range_due_excludes_non_pending` | Running actions excluded from tick dispatch |
| `composite_key_ordering_is_by_timestamp` | Keys sort by timestamp, not UUID |
| `startup_recovery_marks_old_running_as_failed` | Crash recovery works |
| `startup_recovery_leaves_recent_running_alone` | No false recoveries |
| `empty_db_range_due_returns_empty` | No panic on empty DB |
| `startup_recovery_on_empty_db_returns_zero` | No panic on empty DB |

## Build verification

```
SDLC_NO_NPM=1 cargo test -p sdlc-core -- orchestrator
# 7 passed, 0 failed

cargo clippy -p sdlc-core -- -D warnings
# 0 warnings
```
