# Tasks: orchestrator-action-model

## T1: Add redb dependency to workspace and sdlc-core

Add `redb = "2"` to `[workspace.dependencies]` in root `Cargo.toml`.
Add `redb = { workspace = true }` and `uuid = { workspace = true }` to
`crates/sdlc-core/Cargo.toml` `[dependencies]`.

Verify: `SDLC_NO_NPM=1 cargo check -p sdlc-core` succeeds.

## T2: Add OrchestratorDb error variant to SdlcError

In `crates/sdlc-core/src/error.rs`, add:
```rust
#[error("orchestrator DB error: {0}")]
OrchestratorDb(String),
```

## T3: Create orchestrator/action.rs — Action, ActionTrigger, ActionStatus

Create `crates/sdlc-core/src/orchestrator/action.rs` with:
- `ActionTrigger` enum (Scheduled, Webhook) with `key_ts()` method
- `ActionStatus` enum (Pending, Running, Completed { result }, Failed { reason })
- `Action` struct with all fields including serde-as-seconds Duration helper
- `Action::new_scheduled()` constructor
- All types: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- `ActionTrigger` and `ActionStatus`: tagged enums with `#[serde(tag = "type", rename_all = "snake_case")]`

## T4: Create orchestrator/db.rs — ActionDb

Create `crates/sdlc-core/src/orchestrator/db.rs` with:
- `ACTIONS` table definition using `TableDefinition<&[u8], &[u8]>`
- `action_key(ts, id) -> [u8; 24]` and `due_upper_bound(now) -> [u8; 24]` fns
- `ActionDb` struct wrapping `redb::Database`
- `open(path)`, `insert(action)`, `set_status(id, status)`, `range_due(now)`, `startup_recovery(max_age)`, `list_all()`
- All errors mapped to `SdlcError::OrchestratorDb`

## T5: Create orchestrator/mod.rs and register module in lib.rs

Create `crates/sdlc-core/src/orchestrator/mod.rs`:
```rust
pub mod action;
pub mod db;
pub use action::{Action, ActionStatus, ActionTrigger};
pub use db::ActionDb;
```

Add to `crates/sdlc-core/src/lib.rs`:
```rust
pub mod orchestrator;
```

## T6: Write unit tests in db.rs

Add `#[cfg(test)]` module to `db.rs` with tests using `tempfile::TempDir`:
1. `insert_and_range_due` — two actions with timestamps 100ms apart, verify range_due(mid) returns only earlier one
2. `range_due_excludes_non_pending` — Running action in range, verify excluded
3. `composite_key_ordering` — insert out of chronological order, verify range_due returns in order
4. `startup_recovery_marks_old_running` — Running action 10min old, recovery(2min) → Failed
5. `startup_recovery_leaves_recent_running` — Running action 30s old, recovery(2min) → still Running

## T7: Verify build and tests pass

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-core 2>&1 | tail -20
cargo clippy -p sdlc-core -- -D warnings 2>&1 | tail -20
```

All tests pass, no clippy warnings.
