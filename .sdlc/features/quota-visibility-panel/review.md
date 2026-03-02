# Review: Quota Visibility Panel

## Implementation Summary

Three files changed:

1. `frontend/src/lib/types.ts` — added `observability?: { daily_budget_usd?: number }` to `ProjectConfig`
2. `frontend/src/components/layout/QuotaPanel.tsx` — new component (55 lines)
3. `frontend/src/components/layout/AgentPanel.tsx` — integrated QuotaPanel, added config fetch on mount

## Correctness Review

### Spec compliance

- Panel appears at the bottom of the Agent Activity panel: **yes** — placed in a fixed `<div>` outside the scroll container so it is always visible regardless of run list length.
- Panel recalculates after run completes without refresh: **yes** — derives values from `runs` in `AgentRunContext`; when SSE `run_finished` fires, the context updates runs, triggering a re-render.
- Progress bar at `(total_cost_today / daily_budget) * 100`: **yes** — `barPct = Math.min(pct, 100)` for bar width; raw `pct` shown in label.
- Remaining estimate only when ≥ 2 completed runs: **yes** — `completedToday.length >= 2` guard.
- Warning at ≥ 80%: **yes** — `isWarning = pct >= 80 && pct < 100`; `AlertTriangle` icon appears.
- Zero state renders without errors: **yes** — all values default to 0, no division attempted when no runs.
- Default budget $10.00: **yes** — `DEFAULT_DAILY_BUDGET_USD = 10.0` hardcoded constant.
- No new API calls for quota data: **yes** — reads from `useAgentRuns()` already populated in context.

### Logic review

- `isToday()` uses local browser time — matches spec and handles timezone correctly.
- Running runs filtered: `r.status !== 'running'` guard — correct, a running run has no settled `cost_usd`.
- Runs with no `cost_usd` filtered: `r.cost_usd != null` guard — correct.
- Average cost denominator uses `completedToday.length` (count of `status === 'completed'` runs). Stopped/failed runs with cost count toward `totalCostToday` but not the average — minor inconsistency, acceptable for MVP.
- `remainingRuns` clamped to 0: `Math.max(0, ...)` prevents negative display.
- `avgCostPerRun > 0` guard prevents division by zero.

### Visual states

- Amber at ≥ 80%: `isWarning = pct >= 80 && pct < 100` — correct.
- Red at ≥ 100%: `isExceeded = pct >= 100` — correct.
- "Daily budget exceeded" replaces remaining estimate when exceeded — correct per spec.

### Fullscreen modal

QuotaPanel rendered in both the sidebar and fullscreen modal variants. Both receive the same `dailyBudgetUsd` prop from the shared config state in `AgentPanel` — correct.

## TypeScript

`npx tsc --noEmit` passes with zero errors. No `any` types introduced. All props fully typed.

## Findings

### F1 — Stopped/failed run cost excluded from average (minor, accept)
Stopped and failed runs count toward `totalCostToday` but not the `completedToday` average. The average could be slightly underestimated if users frequently stop runs. Impact is minor; behavior is intuitive enough. Accepted as-is.

### F2 — Config fetch on AgentPanel mount only (minor, accept)
If the user edits `config.yaml` while the panel is open, the budget won't refresh until the panel is closed and reopened. Since daily budget is rarely changed mid-session, this is acceptable. Can be improved with SSE subscription in a future pass.

### F3 — No unit tests for quota derivation logic (minor, track as task)
The cost/percentage/remaining derivation is pure arithmetic in the component body. A utility function with unit tests would be more robust. Tracked as a follow-on task.

## Resolution

- F1: Accepted — documented rationale above.
- F2: Accepted — documented rationale above.
- F3: Tracked as future task.

**Verdict: APPROVE.**
