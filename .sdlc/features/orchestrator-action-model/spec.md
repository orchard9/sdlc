# Spec: orchestrator-action-model

## Overview

Add a new `orchestrator` module to `sdlc-core` that defines the `Action` data
model and `ActionDb` persistence layer for the tick-rate orchestrator. This is
the data foundation that `sdlc orchestrate` (the CLI daemon) sits on top of.

An **action** is the atomic unit of orchestration: a trigger condition (when to
fire) paired with a tool (what to run). The orchestrator tick loop queries
`ActionDb` for due actions, marks them running, dispatches `run_tool()`, and
updates their status — all without knowing anything about what the tool does.

## Module Structure

```
crates/sdlc-core/src/orchestrator/
├── mod.rs      — pub re-exports
├── action.rs   — Action, ActionTrigger, ActionStatus structs and enums
└── db.rs       — ActionDb wrapping redb
```

Expose as `pub mod orchestrator;` in `lib.rs`.

## Data Model

### ActionTrigger

```rust
pub enum ActionTrigger {
    Scheduled { next_tick_at: DateTime<Utc> },
    Webhook {
        raw_payload: Vec<u8>,
        received_at: DateTime<Utc>,
    },
}
```

`Scheduled` actions have a timestamp at which they become due. `Webhook` actions
carry the raw HTTP payload stored on ingress (no transformation). Both trigger
types are stored in the DB and processed by the same tick loop.

### ActionStatus

```rust
pub enum ActionStatus {
    Pending,
    Running,
    Completed { result: serde_json::Value },
    Failed { reason: String },
}
```

Status transitions: `Pending → Running → Completed | Failed`. The tick loop
writes `Running` *before* dispatching the tool — this is the no-double-fire
guarantee on restart. `Completed` stores the full tool result JSON for
observability. `Failed` stores the error reason.

### Action

```rust
pub struct Action {
    pub id: Uuid,
    pub label: String,             // human-readable identifier (e.g. "my-svc")
    pub tool_name: String,         // tool slug (e.g. "quality-check")
    pub tool_input: serde_json::Value,  // JSON passed to tool via stdin
    pub trigger: ActionTrigger,
    pub status: ActionStatus,
    pub recurrence: Option<std::time::Duration>,  // if set, re-schedule after Completed
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## DB Design: redb

**Choice:** redb — pure Rust, ACID, range-scannable, actively maintained. No
SQLite (C FFI), no sled (maintenance concerns).

### Table: ACTIONS

```
key:   [u8; 24]   — timestamp_ms (u64 big-endian) ++ uuid (16 bytes)
value: Vec<u8>    — serde_json serialized Action
```

The composite key is the critical design: because timestamp bytes come first in
big-endian encoding, byte ordering equals timestamp ordering. A range scan
`..=due_upper_bound()` returns all due actions in one DB operation with no
filtering required.

```rust
fn action_key(ts: DateTime<Utc>, id: Uuid) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = ts.timestamp_millis() as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].copy_from_slice(id.as_bytes());
    key
}

fn due_upper_bound(now: DateTime<Utc>) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = now.timestamp_millis() as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].fill(0xff);  // max uuid suffix — includes all ids at this ms
    key
}
```

Webhook-triggered actions use `received_at` as the timestamp key, so they
appear in the range scan on the first tick after they arrive.

## ActionDb Interface

```rust
pub struct ActionDb { /* redb::Database */ }

impl ActionDb {
    /// Open or create the DB file at the given path.
    pub fn open(path: &Path) -> Result<Self>;

    /// Insert a new Pending action. The key is derived from the trigger timestamp
    /// (Scheduled: next_tick_at, Webhook: received_at).
    pub fn insert(&self, action: &Action) -> Result<()>;

    /// Atomically update the status of an action by id.
    /// Scans all records — acceptable for current scale (< 10k actions).
    pub fn set_status(&self, id: Uuid, status: ActionStatus) -> Result<()>;

    /// Return all actions where key <= due_upper_bound(now).
    /// Only returns Pending actions — Running/Completed/Failed are excluded.
    pub fn range_due(&self, now: DateTime<Utc>) -> Result<Vec<Action>>;

    /// On daemon startup: any action with status=Running and updated_at older
    /// than max_age is marked Failed { reason: "recovered from restart" }.
    pub fn startup_recovery(&self, max_age: std::time::Duration) -> Result<u32>;

    /// List all actions (for sdlc orchestrate list). Sorted by created_at desc.
    pub fn list_all(&self) -> Result<Vec<Action>>;
}
```

## Error Handling

Extend `SdlcError` with orchestrator variants or use `anyhow` for DB errors.
All `ActionDb` methods return `Result<T>` using the existing `crate::Result`
alias. No `unwrap()` anywhere.

## Cargo Dependencies

Add to `Cargo.toml` workspace dependencies:
```toml
redb = "2"
```

Add to `crates/sdlc-core/Cargo.toml`:
```toml
redb = { workspace = true }
uuid = { workspace = true }
```

(`uuid` is already a workspace dep with `v4` feature.)

## Testing

Unit tests in `db.rs` (or `orchestrator/tests.rs`):

1. **`insert_and_range_due`** — insert two actions with timestamps 100ms apart, call `range_due(now)` where now is between them, verify only the earlier one is returned.
2. **`range_due_excludes_non_pending`** — insert a Running action that is due; verify `range_due` excludes it.
3. **`composite_key_ordering`** — insert actions out of chronological order, verify `range_due` returns them in timestamp order.
4. **`startup_recovery_marks_old_running_as_failed`** — insert a Running action with `updated_at = now - 10min`, call `startup_recovery(2min)`, verify status becomes `Failed` with reason containing "recovered".
5. **`startup_recovery_leaves_recent_running_alone`** — insert a Running action with `updated_at = now - 30s`, call `startup_recovery(2min)`, verify it stays `Running`.

All tests use `tempfile::TempDir` for the DB path.

## Out of Scope

- Webhook-specific storage tables (Phase 2)
- `list_all` filtering by status (Phase 3)
- Any execution logic — this module is data only
