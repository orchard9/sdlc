# Design: dev-driver tool

## Architecture overview

`dev-driver` is a TypeScript file that runs under `node` (same runtime as all stock sdlc tools).
It reads project state, applies a deterministic priority waterfall, optionally spawns a detached
agent process, and exits. No server, no network, no persistent process — just a short-lived script.

```
orchestrator tick
      │
      ▼
dispatch(dev-driver, {})
      │
      ▼
node .sdlc/tools/dev-driver/tool.ts --run  ← reads stdin: '{}'
      │
      ├─ Level 1: read .sdlc/.dev-driver.lock
      │     ├─ exists and < 2h old → print { action: "waiting" } → exit 0
      │     └─ absent or stale → continue
      │
      ├─ Level 2: quality-check
      │     ├─ failed > 0 → print { action: "quality_failing" } → exit 0
      │     └─ all passed → continue
      │
      ├─ Level 3: features with active directives
      │     ├─ sdlc next --json → filter actionable features
      │     ├─ check sdlc run list --status running
      │     ├─ if active runs → print { action: "waiting" } → exit 0
      │     ├─ features found → pick first by slug, write lock, spawn claude
      │     │     └─ print { action: "feature_advanced" } → exit 0
      │     └─ no features → continue
      │
      ├─ Level 4: wave ready
      │     ├─ sdlc milestone list → find waves in PLANNED/READY
      │     ├─ found → write lock, spawn claude
      │     │     └─ print { action: "wave_started" } → exit 0
      │     └─ not found → continue
      │
      └─ Level 5: idle
            └─ print { action: "idle" } → exit 0
```

## File layout

```
.sdlc/tools/dev-driver/
  tool.ts       ← single implementation file (300-400 lines)
  README.md     ← user documentation (how it works, how to configure)
```

No config.yaml needed — the tool has no external configuration.
The priority waterfall logic is not data-driven; it's code.

## Lock file design

Path: `.sdlc/.dev-driver.lock`

```json
{
  "started_at": "2026-03-02T04:00:00.000Z",
  "action": "feature_advanced",
  "slug": "my-feature",
  "pid": 12345
}
```

**Read logic:**
1. Try to read and parse `.sdlc/.dev-driver.lock`
2. If file does not exist → no lock (proceed)
3. If parse fails → treat as stale (proceed)
4. Compute age: `(Date.now() - Date.parse(lock.started_at)) / 60000` minutes
5. If age < 120 minutes → locked (exit with `waiting`)

**Write logic:**
- Written immediately before `spawn()` is called
- Never written for `waiting`, `quality_failing`, or `idle` outcomes

## Spawn design

```typescript
import { spawn } from 'node:child_process'

const child = spawn(
  'claude',
  ['--print', `/sdlc-next ${slug}`],
  {
    detached: true,
    stdio: 'ignore',
    cwd: root,
    env: { ...process.env, SDLC_ROOT: root },
  }
)
child.unref()
```

`child.unref()` is critical — allows the parent process (dev-driver) to exit without waiting
for the child. The orchestrator tick completes. The Claude agent runs independently.

`SDLC_ROOT` is passed so the agent knows the project root.

## Quality check integration

```typescript
import { execSync } from 'node:child_process'

const raw = execSync(
  `node ${join(root, '.sdlc/tools/quality-check/tool.ts')} --run`,
  { input: '{}', encoding: 'utf8', cwd: root }
)
const result = JSON.parse(raw) as ToolResult<QCOutput>
```

Parses `result.data.failed` and `result.data.checks.filter(c => c.status === 'failed').map(c => c.name)`.

## Feature selection

```typescript
const allDirectives = JSON.parse(
  execSync('sdlc next --json', { encoding: 'utf8', cwd: root })
)

const actionable = allDirectives
  .filter(d => d.action !== 'done')
  .filter(d => ['implementation','review','audit','qa'].includes(d.current_phase))
  .filter(d => !hasSkipTag(d.feature, root))
  .sort((a, b) => a.feature.localeCompare(b.feature))
```

`hasSkipTag(slug, root)` reads `.sdlc/features/<slug>/tasks.md` and checks for a task
matching `/skip:autonomous/i`.

## Active run check

```typescript
const runsRaw = execSync('sdlc run list --status running --json', { encoding: 'utf8', cwd: root })
const runs = JSON.parse(runsRaw)
if (runs.length > 0) {
  // exit waiting
}
```

This is a belt-and-suspenders check separate from the lock file. Prevents dispatch if
any sdlc-managed agent is already running (e.g. a ponder run, investigation run, etc.).

If `sdlc run list --status running` doesn't exist yet, the tool gracefully falls back
(catches the error, logs a warning, continues).

## Output shape in action result

When the orchestrator records the result of running dev-driver, it stores the full
`ToolResult` JSON. The Actions UI displays the `data` field verbatim. This means the
`{ action: "feature_advanced", slug: "...", phase: "...", directive: "..." }` object is
visible to the developer in the Actions page without any special UI treatment.

## Error handling

All top-level errors (unexpected crashes) are caught and returned as:
```json
{ "ok": false, "error": "...", "duration_ms": N }
```

Individual level failures (e.g. quality-check fails to parse) are caught per-level and
treated as "skip this check" with a log warning, not as tool failure.

## README structure

1. What it does (one paragraph)
2. Setup (none required — just run)
3. Priority waterfall (5 levels, each a bullet)
4. How to skip a feature (`skip:autonomous` task tag)
5. What "one step" means (vs /sdlc-run)
6. Lock file (path, TTL, what it contains)
7. Default action recipe (label, tool, input, recurrence)
8. Output reference (the 5 action discriminants)
