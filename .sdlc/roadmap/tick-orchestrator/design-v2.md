# Tick Orchestrator — Design v2

## What Changed from v1

Session 1 left "what does an action execute?" open. Session 2 closes it:
**actions execute tools**. The timing (orchestrator's job) and the execution
(tool system's job) are two separate, composable concerns.

---

## The Unified Model

```
Action = trigger + tool
```

| Field | Type | Description |
|---|---|---|
| `id` | `Uuid` | Unique action identifier |
| `trigger` | `ActionTrigger` | When to fire (Scheduled or Webhook) |
| `tool_name` | `String` | Which tool to run (e.g., `"quality-check"`) |
| `tool_input` | `serde_json::Value` | JSON passed to the tool via stdin |
| `status` | `ActionStatus` | Pending → Running → Completed / Failed |
| `recurrence` | `Option<Duration>` | If set, reschedule automatically after completion |
| `created_at` | `DateTime<Utc>` | — |
| `updated_at` | `DateTime<Utc>` | — |

```rust
pub enum ActionTrigger {
    Scheduled { next_tick_at: DateTime<Utc> },
    Webhook {
        raw_payload: Vec<u8>,
        received_at: DateTime<Utc>,
    },
}

pub enum ActionStatus {
    Pending,
    Running,
    Completed { result: serde_json::Value },
    Failed { reason: String },
}
```

---

## The Tick Loop

```
loop {
    tick_start = Instant::now()

    due_actions = db.range(..=due_upper_bound())

    for action in due_actions:
        db.set_status(action.id, Running)          // write before executing

        result = run_tool(
            tool_script(root, &action.tool_name),
            "--run",
            Some(&action.tool_input.to_string()),
            root,
            None,
        )

        match result {
            Ok(stdout) => {
                let tool_result = parse_tool_result(&stdout)
                db.set_status(action.id, Completed { result: tool_result })
                if let Some(interval) = action.recurrence {
                    db.insert(action.rescheduled(interval))  // next run
                }
            }
            Err(e) => db.set_status(action.id, Failed { reason: e })
        }

    elapsed = tick_start.elapsed()
    if elapsed < tick_rate:
        sleep(tick_rate - elapsed)
}
```

`run_tool()` is **the same function** the UI's tool runner calls. Zero new
execution infrastructure.

---

## Why This Unification Is Right

### Tools get better

Tools already have `input_schema` (via `--meta`). For tools to work as
orchestrator payloads they need:
- Clean, stable input schemas (orchestrated calls have no human to fix a bad input)
- Meaningful `ok` / `error` in the result (not just side effects)
- Idempotent behavior (the orchestrator may retry)

This pressure improves every tool — the ones Jordan builds over the next 3 months
will be designed with scheduled invocation in mind from the start.

### The orchestrator gets simpler

No new execution concept. The DB knows *when* and *what*. `run_tool()` knows
*how*. That's the whole system.

### The sdlc use case is a tool

`sdlc next --for <slug>` + agent dispatch = a tool called `sdlc-next`.
Scaffold it once:
```bash
sdlc tool scaffold sdlc-next "Run sdlc next directive and dispatch the agent"
```

Then any service can be scheduled as an action:
```bash
sdlc orchestrate add my-service \
  --tool sdlc-next \
  --input '{"slug": "my-service"}' \
  --every 60s
```

The orchestrator doesn't know what `sdlc-next` does. It just runs it.

---

## Webhook → Tool Mapping (Phase 2)

When a webhook fires, which tool runs? Register webhook routes:

```rust
pub struct WebhookRoute {
    pub id: Uuid,
    pub path: String,           // e.g. "/webhooks/deploy"
    pub tool_name: String,
    pub input_template: String, // "{{payload}}" maps raw body to tool input
}
```

On ingress: store raw payload. On tick: match raw payload against registered
routes → render template → call `run_tool()`.

Phase 1: no webhook routes. Phase 2: add `WebhookRoute` table to redb.

---

## Recurrence

`recurrence: Option<Duration>` on `Action`. After `Completed`, if set, insert
a new `Pending` action with `next_tick_at = now + recurrence`.

For the "run every tick" use case: `recurrence = Some(tick_rate)`.

⚑ Decision pending: include `recurrence` field in Phase 1 struct (cheap) but
leave re-scheduling logic for Phase 2? Costs almost nothing to add upfront.
Recommendation: yes — add the field, skip the re-scheduling logic in v1.

---

## Phase 1 Scope (Revised)

1. `crates/sdlc-core/src/orchestrator/action.rs` — `Action`, `ActionTrigger`, `ActionStatus`
2. `crates/sdlc-core/src/orchestrator/db.rs` — `ActionDb` wrapping redb
   - Composite key: `timestamp_ms (u64 be) ++ uuid (16 bytes)` = 24 bytes
   - `insert(action)`, `set_status(id, status)`, `range_due(now)`, `startup_recovery()`
3. `crates/sdlc-cli/src/cmd/orchestrate.rs`
   - `sdlc orchestrate` — start tick loop
   - `sdlc orchestrate add <slug> --tool <name> --input <json> [--at <timestamp>] [--every <duration>]`
4. Integration test: schedule two `quality-check` actions 100ms apart, tick rate 500ms, verify both fire and return `ok: true`

**Not in Phase 1:** webhooks, webhook routes, web UI, management HTTP API.

---

## Team Voices

**Priya Nair · Distributed Systems**
The `recurrence` field decision is a trap. Add it to the struct now — the cost
is one `Option<Duration>` field and a serde impl. The cost of not having it is
a DB migration when Phase 2 arrives. Struct fields are cheap. Migrations are not.

The `Completed { result: serde_json::Value }` on `ActionStatus` is the right
call for observability. When 1,000 actions run per tick, Jordan needs to be able
to inspect what each one returned. Store the result, don't just mark done.

**Marcus Webb · Enterprise Platform**
The tool-as-action-executor is the correct enterprise model. Tools already have
a defined interface, versioning (the `version` field in `ToolMeta`), and a
clear contract. When a Fortune 500 audit asks "what ran on service X at time T
and what did it produce?", the answer is in redb: `action.tool_name`,
`action.tool_input`, `action.status.result`. Clean audit trail by construction.

**Dana Cho · Product Skeptic**
The `sdlc-next` tool wrapper is Phase 1. Don't scaffold it until the tick loop
is proven. The integration test should use `quality-check` — it already exists,
has no side effects, and proves the loop works. `sdlc-next` is the real payload
but it brings in agent complexity. Validate the orchestrator without it first.
