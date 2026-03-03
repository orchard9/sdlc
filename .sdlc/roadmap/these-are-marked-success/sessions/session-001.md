---
session: 1
timestamp: 2026-03-03T06:00:00Z
orientation:
  current: "Two UAT runs from last night display misleadingly in the UI — one stuck as 'running' forever, one showing a green checkmark despite reporting a hard blocker. The run lifecycle conflates process completion with outcome quality."
  next: "Decide minimum fix: (1) outcome field + agent-callable endpoint, (2) startup recovery for stuck runs. Write a fix-forward task."
  commit: "When we agree on the exact model change (what new fields/endpoints are needed) and the scope is small enough to ship in one cycle."
---

**jordan · Owner**
These are marked success

UAT: ponder-ux-polish
10:06 PM



UAT: v20-feedback-threads
10:06 PM
· $0.54
· 15 turns

[run output showing HARD BLOCKER: localhost:7777 is unreachable]

---

## Session 1 — "Completed" is a lie

**Facilitator** — Let's orient on what we actually observed.

Last night two UAT runs fired for `ponder-ux-polish` and `v20-feedback-threads`. Both are displaying in the dashboard's run feed without any red indicator. Jordan's observation is that they look like successes. Let me pull the facts:

**Run 1 — ponder-ux-polish** (`20260303-050612-hhh`)
```json
{ "status": "running", "completed_at": null, "error": null }
```
The process appears to have died without emitting a completion signal. The run is frozen as `"running"` forever — spinning loader in the UI.

**Run 2 — v20-feedback-threads** (`20260303-050616-xhh`)
```json
{ "status": "completed", "completed_at": "...", "cost_usd": 0.54, "turns": 15, "error": null }
```
The agent exited cleanly (exit code 0), so `spawn_agent_run` recorded it as `"completed"`. But the agent's final text was an explicit hard blocker:
> "HARD BLOCKER: localhost:7777 is unreachable — the sdlc server is not running"

The `RunCard` renders `"completed"` as `<CheckCircle2 className="text-green-400" />`. Green checkmark. Success.

Neither UAT was actually run. Both look fine.

---

**Priya Nair** — This is the classic "failure mode that looks like success" problem. I've seen it destroy observability at scale. The issue is that `status` conflates two orthogonal things:

1. **Process fate** — did the agent SDK call return without throwing? (`running` → `completed` | `failed`)
2. **Outcome quality** — did the operation actually accomplish what it was supposed to? (`passed` | `failed` | `blocked`)

Right now `status: "completed"` means "the agent process exited clean." It says nothing about whether the UAT passed. A UAT agent that runs for 15 turns, hits a blocker, apologizes, and exits with code 0 is "completed" in the same way a UAT that fully verified all 12 acceptance criteria and called `sdlc milestone complete` is "completed."

?  **Open:** Should we model this as a new `outcome` field on `RunRecord`, or should the agent signal a non-zero exit on blockers/failures?

---

**Tobias Krenn** — Let me push back on the scope before we start designing a rich outcome model.

What's the actual user problem? Jordan looked at the run feed and didn't know the UAT had failed. The fix for that is: **the run should show red when the UAT hit a blocker.** That's it.

The simplest path: the UAT skill template instructs the agent to call a `/api/milestone/:slug/uat/fail` endpoint before returning when it hits any hard blocker. The server marks the run `failed`. `RunCard` shows `<XCircle className="text-red-400" />`. Done.

We don't need an `outcome` field with four enum variants. We need the agent to correctly differentiate "I completed successfully" from "I stopped because of an unresolvable blocker." The latter should be `failed`, not `completed`.

⚑ **Decided:** The fix is behavioral (skill instruction) + minimal server support (fail endpoint or status-on-completion hook), not a new data model.

---

**Priya Nair** — I partially agree, but the fail-endpoint approach has a gap: what about the stuck run?

`ponder-ux-polish` is `"running"` and will stay that way indefinitely. The process died — maybe the server restarted, maybe the agent was killed. `spawn_agent_run` only writes `completed`/`failed` on the Tokio task completion path. If the task was dropped (server shutdown), the run file never gets updated.

