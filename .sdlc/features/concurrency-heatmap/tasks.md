# Tasks: Concurrency Heatmap

## Task List

- [ ] T1: Create `frontend/src/hooks/useHeatmap.ts` — pure computation hook that takes `RunRecord[]` and returns `HeatmapData` (bucket array, run lanes, peakConcurrency, spanLabel, bucketSizeMs). Include edge-case handling: runs with no `started_at` excluded; `completed_at = null` treated as `Date.now()`; 5% margin on time range; adaptive bucket size per spec.

- [ ] T2: Create `frontend/src/components/runs/ConcurrencyStrip.tsx` — SVG or flex-based single-row bar chart. Each bucket renders a vertical bar with height proportional to concurrency count and opacity-based color. Zero buckets render as a faint 1px line. Hover shows native `title` tooltip with count. Full-width, responsive.

- [ ] T3: Create `frontend/src/components/runs/RunsHeatmap.tsx` — full heatmap grid component with `compact` prop. Full mode: `ConcurrencyStrip` + run lane rows (label column + colored bar) + time axis ticks. Compact mode: `ConcurrencyStrip` + summary label line only. `onRunClick` callback on bar click. Uses `useHeatmap` hook internally.

- [ ] T4: Create `frontend/src/pages/RunsPage.tsx` — `/runs` page. Imports `useAgentRuns` for runs data and `useAgentRuns().focusRun` for run click handling. Renders `RunsHeatmap` at full width or an empty state when fewer than 2 runs exist. Follows existing page layout patterns (max-w-5xl, p-4 sm:p-6, space-y-6).

- [ ] T5: Modify `frontend/src/components/layout/AgentPanel.tsx` — add compact heatmap strip above `RunList`. Only shown when `runs.length >= 2`. Renders `<RunsHeatmap runs={runs} compact onRunClick={focusRun} />`. Add a "View full →" link to `/runs` in the panel header row. Clicking a bar in compact mode calls `focusRun(run.id)`.

- [ ] T6: Modify `frontend/src/App.tsx` — add `<Route path="/runs" element={<RunsPage />} />` alongside existing routes. Import `RunsPage`.

- [ ] T7: Modify `frontend/src/components/layout/Sidebar.tsx` — add `{ path: '/runs', label: 'Run History', icon: BarChart2, exact: true }` to the `work` nav group (after Features). Import `BarChart2` from lucide-react.

- [ ] T8: Write unit tests for `useHeatmap` in `frontend/src/hooks/useHeatmap.test.ts` — test cases: empty array → no-op; single run → 1 lane, no concurrency; two overlapping runs → concurrency=2 in shared buckets; two non-overlapping runs → concurrency=0 between them; run with null `completed_at` → treated as live. Use Vitest.
