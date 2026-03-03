# Code Review: dev-driver tool

## Summary

Reviewing `.sdlc/tools/dev-driver/tool.ts` (300 lines) and `.sdlc/tools/dev-driver/README.md`.

---

## Findings

### ✅ Structural correctness

The tool follows the quality-check pattern exactly:
- `export const meta: ToolMeta` — present, well-formed
- `export async function run(input, root)` — present, correct signature
- CLI entrypoint at bottom (`--meta` / `--run` modes) — present
- All log output to STDERR, JSON only on STDOUT — correct
- No `unwrap()` equivalents; all errors are caught per-level — correct

### ✅ Priority waterfall

All 5 levels implemented in order with early returns:
1. Flight lock (L1) — reads lock, checks TTL, returns `waiting` if active
2. Quality check (L2) — runs qc tool, returns `quality_failing` if failed > 0
3. Active run check (between L2 and L3) — guards against concurrent dispatches
4. Feature selection (L3) — filters by phase, skip tag, sorts alphabetically
5. Wave detection (L4) — finds milestones with all features PLANNED/READY
6. Idle (L5) — clean exit

### ✅ One step invariant

Level 3 dispatches `/sdlc-next <slug>` with comment:
```
// Intentionally /sdlc-next — one step per tick, not /sdlc-run to completion
```
No `/sdlc-run` appears in the dispatch logic. T7 satisfied.

### ✅ Flight lock

- Reads and writes `.sdlc/.dev-driver.lock` as JSON
- TTL check: `ageMs < 120 * 60 * 1000`
- `writeLock` called before `spawnClaude` for both Level 3 and Level 4
- Stale lock is logged and ignored correctly

### ✅ Skip mechanism

`hasSkipTag(slug, root)` reads tasks.md and checks for `/skip:autonomous/i`.
Features with matching task titles are filtered before sort/select in Level 3.
Documented in README with example command.

### ✅ Active run check

`hasActiveRuns(root)` calls `sdlc run list --status running --json`.
Wrapped in try/catch — gracefully falls through if command doesn't exist.
Returns `{ action: "waiting", reason: "agent run in progress" }` if runs found.

### ✅ Output schema

TypeScript discriminated union covers all 6 output shapes.
`ok: true` for all non-error outcomes (waiting, quality_failing, idle, feature_advanced, wave_started).
`ToolResult<DevDriverOutput>` matches the shared type contract.

### ✅ README

7 sections present:
1. What it does
2. Default action recipe (label, tool, input, recurrence: 14400s)
3. Priority waterfall (5 levels with detail)
4. How to skip a feature (`skip:autonomous` task tag)
5. One step vs /sdlc-run explanation
6. Lock file (path, TTL, format)
7. Output reference (all 5 action discriminants with example JSON)

### Minor findings

**M1 — Lock file written twice on dispatch**

The current code writes lock with `pid: 0` then overwrites with the real PID. This is two
atomic writes in rapid succession with a tiny window between them. Functionally harmless
since the lock check only uses `started_at`, not `pid`.

_Resolution: Accept as-is. The double-write is intentional to record the real PID for
observability, and the window is < 1ms on local filesystems._

**M2 — Wave detection filters released features**

`WAVE_READY_PHASES.has(f.phase)` plus `f.phase === 'released'` in `findReadyWave`.
This correctly handles milestones where some features are already released — they're
treated as done and don't block the wave-ready check.

_Resolution: Correct behavior, no change needed._

**M3 — Quality check invocation path**

Uses full path: `node ${toolPath} --run`. On projects without `quality-check` installed,
returns gracefully `{ failed: 0, failedNames: [] }`. This is the correct fallback.

_Resolution: Correct. No hardening needed._

---

## Verdict: APPROVED

All spec requirements met. All 10 tasks implemented. Tool is functional (verified with
`node .sdlc/tools/dev-driver/tool.ts --meta` and `--run`). Three minor findings are
all accepted as-is with documented rationale.
