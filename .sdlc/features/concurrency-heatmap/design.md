# Design: Concurrency Heatmap

## Overview

The concurrency heatmap is a purely frontend feature. It derives all data from `GET /api/runs` (already in `AgentRunContext`) and renders two surfaces:

1. **Compact strip** — added to the Agent Activity panel header (above the run list).
2. **Runs history page** — new route `/runs` with the full heatmap.

No backend changes are required.

---

## Architecture

```
AgentRunContext (existing)
  └── runs: RunRecord[]            ← already fetched, SSE-updated

useHeatmap(runs: RunRecord[])      ← new hook (pure computation)
  └── returns: HeatmapData

HeatmapData:
  bucketSizeMs: number
  startMs: number
  endMs: number
  buckets: number[]               ← concurrency count per bucket
  lanes: RunLane[]                ← one per run

RunLane:
  run: RunRecord
  startBucket: number             ← index into buckets[]
  endBucket: number               ← index into buckets[] (inclusive)

Components:
  ConcurrencyStrip                ← compact bar chart, single row
  RunsHeatmap                     ← full grid: strip + lanes + time axis
  RunsPage                        ← /runs route, wraps RunsHeatmap
```

---

## Hook: `useHeatmap`

File: `frontend/src/hooks/useHeatmap.ts`

```ts
export interface RunLane {
  run: RunRecord
  startBucket: number
  endBucket: number
}

export interface HeatmapData {
  bucketSizeMs: number
  startMs: number
  endMs: number
  buckets: number[]         // length = ceil((endMs - startMs) / bucketSizeMs)
  lanes: RunLane[]
  peakConcurrency: number
  spanLabel: string         // e.g. "43 minutes", "2h 14m"
}
```

**Bucket size selection:**
```
rangeMs <= 10min  → 30s buckets
rangeMs <= 1h     → 2min buckets
rangeMs <= 6h     → 10min buckets
else              → 30min buckets
```

**Computation:**
1. Filter to runs with a valid `started_at`. Treat `completed_at = null` as `Date.now()`.
2. Compute `startMs = min(started_at)`, `endMs = max(completed_at)`. Add 5% margin on each side.
3. Compute bucket array: for each run, increment `buckets[b]` for every bucket `b` overlapping `[run.startMs, run.endMs]`.
4. Map each run to a `RunLane` with `startBucket` and `endBucket`.
5. `peakConcurrency = max(buckets)`.
6. `spanLabel`: format `endMs - startMs` as human-readable.

The hook is pure: `useMemo(() => computeHeatmap(runs), [runs])`. No side effects.

---

## Component: `ConcurrencyStrip`

File: `frontend/src/components/runs/ConcurrencyStrip.tsx`

**Props:**
```ts
interface ConcurrencyStripProps {
  data: HeatmapData
  onBarClick?: (bucketIndex: number) => void
  height?: number          // px, default 24
}
```

**Rendering:**
- An `<svg>` (or `<div>` with flex) where each bucket is a vertical bar.
- Bar height = `(buckets[i] / peakConcurrency) * height` (scaled).
- Bar color: solid `hsl(var(--primary))` at 40%–100% opacity based on count.
- Zero-count buckets: rendered as 1px thin lines in `muted-foreground/20` (shows gap, not empty void).
- Width: `100%`, with each bar `max(2px, totalWidth / bucketCount)` wide.
- Tooltip on hover: `{count} active` (using a simple `title` attribute in V1 — no custom tooltip component needed).

---

## Component: `RunsHeatmap`

File: `frontend/src/components/runs/RunsHeatmap.tsx`

**Props:**
```ts
interface RunsHeatmapProps {
  runs: RunRecord[]
  onRunClick?: (run: RunRecord) => void
  compact?: boolean        // if true, only renders ConcurrencyStrip + summary label
}
```

**Full layout (not compact):**

```
┌─────────────────────────────────────────────────────────┐
│ Concurrency strip (full-width, height=32px)              │
├─────────────────────────────────────────────────────────┤
│ run label  │████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │  ← RunLane
│ run label  │░░░░░░░████████████████░░░░░░░░░░░░░░░░░░░ │
│ run label  │░░░░░░░░░░░░░░░░░████████████░░░░░░░░░░░░░ │
├─────────────────────────────────────────────────────────┤
│ +0m      +10m     +20m     +30m     +40m                 │  ← Time axis
└─────────────────────────────────────────────────────────┘
```

**Run lane:**
- Each lane is a `div` with `height: 20px`, `position: relative`.
- Label: left-aligned, `w-28 text-xs truncate text-muted-foreground` — fixed width column.
- Bar area: `flex-1`, `position: relative`, `overflow: hidden`.
- Bar: `position: absolute`, left/right derived from `startBucket/endBucket` percentages.
- Bar color: mapped from `run.run_type` (see color map below).
- On hover: bar gets a `ring-1` highlight; cursor is `pointer` if `onRunClick` provided.
- On click: calls `onRunClick(run)`.

