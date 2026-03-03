# Tasks: dev-driver tool

## T1 — Scaffold dev-driver/tool.ts following quality-check pattern

Create `.sdlc/tools/dev-driver/tool.ts` with:
- File header comment block explaining what the tool does
- Import block: `node:child_process`, `node:fs`, `node:path`, shared types + runtime
- `export const meta: ToolMeta` with full JSON Schema for input/output
- `export async function run(input: {}, root: string): Promise<ToolResult<DevDriverOutput>>`
- CLI entrypoint at bottom (`--meta` and `--run` modes)

**Acceptance:** `node .sdlc/tools/dev-driver/tool.ts --meta` returns valid JSON matching ToolMeta shape

---

## T2 — Implement flight lock read/write/check

- `readLock(root)` → parses `.sdlc/.dev-driver.lock`, returns null if missing or invalid
- `isLockActive(lock)` → `(Date.now() - Date.parse(lock.started_at)) / 60000 < 120`
- `writeLock(root, payload)` → writes lock file atomically before dispatch
- Level 1 check: if `readLock(root)` && `isLockActive(lock)` → return `{ action: "waiting", lock_age_mins: N }`

**Acceptance:** Lock file presence and age correctly gates dispatch

---

## T3 — Implement priority waterfall levels 1-5

Level 1: flight lock (implemented in T2)
Level 2: quality check — `runQualityCheck(root)` parses qc output, returns failed check names
Level 3: feature selection — `sdlc next --json`, filter by phase + action + skip tag, sort by slug
Level 4: wave detection — `sdlc milestone list --json`, find milestone with all features PLANNED/READY
Level 5: idle — return `{ action: "idle", reason: "no actionable work found" }`

Each level returns early if it matches. No fallthrough.

**Acceptance:** Each level fires independently; running dev-driver on a project with
different states produces the correct action discriminant at each level

---

## T4 — Implement async spawn for Claude dispatch

```typescript
spawn('claude', ['--print', `/sdlc-next ${slug}`], {
  detached: true,
  stdio: 'ignore',
  cwd: root,
  env: { ...process.env, SDLC_ROOT: root },
})
child.unref()
```

Same pattern for wave: `spawn('claude', ['--print', `/sdlc-run-wave ${milestone}`], ...)`

**Acceptance:** Spawned process runs after parent exits; parent exits in < 1 second

---

## T5 — Define and validate output schema

TypeScript discriminated union for all output shapes:
- `{ action: "waiting"; lock_age_mins: number }` (Level 1)
- `{ action: "waiting"; reason: string }` (active run check)
- `{ action: "quality_failing"; failed_checks: string[] }` (Level 2)
- `{ action: "feature_advanced"; slug: string; phase: string; directive: string }` (Level 3)
- `{ action: "wave_started"; milestone: string }` (Level 4)
- `{ action: "idle"; reason: string }` (Level 5)

All wrapped in `ToolResult<DevDriverOutput>` with `ok: true` for all non-error cases.

**Acceptance:** `--meta` output_schema matches the union; TypeScript compiles cleanly

---

## T6 — Write dev-driver/README.md

Document:
1. What it does (one paragraph — autonomous, one step per tick)
2. Default action recipe (label, tool, input, recurrence: 14400s)
3. Priority waterfall (5 levels as bullets)
4. How to skip a feature (add task matching `skip:autonomous`)
5. One step vs /sdlc-run (4h recurrence is the rhythm)
6. Lock file (path, TTL, format)
7. Output reference (all 5 action discriminants with example JSON)

**Acceptance:** README answers: "what does this do?", "how do I stop it from touching a feature?", "why is only one step taken?"

---

## T7 — [user-gap] Use /sdlc-next (one step) not /sdlc-run for feature advancement

Level 3 dispatch must use `/sdlc-next <slug>` exclusively. Never `/sdlc-run <slug>`.
Add a code comment at the spawn call: `// Intentionally /sdlc-next — one step per tick`
Document in README that this is by design.

**Acceptance:** No `/sdlc-run` appears in dispatch logic; comment is present

---

## T8 — [user-gap] Check for any active agent runs before dispatching

Before Level 3 dispatch, run:
```typescript
const runs = execSync('sdlc run list --status running --json', { ... })
if (JSON.parse(runs).length > 0) return { ok: true, data: { action: "waiting", reason: "agent run in progress" } }
```

Wrap in try/catch — if `sdlc run list` doesn't exist, log warning and skip this check.

**Acceptance:** Tool exits with `{ action: "waiting" }` when any sdlc agent run is active

---

## T9 — [user-gap] Surface dev-driver decision in action output

The `directive` field in `feature_advanced` must contain the next command string from
`sdlc next --for <slug> --json` → `next_command` field. This makes the Actions UI show:
- Which feature was picked
- Which directive was dispatched
- What phase it was in

**Acceptance:** `{ action: "feature_advanced", slug: "my-feature", phase: "implementation", directive: "/sdlc-next my-feature" }` visible in Actions page output

---

## T10 — [user-gap] Document skip mechanism in README

In README section "How to skip a feature":
```bash
sdlc task add <slug> --title "skip:autonomous: human review needed before proceeding"
```
Explain that any task whose title contains `skip:autonomous` causes the feature to be
excluded from Level 3 selection until the task is done or removed.

**Acceptance:** README clearly explains how to opt a feature out of autonomous advancement