This is the "stuck service" failure mode. It's not a UAT-specific problem — it affects any `spawn_agent_run` call when the server restarts mid-run.

Two separate fixes:
1. **Startup recovery**: On server start, scan all `status: "running"` runs. Any run where `started_at < server_started_at` → mark `failed` with `error: "Run interrupted (server restart)"`. This is a one-liner at startup.
2. **Outcome signaling**: The UAT skill template must call an explicit endpoint on both success (already does via `sdlc milestone complete`) and failure (missing). On blocker → call a fail endpoint so the run status is `failed`, not `completed`.

⚑ **Decided:** Two distinct bugs, two distinct fixes. Don't conflate them.

---

**Ben Hartley** — From the UX angle: the green checkmark is a trust violation. Jordan saw green, assumed pass, moved on. The information hierarchy failed.

There's a zoom-level problem too. The run feed shows `label + timestamp + cost + turns`. No quick-glance signal for "this run accomplished its goal" vs. "this run ran but failed quietly." The `error` field is surfaced in the meta line — but only if `run.error` is non-null. A "hard blocker" that the agent described in text but didn't encode in `error` is invisible.

One targeted improvement: when the agent calls the fail endpoint with a reason string, the server stores it in `error`. Then `RunCard` already has:

```tsx
{run.error && <span className="text-red-400 truncate">· {run.error.slice(0, 40)}</span>}
```

So the meta line reads: `10:06 PM · $0.54 · 15 turns · HARD BLOCKER: localhost:7777 is...`

No UI change needed. The data just needs to flow through.

⚑ **Decided:** `error` field in `RunRecord` should be populated by fail endpoint with reason string. Existing RunCard UI handles it automatically.

---

**Facilitator** — Synthesizing the minimum fix:

### What broke

| Run | `status` | Truth | User sees |
|-----|---------|-------|-----------|
| ponder-ux-polish | `running` | Process died | Spinning loader, forever |
| v20-feedback-threads | `completed`, `error: null` | HARD BLOCKER | Green ✓ |

### Root causes

1. **No startup recovery for interrupted runs** — `spawn_agent_run` only transitions to `failed` on task error. If the server restarts, in-flight runs are orphaned in `"running"` state.
2. **Agent has no explicit failure path** — The UAT skill template tells the agent to call `sdlc milestone complete` on success, but gives no instruction for what to call on blockers. The agent exits cleanly, `spawn_agent_run` records `completed`.

### Minimum fix (two tasks)

**T1 — Startup recovery** (Rust, server startup in `state.rs` or `lib.rs`)
- Record `server_started_at` on startup
- Scan `.sdlc/.runs/*.json` where `status == "running"` AND `started_at < server_started_at`
- Write: `{ status: "failed", error: "Run interrupted (server restart)", completed_at: server_started_at }`
- Emit SSE: `RunFailed { key }` for each recovered run
- Applies to ALL run types — not UAT-specific

**T2 — UAT agent failure signaling** (Skill template in `init.rs` + Rust endpoint)
- Add `POST /api/milestone/:slug/uat/fail` body `{ "reason": "..." }` → finds current running UAT run for `:slug`, sets `status: "failed"`, `error: reason`, `completed_at: now`
- Update UAT skill template:
  > If localhost:7777 is unreachable or a hard blocker is encountered: call `POST /api/milestone/:slug/uat/fail` with the reason, then stop. This marks the run failed in the dashboard.

### What we explicitly are NOT doing

- New `outcome` field on `RunRecord` — not needed
- Frontend changes — RunCard's existing error display already handles it
- `outcome: passed | failed | blocked` enum — over-engineered

⚑ **Decided:** Two-task fix. Startup recovery covers stuck runs generically. Fail endpoint + skill template update covers false-success UATs. No new data model needed.

---

**Tobias Krenn** — The recovery trigger should be `started_at < server_started_at`, not a fixed timeout. Priya is right on that. A long prepare run is legitimately long. But a run that started before this server process launched is definitively orphaned.

⚑ **Decided:** Recovery trigger is `started_at < server_started_at`.