**Run type color map** (Tailwind bg classes, dark-mode compatible):
```ts
const RUN_TYPE_COLORS: Record<string, string> = {
  feature:               'bg-blue-500/70',
  milestone_uat:         'bg-purple-500/70',
  milestone_prepare:     'bg-amber-500/70',
  milestone_run_wave:    'bg-amber-600/70',
  ponder:                'bg-teal-500/70',
  investigation:         'bg-teal-600/70',
  vision_align:          'bg-green-500/70',
  architecture_align:    'bg-green-600/70',
}
const DEFAULT_COLOR = 'bg-muted-foreground/40'
```

**Time axis:**
- Renders relative tick labels: `+0m`, `+Nm`, ... at even multiples of `bucketSizeMs * 5` (one tick every 5 buckets).
- `text-[10px] text-muted-foreground` with `justify-between` flex layout.

**Compact mode (`compact=true`):**
- Renders only `ConcurrencyStrip` plus a summary line: `{N} runs · peak {P} concurrent · span {spanLabel}`.
- No lane rows, no time axis.

---

## Component: `RunsPage`

File: `frontend/src/pages/RunsPage.tsx`

```tsx
export function RunsPage() {
  const { runs } = useAgentRuns()          // existing context
  const navigate = useNavigate()

  const handleRunClick = (run: RunRecord) => {
    // Re-use existing panel behavior: focus the run in the agent panel
    focusRun(run.id)
  }

  return (
    <div className="max-w-5xl mx-auto p-4 sm:p-6 space-y-6">
      <div className="flex items-center gap-2 mb-1">
        <BarChart2 className="w-5 h-5 text-muted-foreground" />
        <h2 className="text-xl font-semibold">Run History</h2>
      </div>
      <p className="text-sm text-muted-foreground">
        Cross-run concurrency view. Spot parallelism opportunities and idle gaps.
      </p>

      {runs.length < 2 ? (
        <EmptyState />
      ) : (
        <RunsHeatmap runs={runs} onRunClick={handleRunClick} />
      )}
    </div>
  )
}
```

---

## Agent Activity Panel Integration

File: `frontend/src/components/layout/AgentPanel.tsx` (modified)

**New panel header section** (above `RunList`):
- Shown only when `runs.length >= 2`.
- Renders `<RunsHeatmap runs={runs} compact />` (compact mode).
- A "View full heatmap" link (`→ /runs`) in `text-xs text-primary` at the right of the header.
- The compact strip auto-refreshes whenever `runs` updates from SSE (no extra wiring needed — AgentRunContext already handles this).

```
┌─────────────────────────────────────────────────────┐
│ Agent Activity              [⇲] [✕]                  │ ← existing header
├─────────────────────────────────────────────────────┤
│ ████░░░░░░██████░░░░░███░░░  3 runs · peak 2 · 43m │ ← ConcurrencyStrip (compact)
│                                          [full view→] │
├─────────────────────────────────────────────────────┤
│ [run card] ...                                       │ ← existing RunList
```

---

## Router Integration

File: `frontend/src/App.tsx` (modified)

Add one route:
```tsx
<Route path="/runs" element={<RunsPage />} />
```

---

## Sidebar Integration

File: `frontend/src/components/layout/Sidebar.tsx` (modified)

Add to `work` group:
```ts
{ path: '/runs', label: 'Run History', icon: BarChart2, exact: true }
```

---

## Files Modified / Created

| File | Action |
|------|--------|
| `frontend/src/hooks/useHeatmap.ts` | CREATE — pure computation hook |
| `frontend/src/components/runs/ConcurrencyStrip.tsx` | CREATE — compact bar chart |
| `frontend/src/components/runs/RunsHeatmap.tsx` | CREATE — full heatmap grid |
| `frontend/src/pages/RunsPage.tsx` | CREATE — `/runs` route page |
| `frontend/src/components/layout/AgentPanel.tsx` | MODIFY — add compact strip to header |
| `frontend/src/App.tsx` | MODIFY — add `/runs` route |
| `frontend/src/components/layout/Sidebar.tsx` | MODIFY — add Run History nav item |

No backend files are modified.

---

## No-New-API Rationale

The `RunRecord[]` from `GET /api/runs` already includes `started_at`, `completed_at`, `run_type`, `label`, and `id`. All concurrency data is computable from these fields client-side. Adding a backend endpoint for pre-computed heatmap data would be premature optimization — the dataset is small (50 records max due to existing retention) and computation is O(N * B) where N ≤ 50 and B ≤ 200 buckets.

---

## Edge Cases

| Condition | Behavior |
|-----------|----------|
| 0–1 runs | Compact strip hidden; `/runs` page shows empty state |
| All runs same instant | Single-bucket heatmap, width = full bar |
| Run with no `completed_at` | Use `Date.now()` (live run extends to right edge) |
| Very long gap between runs (> 24h) | Bucket size expands to 30min; gaps render as empty buckets |
| Panel width < 200px | Strip renders with minimum 2px bars; horizontal scroll on RunsPage |
