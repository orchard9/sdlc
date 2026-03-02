# Spec: activity-time-series

## Problem

The run activity feed (`RunActivityFeed`) shows a chronological list of events but provides no visual summary of *how* a run spent its time. You cannot answer questions like:

- What fraction of the run was waiting on LLM responses vs. executing tools?
- Were tool calls clustered at the start or evenly distributed?
- Did subagent work overlap with parent agent work?

The existing `GET /api/runs/:id/telemetry/summary` endpoint returns aggregate counts (tool_calls, subagents_spawned, cost_usd) but no temporal breakdown.

## Prerequisite

This feature depends on `telemetry-wallclock-timestamps`, which adds a `ts` (ISO-8601 wall-clock timestamp) field to every event stored in the telemetry sidecar. Without per-event timestamps, interval computation is not possible. The time-series chart must be gated on timestamp availability — if no `ts` fields are present, it falls back gracefully (hides the chart or shows a "timestamps not available" placeholder).

## Solution

Add a stacked time-series bar chart to the run detail view that shows agent activity broken down by wait type over the run duration.

### Wait-type breakdown

Each interval between consecutive timestamped events is classified into one of four wait types:

| Wait type | Definition |
|---|---|
| `llm` | Interval between an `assistant` event and the next `user` event (LLM inference time) |
| `tool` | Interval between a `user` event and the next `assistant` event (tool execution) |
| `subagent` | Interval between `subagent_started` and `subagent_completed` for a given `task_id` |
| `idle` | Any interval not covered by the above (init, result, error, gaps) |

Subagent intervals are tracked independently and may overlap with `llm` or `tool` intervals from the parent agent.

### Chart design

- **Chart type**: Stacked horizontal bar chart (or stacked area chart rendered as bars for discrete time buckets)
- **X axis**: Wall-clock time (seconds from run start, 0 to total duration)
- **Y axis / stacking**: Wait types — `llm`, `tool`, `subagent`, `idle`
- **Color coding**:
  - `llm` → purple / violet (matches the model/AI association)
  - `tool` → amber / orange (active computation)
  - `subagent` → blue (delegation)
  - `idle` → gray / muted (overhead)
- **Buckets**: The time range is divided into N fixed-width buckets (N=20 default). Each bucket shows the proportion of time in that window attributed to each wait type.
- **Tooltip**: Hovering a bucket shows the bucket time range, per-type seconds, and dominant type.
- **No chart library dependency**: Implement using SVG primitives in React — no third-party chart library. The chart dimensions are responsive via ResizeObserver. The design should match the existing muted/dark UI aesthetic.

### Data derivation (frontend)

A new pure function `buildTimeSeries(events: RawRunEvent[], buckets?: number): TimeSeriesData` in `frontend/src/components/runs/buildTimeSeries.ts` computes the chart data:

1. Extract all events with a `ts` field; return empty if fewer than two timestamps found.
2. Compute `runStart` (first `ts`) and `runEnd` (last `ts`).
3. Derive intervals by iterating event pairs:
   - `assistant` → next `user`: classify as `llm`
   - `user` → next `assistant`: classify as `tool`
   - `subagent_started` → matching `subagent_completed` (by `task_id`): classify as `subagent`
   - Remaining gaps: classify as `idle`
4. Divide `[runStart, runEnd]` into N equal-width buckets.
5. For each bucket, sum the overlap of each wait-type interval with the bucket window.
6. Return `{ buckets: BucketData[], runDurationMs: number }`.

### Types

```ts
export interface BucketData {
  startMs: number   // ms from run start
  endMs: number
  llm: number       // ms in this bucket spent as llm wait
  tool: number      // ms in this bucket spent as tool wait
  subagent: number  // ms in this bucket spent as subagent wait
  idle: number      // ms in this bucket as idle
}

export interface TimeSeriesData {
  buckets: BucketData[]
  runDurationMs: number
}
```

### New types in `RawRunEvent`

`telemetry-wallclock-timestamps` adds `ts?: string` (ISO-8601) to every `RawRunEvent`. The time-series feature reads this field but does not add it — it comes from that prerequisite feature.

### Component structure

- `frontend/src/components/runs/ActivityTimeSeries.tsx` — chart component; accepts `events: RawRunEvent[]` and `isRunning: boolean`; renders SVG chart or fallback
- `frontend/src/components/runs/buildTimeSeries.ts` — pure function to compute `TimeSeriesData` from events
- Integration: render `<ActivityTimeSeries>` above `RunActivityFeed` in the run detail panel

### Fallback behavior

If `events` contains no `ts` fields (legacy runs or events predating the timestamp feature):
- Render a muted placeholder: *"Time breakdown not available for this run"*
- The `RunActivityFeed` event list still renders below

### Live updates

While `isRunning` is true, the chart re-renders on each telemetry poll (same cadence as the activity feed, currently polling via `useRunTelemetry`). No separate polling mechanism is needed.

## Scope

- New files: `ActivityTimeSeries.tsx`, `buildTimeSeries.ts` in `frontend/src/components/runs/`
- Update `RunActivityFeed.tsx` or its parent to include the chart above the event list
- Update `RawRunEvent` in `frontend/src/lib/types.ts` to add `ts?: string` if not already added by `telemetry-wallclock-timestamps`
- No backend changes

## Out of Scope

- Per-tool breakdown within the `tool` category (future: concurrency-heatmap feature)
- Subagent nesting depth visualization
- Export/download of chart data
- Zoom or pan on the time axis
