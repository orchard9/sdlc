# Spec: dev-driver — stock tool that finds and does the next development action

## Problem

A developer using sdlc must manually trigger every step. After `/sdlc-run` completes a feature,
they need to remember to run `/sdlc-prepare`, then `/sdlc-run-wave`, then `/sdlc-run` for each
feature in the next wave. Between sessions this work stalls.

The `dev-driver` tool closes this gap: it reads project state, applies a priority waterfall,
picks the single most important next development action, dispatches it asynchronously, and exits.
Paired with a recurring orchestrator action (every 4 hours), it turns sdlc into a
self-advancing system.

---

## What it does

`dev-driver` is a stock sdlc tool (like `quality-check` and `ama`) that takes `{}` as input and
returns a structured result describing what it did.

### Priority waterfall

The tool evaluates conditions in order. First match wins. One action per invocation.

**Level 1 — Flight lock check**
- Read `.sdlc/.dev-driver.lock`
- If file exists AND `started_at` is < 2 hours old → exit immediately
- Output: `{ action: "waiting", lock_age_mins: N }`
- No agent spawned, no side effects

**Level 2 — Quality check**
- Run the `quality-check` tool (via its `--run` mode, piping `{}` on stdin)
- If `failed > 0` → exit immediately
- Output: `{ action: "quality_failing", failed_checks: ["check-name", ...] }`
- No agent spawned, no side effects

**Level 3 — Features with active directives**
- Run `sdlc next --json` (all features)
- Filter to features where `action !== "done"` and phase is in: `implementation`, `review`, `audit`, `qa`
- Exclude features with a task tagged `skip:autonomous`
- Check for any actively running sdlc agent runs in the DB (via `sdlc run list --status running`)
- If any runs are active → treat as if lock exists, exit with `{ action: "waiting", reason: "agent run in progress" }`
- If features found → pick first alphabetically by slug
- Run: `spawn("claude", ["--print", "/sdlc-next <slug>"], { detached: true, stdio: "ignore" })`
- Write lock file: `{ started_at: ISO, action: "feature_advanced", slug: "<slug>", pid: N }`
- Output: `{ action: "feature_advanced", slug: "<slug>", phase: "<phase>", directive: "<next_command>" }`

**Level 4 — Wave ready**
- Run `sdlc project prepare --dry-run` (or equivalent) to detect milestones with all features in PLANNED/READY
- If a wave is ready → pick first milestone alphabetically
- Run: `spawn("claude", ["--print", "/sdlc-run-wave <milestone>"], { detached: true, stdio: "ignore" })`
- Write lock file: `{ started_at: ISO, action: "wave_started", milestone: "<slug>", pid: N }`
- Output: `{ action: "wave_started", milestone: "<slug>" }`

**Level 5 — Idle**
- Nothing matches → exit cleanly
- Output: `{ action: "idle", reason: "no actionable work found" }`

### Key invariant: one step, not full run

Level 3 dispatches `/sdlc-next <slug>` — NOT `/sdlc-run <slug>`. The 4-hour recurrence IS the
iteration rhythm. Each tick advances exactly one feature by one directive. This keeps the
developer in control: they can review after each step and course-correct.

---

## Flight lock format

File: `.sdlc/.dev-driver.lock`
```json
{
  "started_at": "2026-03-02T04:00:00.000Z",
  "action": "feature_advanced",
  "slug": "my-feature",
  "pid": 12345
}
```

TTL: 2 hours from `started_at`. Lock is cleared by the spawned agent process when it finishes,
or by the TTL if the process exits without cleaning up.

The lock is overwritten each time a dispatch succeeds. It is NOT overwritten when exiting with
`waiting`, `quality_failing`, or `idle`.

---

## Input / output schema

### Input
```typescript
type DevDriverInput = {} // no parameters
```

### Output
```typescript
type DevDriverAction =
  | { action: "waiting"; lock_age_mins: number }
  | { action: "waiting"; reason: string }
  | { action: "quality_failing"; failed_checks: string[] }
  | { action: "feature_advanced"; slug: string; phase: string; directive: string }
  | { action: "wave_started"; milestone: string }
  | { action: "idle"; reason: string }

type DevDriverOutput = ToolResult<DevDriverAction>
```

`ok: true` for all cases except internal tool errors.
The action discriminant (`action` field) conveys the semantic outcome.

---

## Tool file structure

```
.sdlc/tools/dev-driver/
  tool.ts       ← the implementation (this spec)
  README.md     ← user documentation
```

Follows the same pattern as `quality-check`:
- `export const meta: ToolMeta` — tool metadata
- `export async function run(input: {}, root: string): Promise<ToolResult<DevDriverAction>>`
- CLI entrypoint at bottom: `--meta` and `--run` modes

---

## Skip a feature

A feature can be excluded from autonomous advancement by adding a task tagged `skip:autonomous`:
```bash
sdlc task add <slug> --title "skip:autonomous: human review needed before next step"
```

The tool filters out features with any task title matching `/skip:autonomous/`.

---

## Active run check

Before dispatching, the tool also checks for any running sdlc agent runs:
```bash
sdlc run list --status running --json
```

If any runs are active, the tool exits with `{ action: "waiting", reason: "agent run in progress" }`.
This prevents double-dispatch even if the lock file is somehow absent.

---

## Quality check integration

The tool invokes quality-check via its standard CLI interface:
```bash
node .sdlc/tools/quality-check/tool.ts --run <<< '{}'
```

Parses the `ToolResult<{ passed, failed, checks }>` output. If `failed > 0`, reports the
`name` of each failed check in `failed_checks`.

---

## Acceptance criteria

1. Running dev-driver advances exactly ONE feature by ONE directive per invocation
2. If `.sdlc/.dev-driver.lock` is < 2h old, exits immediately with `waiting`
3. If quality checks fail, exits with `quality_failing` and does not dispatch
4. If any sdlc agent run is active, exits with `waiting` even if no lock exists
5. Features tagged `skip:autonomous` are excluded from Level 3
6. The dispatch uses `/sdlc-next <slug>` — never `/sdlc-run <slug>`
7. The output JSON matches the `DevDriverAction` discriminated union
8. `ok: true` in all non-error outcomes (including `waiting`, `idle`)
9. Lock file is written before `exit()` returns in Level 3 and Level 4
10. The tool follows the `quality-check` code pattern exactly (meta, run, CLI entrypoint)

---

## Tasks in scope

- T1: Scaffold dev-driver/tool.ts following quality-check pattern (meta + run exports)
- T2: Implement flight lock read/write/check (.sdlc/.dev-driver.lock, 2h TTL)
- T3: Implement priority waterfall levels 1-5
- T4: Implement async spawn for Claude dispatch (detached: true, stdio: ignore)
- T5: Define and validate output schema (ToolResult<DevDriverResult>)
- T6: Write dev-driver/README.md
- T7: Use /sdlc-next (one step) not /sdlc-run for feature advancement
- T8: Check for any active agent runs before dispatching
- T9: Surface dev-driver decision in action output
- T10: Document in README: autonomous advancement model and how to skip a feature
