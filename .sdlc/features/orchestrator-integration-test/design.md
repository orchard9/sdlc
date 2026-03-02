# Design: Orchestrator Integration Test

## Overview

This design covers the implementation approach for two integration tests of the orchestrator subsystem. The tests live in `crates/sdlc-cli/tests/integration.rs` and exercise the full dispatch path end-to-end.

## Test 1: Happy Path — Two Actions Complete in One Tick

### Approach

Rather than running the infinite `run_daemon` loop, we extract the single-tick body into a `pub(crate)` helper `run_one_tick(root, db)` in `orchestrate.rs`. This lets tests call it directly without threads or timeouts.

```
run_one_tick(root, db)
  └── db.range_due(Utc::now())      → Vec<Action>  [filters Pending by timestamp]
  └── for action in due:
        dispatch(root, db, action)  → sets Running → executes tool → sets Completed/Failed
```

The test calls `run_one_tick` once after the scheduled times have passed (with a short `std::thread::sleep` to let `now+100ms` and `now+200ms` elapse).

### Tool Stub

The tool stub is a minimal TypeScript file written to `.sdlc/tools/stub-tool/tool.ts` at test setup time:

```ts
console.log(JSON.stringify({ok:true}));
```

This file is valid TypeScript that any of the supported runtimes (bun, deno, node+tsx) will execute correctly. On stdout it emits `{"ok":true}`, which satisfies `ActionStatus::Completed`.

If no TypeScript runtime is detected (`detect_runtime()` returns `None`), the test is skipped via a guard:

```rust
if sdlc_core::tool_runner::detect_runtime().is_none() {
    eprintln!("skip: no JS runtime available");
    return;
}
```

### Sequence

```
test setup:
  TempDir → root
  write root/.sdlc/tools/stub-tool/tool.ts
  ActionDb::open(root/.sdlc/orchestrator.db)
  insert Action(label="a1", tool="stub-tool", at=now+100ms)
  insert Action(label="a2", tool="stub-tool", at=now+200ms)

sleep(300ms)   ← ensures both timestamps are in the past

call run_one_tick(&root, &db)

assert:
  db.list_all() contains exactly 2 actions
  both have status == ActionStatus::Completed
```

Total elapsed: ~300ms sleep + tool execution overhead, well within the 600ms window stated in the spec.

## Test 2: Startup Recovery

### Approach

This test operates entirely at the `ActionDb` level — no tool execution, no file system beyond the DB.

```
test setup:
  TempDir → root
  ActionDb::open(root/.sdlc/orchestrator.db)
  insert Action(label="stale", tool="any", at=now-1min)
  db.set_status(id, ActionStatus::Running)
  backdate updated_at to now-10min via direct DB manipulation
    (same pattern as startup_recovery_marks_old_running_as_failed in db.rs)

call db.startup_recovery(Duration::from_secs(120))

assert: return value == 1
assert: action status == ActionStatus::Failed { reason contains "recovered" }
```

The DB manipulation (backdating `updated_at`) uses the internal `action_key` + raw redb write pattern already established in the existing unit test in `db.rs`. Since the integration test is in a different crate, this requires either:

- Option A: Exposing a `pub fn backdate_for_test(db, id, dt)` helper gated on `#[cfg(test)]`.
- Option B: Duplicating the backdating pattern inline in the integration test using `sdlc_core`'s public API plus raw redb access.
- Option C: Inserting the action with `updated_at` already in the past by constructing the `Action` struct manually and bypassing `Action::new_scheduled`.

**Decision: Option C** — construct the `Action` struct directly with a backdated `updated_at` and insert it via `db.insert()`. Since `Action` derives `Serialize/Deserialize` and all fields are `pub`, this avoids leaking internal test helpers across crate boundaries.

```rust
let mut action = Action::new_scheduled("stale", "noop", json!({}), Utc::now(), None);
action.status = ActionStatus::Running;
action.updated_at = Utc::now() - chrono::Duration::minutes(10);
db.insert(&action).unwrap();
// startup_recovery checks updated_at, not created_at, so this is sufficient
```

Wait — `startup_recovery` calls `set_status` which scans `list_all` and rewrites the entry. The initial `insert` stores `status=Running` and `updated_at=now-10min`. Since we insert a Running action (not Pending then transition), the DB contains it with the stale timestamp from the start, and `startup_recovery` will find it.

## Code Changes Required

### `crates/sdlc-cli/src/cmd/orchestrate.rs`

Extract the inner tick body:

```rust
/// Execute one tick of the orchestrator: dispatch all due Pending actions.
/// Exposed for integration testing.
pub fn run_one_tick(root: &Path, db: &ActionDb) -> Result<()> {
    let now = Utc::now();
    let due = db.range_due(now).context("range_due failed")?;
    for action in due {
        dispatch(root, db, action)?;
    }
    Ok(())
}
```

`run_daemon` becomes:

```rust
pub fn run_daemon(root: &Path, db: &ActionDb, tick_rate_secs: u64) -> Result<()> {
    // ... startup_recovery ...
    loop {
        let tick_start = Instant::now();
        run_one_tick(root, db)?;
        let elapsed = tick_start.elapsed();
        if elapsed < tick_rate { std::thread::sleep(tick_rate - elapsed); }
    }
}
```

### `crates/sdlc-cli/tests/integration.rs`

Add two new test functions using `sdlc_core::orchestrator::{Action, ActionDb, ActionStatus}` and `sdlc_core::tool_runner::detect_runtime`.

### `crates/sdlc-cli/Cargo.toml`

`sdlc_core` is already a dependency of `sdlc-cli`. No new dependencies needed for the integration tests.

## File Layout After Implementation

```
crates/sdlc-cli/
  src/cmd/orchestrate.rs     ← add pub fn run_one_tick()
  tests/integration.rs       ← add 2 new test functions
```

No new files required.
