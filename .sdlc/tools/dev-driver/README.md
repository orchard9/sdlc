# dev-driver

A stock sdlc tool that reads project state, finds the single most important next
development action, dispatches it asynchronously, and exits. Paired with a recurring
orchestrator action (every 4 hours), it makes your sdlc project self-advancing.

---

## What it does

On each invocation, dev-driver applies a 5-level priority waterfall and takes exactly one action:

1. **Flight lock** — if a previous dispatch is still in flight (< 2h), do nothing
2. **Quality check** — if `quality-check` reports failures, do nothing (fix quality first)
3. **Feature advancement** — if any feature has an active directive, advance it one step
4. **Wave start** — if a milestone has all features PLANNED/READY, start the wave
5. **Idle** — nothing actionable, exit cleanly

One action per tick. The 4-hour recurrence IS the iteration rhythm.

---

## Default action recipe

Create this action once in the sdlc UI or via CLI:

```
Label:      dev-driver
Tool:       dev-driver
Input:      {}
Recurrence: 14400    (4 hours in seconds)
```

Then run `sdlc ui --run-actions` to enable the orchestrator.

---

## Priority waterfall (detail)

### Level 1: Active run check

Queries `sdlc run list --status running`. If any agent run is currently in flight,
exits immediately with `{ action: "waiting", reason: "agent run in progress" }`.

This prevents double-dispatch when a previous Claude agent is still running.
Flight detection is exact — based on the server's `agent_runs` map — not a TTL.

### Level 2: Quality check

Runs the `quality-check` tool. If any checks fail, exits with:
```json
{ "action": "quality_failing", "failed_checks": ["test", "lint"] }
```

Fix the failing checks before dev-driver will advance features.

### Level 3: Feature advancement

Finds features in `implementation`, `review`, `audit`, or `qa` phase with a
pending directive. Picks the first one alphabetically. Dispatches via the server:

```
POST /api/tools/agent-dispatch
{ "prompt": "/sdlc-next <slug>", "run_key": "dev-driver:feature:<slug>", "label": "dev-driver: advance <slug>" }
```

**This is `/sdlc-next` — one step only.** The 4-hour recurrence advances the feature
step by step over time. This is intentional: it keeps you in control and lets you
course-correct between steps.

The dispatched run appears in the activity feed and emits SSE events. If the server
returns 409 (run already in flight for this key), dev-driver returns `waiting`.

Returns:
```json
{ "action": "feature_advanced", "slug": "my-feature", "phase": "implementation", "directive": "/sdlc-next my-feature", "run_id": "20260303-120000-abc" }
```

### Level 4: Wave start

If no features have active directives but a milestone has all features in PLANNED or READY
phase, starts the next wave via the server:

```
POST /api/tools/agent-dispatch
{ "prompt": "/sdlc-run-wave <milestone>", "run_key": "dev-driver:wave:<milestone>", "label": "dev-driver: run wave <milestone>" }
```

Returns:
```json
{ "action": "wave_started", "milestone": "v21-dev-driver", "run_id": "20260303-120000-xyz" }
```

### Level 5: Idle

No actionable work found. Returns:
```json
{ "action": "idle", "reason": "no actionable work found" }
```

---

## How to skip a feature

If you don't want dev-driver to autonomously advance a specific feature, add a task
with `skip:autonomous` in the title:

```bash
sdlc task add <slug> --title "skip:autonomous: needs human review before proceeding"
```

Dev-driver will exclude this feature from Level 3 selection until the task is removed
or marked done. You retain full control over which features advance autonomously.

---

## One step, not full run

Dev-driver dispatches `/sdlc-next <slug>`, NOT `/sdlc-run <slug>`.

`/sdlc-next` executes exactly one directive (write a spec, approve a design, implement
a task, etc.) and exits. The next tick, dev-driver will pick the same feature again
and advance it one more step.

This means:
- Each tick = one atomic state machine step
- You can review after each step in the Actions page
- No surprise full-feature runs that take hours

---

## Dispatch

Dev-driver dispatches agent runs via `POST /api/tools/agent-dispatch` on the local
sdlc-server. Each dispatched run:

- Creates a **RunRecord** visible in the activity feed
- Streams **SSE events** to the frontend
- Is keyed under `dev-driver:feature:<slug>` or `dev-driver:wave:<milestone>` — the
  server returns 409 Conflict if that key is already in flight, which dev-driver
  treats as a "waiting" signal

This replaces the old TTL-based `.sdlc/.dev-driver.lock` file, which was imprecise
and invisible to the frontend.

Requires `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` (injected automatically by the
server for every tool subprocess). Running dev-driver outside the server (e.g. `sdlc
tool run dev-driver` without a running server) will fail with a clear error.

---

## Output reference

All five possible outputs:

```json
// Level 1 (active run in flight)
{ "action": "waiting", "reason": "agent run in progress" }

// Level 2
{ "action": "quality_failing", "failed_checks": ["test", "clippy"] }

// Level 3
{ "action": "feature_advanced", "slug": "my-feature", "phase": "implementation", "directive": "/sdlc-next my-feature", "run_id": "20260303-120000-abc" }

// Level 4
{ "action": "wave_started", "milestone": "v21-dev-driver", "run_id": "20260303-120001-xyz" }

// Level 5
{ "action": "idle", "reason": "no actionable work found" }
```

All wrapped in: `{ "ok": true, "data": { ... }, "duration_ms": N }`
