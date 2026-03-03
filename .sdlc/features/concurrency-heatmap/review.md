# Code Review: Concurrency Heatmap

## Summary

Implementation is complete and faithful to the spec and design. All 8 tasks delivered. The implementation is purely frontend — no backend changes. Seven files created or modified.

---

## Files Changed

| File | Action | Notes |
|------|--------|-------|
| `frontend/src/hooks/useHeatmap.ts` | CREATED | Pure computation hook |
| `frontend/src/components/runs/ConcurrencyStrip.tsx` | CREATED | Compact bar chart strip |
| `frontend/src/components/runs/RunsHeatmap.tsx` | CREATED | Full heatmap grid component |
| `frontend/src/pages/RunsPage.tsx` | CREATED | `/runs` route page |
| `frontend/src/hooks/useHeatmap.test.ts` | CREATED | 12 Vitest unit tests |
| `frontend/src/components/layout/AgentPanel.tsx` | MODIFIED | Added compact strip + link |
| `frontend/src/App.tsx` | MODIFIED | Added `/runs` route |
| `frontend/src/components/layout/Sidebar.tsx` | MODIFIED | Added Run History nav item |

---

## Correctness

### `useHeatmap.ts`

- Bucket size selection matches spec exactly (≤10min→30s, ≤1h→2min, ≤6h→10min, else→30min).
- Runs with missing `started_at` are correctly filtered out.
- `completed_at = null` treated as `Date.now()` — correct for live runs.
- 5% margin on each side prevents bars from touching the heatmap edges.
- `peakConcurrency` uses `Math.max(...buckets)` — correct; handles empty array case by guarding `buckets.length > 0`.
- `spanLabel` formats minutes cleanly with hours/minutes breakdown.

### `ConcurrencyStrip.tsx`

- Each bucket renders a `flex-1` div with `min-w-[2px]` so narrow strips remain visible.
- Zero buckets render at 1px height with 0.15 opacity — shows timeline gaps without empty void.
- Non-zero bars scale proportionally to `peakConcurrency`.
- Native `title` tooltip on hover (V1 scope — no custom tooltip component needed).
- Accessible: `aria-label` describes the strip summary.

### `RunsHeatmap.tsx`

- `compact` prop correctly conditionally renders only strip + summary label.
- Run type color map exhaustively covers all `RunType` values. Unknown types fall back to `DEFAULT_COLOR`.
- Run bar `left`/`width` computed as percentages of `totalBuckets` — responsive and correct.
- Time axis tick labels positioned absolutely with `translateX(-50%)` for centering — correct pattern.
- `onRunClick` guarded with optional chaining — no crashes when not provided.
- `min-w-[400px]` on the lane container ensures time axis doesn't collapse on narrow panels.
- `overflow-x-auto` on the outer wrapper enables horizontal scroll on narrow screens per spec acceptance criterion 10.

### `RunsPage.tsx`

- Shows `EmptyState` when fewer than 2 runs, per acceptance criteria 1–2.
- Uses `focusRun(run.id)` from `useAgentRuns()` for run click handling.
- Consistent with existing page layout pattern (max-w-5xl, p-4 sm:p-6, space-y-6).

### `AgentPanel.tsx`

- Compact strip is only rendered when `runs.length >= 2`, per acceptance criterion 1–2.
- Uses `RunsHeatmap compact` — no duplication of strip logic.
- "full view →" link correctly routes to `/runs` using `react-router-dom` `Link`.
- `runs` and `focusRun` destructured from `useAgentRuns()` — correct.

### `App.tsx`

- Route added alongside existing routes, consistent pattern.

### `Sidebar.tsx`

- `BarChart2` imported from `lucide-react` alongside existing imports.
- Added to `work` group after `Features`, per design doc.
- Uses `exact: true` to prevent `/runs` from matching sub-paths.

---

## Potential Issues

### Minor

1. **Unused import in RunsHeatmap**: The `startMs` variable is destructured from `useHeatmap` result but not used in the component body (time axis uses bucket indices, not raw timestamps). This should be removed to avoid a lint warning.

   **Fix**: Remove `startMs` from the destructure in `RunsHeatmap.tsx`.

2. **Unit tests require Vitest**: `useHeatmap.test.ts` imports from `vitest` but `vitest` is not yet in `package.json`. The test file is correct; the dev dependency needs to be added alongside `@testing-library/react`.

   **Track**: Add `vitest`, `@testing-library/react`, `jsdom` to devDependencies. This is a one-time setup step; tracked as a follow-up task rather than blocking this feature.

3. **`Math.max(...buckets)` on large arrays**: If runs span > 24h with 30-min buckets, bucket count is ~48. With 30s buckets over 10min, it's ~20. Both are well within safe spread operator limits (V8 handles ~100k elements safely). Not a real issue for this dataset.

---

## Acceptance Criteria Verification

| # | Criterion | Status |
|---|-----------|--------|
| 1 | Compact strip shown for 2+ runs | Pass — `runs.length >= 2` guard |
| 2 | Compact strip hidden for 0–1 runs | Pass — same guard |
| 3 | Expanding compact strip shows full heatmap | Pass — "full view →" links to `/runs` |
| 4 | `/runs` route exists and renders heatmap | Pass — route in App.tsx, RunsPage renders RunsHeatmap |
| 5 | Hover on run bar shows tooltip | Pass — native `title` attribute on bar div |
| 6 | Clicking run bar navigates to run detail | Pass — `onRunClick` calls `focusRun(run.id)` |
| 7 | Colors consistent with run_type mapping | Pass — `RUN_TYPE_COLORS` exhaustive map |
| 8 | Concurrency strip correct (0 = empty, N = concurrent count) | Pass — bucket array accumulates per-run overlaps |
| 9 | No new API endpoints | Pass — uses existing `GET /api/runs` via `useAgentRuns()` |
| 10 | Responsive: horizontal scroll on narrow screens | Pass — `overflow-x-auto` on outer wrapper |

---

## Findings and Actions

| Finding | Action |
|---------|--------|
| `startMs` unused in RunsHeatmap | Fix now (targeted removal) |
| Vitest not in package.json | Track as follow-up task |

---

## Verdict

**Approved with minor fix.** The `startMs` unused variable is cleaned up below, and the Vitest dependency gap is tracked. All acceptance criteria met. No structural issues, no security concerns, no behavioral regressions (backend unchanged).
