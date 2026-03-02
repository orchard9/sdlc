# Tasks: activity-time-series

## T1 — Add `ts?: string` to `RawRunEvent` in `types.ts`

Update `frontend/src/lib/types.ts` to add the optional `ts?: string` (ISO-8601 wall-clock timestamp) field to the `RawRunEvent` interface. This is the field written by the `telemetry-wallclock-timestamps` feature; adding it to the type here is the prerequisite for all other tasks.

**File**: `frontend/src/lib/types.ts`
**Change**: Add `ts?: string` to `RawRunEvent`

---

## T2 — Implement `buildTimeSeries.ts`

Create `frontend/src/components/runs/buildTimeSeries.ts` with the pure `buildTimeSeries` function and its exported types (`BucketData`, `TimeSeriesData`).

**Algorithm**:
1. Filter events with a truthy `ts` field; return `null` if fewer than 2
2. Compute `runStartMs` / `runEndMs` from first/last timestamps
3. Derive typed intervals:
   - `assistant` → next `user` = `llm`
   - `user` → next `assistant` = `tool`
   - `subagent_started` → matching `subagent_completed` = `subagent`
   - uncovered gaps = `idle`
4. Divide run duration into N equal buckets (default 20)
5. Compute per-bucket ms overlap for each type; clamp idle to ≥ 0
6. Return `{ buckets, runDurationMs }`

**File**: `frontend/src/components/runs/buildTimeSeries.ts` (new)

---

## T3 — Implement `ActivityTimeSeries.tsx`

Create `frontend/src/components/runs/ActivityTimeSeries.tsx`. This component:
- Accepts `events: RawRunEvent[]` and `isRunning: boolean`
- Calls `buildTimeSeries(events, 20)`; renders fallback text if result is `null`
- Uses `ResizeObserver` (via a `useRef` on the container div) to get chart width
- Renders an SVG stacked bar chart (height 72px fixed):
  - N vertical bars, each stacked with llm/tool/subagent/idle segments
  - Colors: llm = violet hsl(270 60% 55%), tool = amber hsl(35 70% 55%), subagent = steel hsl(210 60% 55%), idle = gray hsl(220 10% 35%)
  - X-axis: 3–5 evenly spaced time labels (format `Xs` or `Xm Ys`)
  - Legend row: `● LLM ● Tool ● Subagent ● Idle` in 10px muted text
- Hover tooltip: shows bucket time range and per-type ms breakdown

**File**: `frontend/src/components/runs/ActivityTimeSeries.tsx` (new)

---

## T4 — Lift `useRunTelemetry` in `RunCard` and wire `ActivityTimeSeries`

Refactor `frontend/src/components/layout/RunCard.tsx` to:
1. Call `useRunTelemetry(run.id, isActive)` at the `RunCard` level (currently called inside `RunActivityFeed`)
2. Pass `telemetry?.events ?? []` and `telemetry?.prompt` as props down to the children
3. Render `<ActivityTimeSeries events={...} isRunning={isActive} />` above `<RunActivityFeed .../>`

**File**: `frontend/src/components/layout/RunCard.tsx`

---

## T5 — Accept `events` prop in `RunActivityFeed`

Update `frontend/src/components/runs/RunActivityFeed.tsx` to accept an optional `events?: RawRunEvent[]` and `prompt?: string | null` prop. When provided, skip the internal `useRunTelemetry` call and use the passed data directly. This avoids a duplicate HTTP request after the lift in T4.

Maintain backward compatibility: if `events` prop is absent, fall back to the existing `useRunTelemetry(runId)` behavior.

**File**: `frontend/src/components/runs/RunActivityFeed.tsx`

---

## T6 — Manual smoke test

Verify the chart renders correctly in the UI:
- Open a completed run in the sidebar (expanded view)
- Confirm the chart appears above the event feed with colored bars
- Confirm hovering shows a tooltip with time breakdown
- Confirm the fallback text appears for runs with no `ts` fields
- Confirm a live/running run updates the chart as events arrive

**No code changes** — validation step only.
