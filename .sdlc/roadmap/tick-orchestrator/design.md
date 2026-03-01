# Tick Orchestrator — Design

## The Mental Model

The orchestrator is a game loop. It has a tick rate. Each tick is atomic — it
runs once, processes everything due, then waits exactly `tick_rate - elapsed`
before the next tick. Only one tick ever runs at a time.

Actions have two trigger modes:

| Mode | Description |
|---|---|
| `Scheduled` | Has a `next_tick_at` timestamp. Fires when `now >= next_tick_at`. |
| `Webhook` | Raw payload stored on ingress. Processed on the next tick. |

That's the whole model. The tick loop doesn't care what the action does — it
finds due actions, marks them running, dispatches them, marks them done.

---

## Data Model

```rust
pub enum ActionTrigger {
    Scheduled { next_tick_at: DateTime<Utc> },
    Webhook { raw_payload: Vec<u8>, received_at: DateTime<Utc> },
}

pub enum ActionStatus {
    Pending,
    Running,
    Completed,
    Failed { reason: String },
}

pub struct Action {
    pub id: Uuid,
    pub slug: String,      // which service/feature this action is for
    pub trigger: ActionTrigger,
    pub status: ActionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## DB: redb

**Choice: redb** — pure Rust, ACID, range-scannable, actively maintained.

No SQLite (C FFI, cross-compile friction). No sled (maintenance concerns).
redb gives typed tables with ordered keys, which is exactly what the
timestamp-keyed action table needs.

Two tables:

### Table 1: `actions`

```
key:   [u8; 24]  =  timestamp_ms (u64 big-endian) ++ uuid (16 bytes)
value: Vec<u8>   =  JSON-serialized Action
```

The composite key is the critical insight: since timestamp bytes come first
in big-endian, byte-order sorts by timestamp. A range scan `..=now_key`
returns all due actions in chronological order with a single DB operation.

```rust
fn now_key(id: Uuid) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ts = Utc::now().timestamp_millis() as u64;
    key[..8].copy_from_slice(&ts.to_be_bytes());
    key[8..].copy_from_slice(id.as_bytes());
    key
}

fn due_upper_bound() -> [u8; 24] {
    let mut key = [0u8; 24];
    let ts = Utc::now().timestamp_millis() as u64;
    key[..8].copy_from_slice(&ts.to_be_bytes());
    key[8..].fill(0xff);  // max uuid suffix
    key
}

// Get all actions due now:
// table.range(..=due_upper_bound())
```

### Table 2: `webhooks` (Phase 2)

```
key:   [u8; 16]  =  uuid bytes
value: Vec<u8>   =  raw HTTP body (no transformation on ingress)
```

Webhooks are stored raw. On tick, they're read, processed, then deleted.
Never transform on ingress — the action handler owns interpretation.

---

## The Tick Loop

```
loop {
    tick_start = Instant::now()

    // 1. Find all scheduled actions due now
    due_actions = db.range(..=due_upper_bound())

    // 2. Process each
    for action in due_actions:
        db.set_status(action.id, Running)   // write BEFORE executing
        result = execute(&action)            // dispatch — never block the loop
        db.set_status(action.id, result)

    // 3. (Phase 2) Process pending webhooks
    // for webhook in db.all_webhooks(): ...

    // 4. Sleep the remainder of the tick
    elapsed = tick_start.elapsed()
    if elapsed < tick_rate:
        sleep(tick_rate - elapsed)
    // if elapsed >= tick_rate: fire immediately (no backlog accumulation)
}
```

**Correctness property — no double-fire:** Mark `Running` in the DB inside
a write transaction BEFORE executing. On restart, `Running` actions are
recovered: either retried (idempotent actions) or surfaced as `Failed` for
inspection. This is the only state machine property that matters for v1.

---

## Where It Lives

**Phase 1: `sdlc orchestrate` CLI command**

```bash
sdlc orchestrate --tick-rate 60          # tick every 60 seconds (default)
sdlc orchestrate --tick-rate 30 --db .sdlc/orchestrator.db
```

Runs as a foreground daemon. Add to supervisor/systemd in production.

Location: `crates/sdlc-cli/src/cmd/orchestrate.rs`
DB location: `crates/sdlc-core/src/orchestrator/` (db.rs, action.rs)

This follows the existing pattern: core holds data structures + I/O,
CLI holds the command loop.

**Phase 2:** Webhook receiver as an HTTP endpoint in `sdlc-server`.
**Phase 3:** Management API (add/list/cancel actions) + web UI panel.

---

## Phase 1 Scope

Build **only** this:

1. `Action` struct + `ActionDb` wrapper (redb) in `sdlc-core`
2. `sdlc orchestrate` command in `sdlc-cli`
3. The tick loop: reads due scheduled actions, dispatches `sdlc next --for <slug>`, sleeps
4. One integration test: two actions scheduled 100ms apart, tick rate 500ms, both fire

**Not in Phase 1:**
- Webhooks (Phase 2)
- Management API (Phase 3)
- Web UI panel (Phase 3)
- Distributed orchestration (future)

---

## Team Voices

**Priya Nair · Distributed Systems**
The composite key design (timestamp_ms || uuid as 24 bytes) is the right
call for redb range scans. One addition: the `Running` → `Failed` recovery
on startup matters more than it seems. When Jordan's orchestrating 1,000
services, restarts happen. Add a startup sweep: any `Running` action older
than 2× tick_rate gets marked `Failed { reason: "recovered from restart" }`.
That gives him a signal something went wrong without silent loss.

**Marcus Webb · Enterprise Platform**
The single-file DB (`.sdlc/orchestrator.db`) is clean. It should live
adjacent to `.sdlc/state.yaml` so git ignores it cleanly (add to .gitignore)
while the rest of `.sdlc/` stays tracked. For enterprise, the audit trail
is git — the DB is operational state only, disposable on restart.

**Dana Cho · Product Skeptic**
Phase 1 scope is right. Don't touch webhooks until the tick loop is proven.
The integration test is the gate: if you can write a test that schedules
two actions and verifies both fired, the model is sound. Everything else
is expansion on a working foundation.
