# Dev Run

**Dev run** is the autonomous heartbeat of the system. Each invocation is a single, incremental
advance of the best available next thing — no human in the loop, no coordination overhead,
no "what should I work on?" Every run exits with a clear state, and the next run picks up
exactly where the last left off.

---

## Core Ethos

> **Fail = re-enter.** When something breaks, the response is not to stop — it is to push the
> failure through the right sdlc command and let the machine decide what to do next.

This is the inversion of the traditional CI model (fail → alert → human). Dev run treats
failures as state transitions. A broken quality gate is not a crash; it is an input to
`/sdlc-quality-fix`. A failed UAT is not a blocker; it is an input to the failure-pathway
workflow. The state machine always has a next action.

---

## Current Flow (v2 — parallel dispatch)

```
dev-driver --run
  │
  ├─ Quality gate (synchronous, blocks dispatch)
  │    pass → continue
  │    fail / error → { action: quality_failing }
  │                   → agent runs /sdlc-quality-fix
  │                   → next dev-driver run re-checks
  │
  ├─ GET /api/state → parallel_work[]  (Rust: select_parallel_work)
  │    empty → { action: idle }
  │
  └─ Dispatch all slots concurrently
       feature slot → /sdlc-run <slug>     (max 4 total)
       UAT slot     → /sdlc-milestone-uat  (max 1)
       409 Conflict → already running, skip
```

**Selection:** `select_parallel_work()` in `crates/sdlc-core/src/parallel_work.rs` is the
single source of truth — same logic used by the dashboard's "Current" zone.

---

## Re-entry Contract

Every failure pathway terminates in an sdlc command that re-enters the state machine.
The dev-driver never retries raw — it always routes through a command that produces
new state, then the next run reads that state.

| Failure | Re-entry command | What it produces |
|---------|-----------------|-----------------|
| Quality gate fails | `/sdlc-quality-fix` | Fixed code or task tracking failure |
| UAT fails | _(see: UAT failure pathways)_ | Tasks, re-run, or escalation |
| Feature run stalls | `/sdlc-next <slug>` | Single-step recovery |
| Agent errors | Run record in `failed` state | Next dev-driver run re-dispatches |

---

## Planned Layers

The dev-run model will grow to handle more situations. Each layer adds a new re-entry
path without changing the core dispatch loop.

### Layer 1: Quality re-entry ✅ (implemented)
Quality gate failure routes to `/sdlc-quality-fix`. Dev-driver blocks dispatch until clean.

### Layer 2: UAT failure pathways _(in progress)_
When `milestone-uat` returns a failed verdict, the next dev-driver run must choose:

- **Minor failures** (≤ N tests, all have fix tasks) → dispatch fix tasks, re-queue UAT
- **Major failures** → escalate: create escalation record, surface to human via dashboard
- **Flaky / infra** → park UAT slot temporarily, advance other milestones

The UAT failure pathway is a separate workflow invoked by dev-driver when it detects
a milestone stuck in `Verifying` with failed UAT runs.

### Layer 3: Escalation surface _(planned)_
Some failures cannot be auto-resolved:

- Secret requests (credentials needed)
- Vision questions (ambiguous requirements)
- Manual test gates (hardware-in-the-loop)

These produce an `Escalation` record (`crates/sdlc-core/src/escalation.rs`) and appear
in the dashboard for human response. Dev-driver skips escalated slots — they are not
dispatched until the escalation is resolved.

```
dev-driver detects stuck slot (failed run + open escalation)
  → skip slot
  → include in output: { status: "escalated", escalation_id: "..." }
```

### Layer 4: Milestone-level re-entry _(planned)_
When an entire milestone is stuck (all features blocked or escalated), dev-driver will
invoke a milestone-level recovery command:

```
/sdlc-milestone-recover <slug>
```

This command audits the milestone, closes stale escalations it can, creates tasks for
the rest, and attempts to unblock at least one feature so the next dev-driver run has
something to dispatch.

### Layer 5: Adaptive turn budgets _(planned)_
Run failures with `stop_reason: max_turns` trigger a re-dispatch with a higher turn
budget. Dev-driver will read the run record's `stop_reason` field and bump `maxTurns`
on retry (e.g. 40 → 80 → 120, capped).

---

## Invariants

1. **Dev-driver never blocks on human input.** Every non-escalation failure has an
   autonomous re-entry path. Escalations are the explicit, intentional exception.

2. **One command per failure type.** Re-entry routes are stable — the same failure
   always triggers the same command. Agents and orchestrators can predict behavior.

3. **State advances, never rewinds.** Re-entry commands produce new artifacts or tasks —
   they never reset phase or revert completed work.

4. **409 is not a failure.** A conflict means the slot is already being worked. Dev-driver
   reports it as `status: conflict` and moves on. It is normal and expected.

5. **The orchestrator is the scheduler, not the logic.** Dev-driver is stateless. The
   cadence, retry interval, and concurrency limits live in orchestrator actions —
   not in the tool itself.

---

## Key Files

| File | Role |
|------|------|
| `.sdlc/tools/dev-driver/tool.ts` | The tool — quality gate + parallel dispatch |
| `crates/sdlc-core/src/parallel_work.rs` | Slot selection logic (Rust) |
| `crates/sdlc-server/src/routes/runs.rs` | `spawn_agent_run` — all dispatches go here |
| `crates/sdlc-core/src/escalation.rs` | Escalation records (future re-entry gate) |
| `~/.claude/commands/sdlc-quality-fix.md` | Quality re-entry command |
| `~/.claude/commands/sdlc-organize-parallel.md` | Manual parallel dispatch (same slots, agent-driven) |

---

## Relationship to `/sdlc-organize-parallel`

`/sdlc-organize-parallel` is the **agent-native** version of dev-driver. It reads the
same `parallel_work` queue via `sdlc parallel-work --json` and dispatches the same slots
via the Agent tool. The difference:

| | dev-driver | sdlc-organize-parallel |
|---|---|---|
| Runtime | Tool (TypeScript, cron) | Agent (slash command, human-invoked) |
| Dispatch | `POST /api/tools/agent-dispatch` | Agent tool calls |
| Quality gate | `quality-check` tool | Not included (agent decides) |
| Re-entry | Automatic (next cron tick) | Manual (`**Next:** /sdlc-organize-parallel`) |

They are complementary — dev-driver runs autonomously on cadence, organize-parallel is
for human-initiated burst runs or when the orchestrator is not set up.
