# Design: activity-time-series

## Overview

A stacked bar chart rendered in SVG above the `RunActivityFeed` in each `RunCard`. The chart shows how a run spent time across four wait categories: `llm` (LLM inference), `tool` (tool execution), `subagent` (delegated work), and `idle` (gaps/overhead). All computation is frontend-only — no backend changes.

## Architecture

```
RunCard (layout/RunCard.tsx)
  └── expanded panel
        ├── ActivityTimeSeries (runs/ActivityTimeSeries.tsx)   ← NEW
        │     uses: buildTimeSeries (runs/buildTimeSeries.ts)  ← NEW
        │     data: events from useRunTelemetry (shared)
        └── RunActivityFeed (runs/RunActivityFeed.tsx)
              uses: useRunTelemetry (hooks/useRunTelemetry.ts)
              uses: pairEvents (runs/pairEvents.ts)
```

`ActivityTimeSeries` receives `events: RawRunEvent[]` directly (not a run ID) so it can reuse the already-fetched telemetry that `RunActivityFeed` also consumes. The parent (`RunCard` or a thin wrapper) passes both down after a single `useRunTelemetry` call.

## Data flow

```
useRunTelemetry(runId)
  → telemetry.events: RawRunEvent[]
  → buildTimeSeries(events, 20)
      → TimeSeriesData { buckets[], runDurationMs }
  → SVG chart via ActivityTimeSeries
```

## Integration changes

`RunActivityFeed` currently calls `useRunTelemetry` internally. To share the data with `ActivityTimeSeries` without a second fetch, lift `useRunTelemetry` up to `RunCard` and pass `events` as a prop to both components. This is a minor refactor of two files.

Alternatively — to minimize diff — `ActivityTimeSeries` can call `useRunTelemetry` independently. Since the hook has no side effects beyond a fetch, two calls with the same `runId` both hit the cache layer of the browser (the GET is idempotent). The simpler approach is accepted for this feature.

**Decision**: `ActivityTimeSeries` receives `events: RawRunEvent[] | undefined` and `isRunning: boolean`. The parent passes telemetry events directly (lifted from `RunActivityFeed`) to avoid a second HTTP call. `RunCard` is updated to lift the hook.

## `buildTimeSeries.ts` — algorithm

```
Input: events: RawRunEvent[], bucketCount = 20
Output: TimeSeriesData | null  (null = no timestamps available)

1. Filter events to those with ts field → timestamped[]
2. If < 2 timestamped events → return null
3. runStartMs = Date.parse(timestamped[0].ts)
   runEndMs   = Date.parse(timestamped[last].ts)
   runDurationMs = runEndMs - runStartMs
4. Derive intervals:
   - Scan timestamped events in order:
     * assistant(t0) → next user(t1): push {type:'llm', start:t0, end:t1}
     * user(t0) → next assistant(t1): push {type:'tool', start:t0, end:t1}
     * subagent_started(t0, task_id) → subagent_completed(t1, same task_id):
         push {type:'subagent', start:t0, end:t1}
     * Uncovered spans → {type:'idle'}
5. Bucket fill:
   bucketWidthMs = runDurationMs / bucketCount
   For each bucket i in [0, bucketCount):
     bucketStart = runStartMs + i * bucketWidthMs
     bucketEnd   = bucketStart + bucketWidthMs
     For each interval:
       overlap = max(0, min(interval.end, bucketEnd) - max(interval.start, bucketStart))
       bucket[i][interval.type] += overlap
     idle[i] = bucketWidthMs - sum(llm+tool+subagent) for that bucket
     clamp idle to ≥ 0
6. Return { buckets, runDurationMs }
```

Note: subagent intervals can overlap with llm/tool intervals. The overlap is assigned to `subagent` first; the remaining span is then checked against llm/tool. This means in a bucket where subagent and llm overlap, the bucket shows both — the total displayed height can exceed `bucketWidthMs` in such buckets. For simplicity in v1, cap each type at bucketWidthMs and normalize when stacking.

## SVG chart design

```
Height: 72px fixed
Width: 100% (ResizeObserver via useRef + getBoundingClientRect)
Padding: 8px left, 8px right, 20px bottom (for x-axis labels), 4px top

+------------------------------------------------------------------+
| [bar][bar][bar][bar]...(20 bars total)                           |
+------------------------------------------------------------------+
  0s                             15s                          30s  ← x-axis ticks (3–5 labels)
```

Each bar is a vertical stack of 4 colored segments proportional to their ms values in the bucket. Bar width = (chartWidth - padding) / bucketCount - 1px gap.

Segment colors (CSS custom properties for dark-mode compatibility):
- `llm`: hsl(270 60% 55%) — muted violet
- `tool`: hsl(35 70% 55%) — amber
- `subagent`: hsl(210 60% 55%) — steel blue
- `idle`: hsl(220 10% 35%) — muted gray

### Tooltip

On `mouseenter` of a bar group, show a floating `<div>` (absolute positioned) with:
```
0.8s – 1.2s
LLM    380ms
Tool   120ms
Idle   100ms
```

Implemented as React state `hoveredBucket: { index, x, y } | null`. Tooltip dismisses on `mouseleave`.

### X-axis labels

3–5 evenly-spaced time labels in `text-[10px] text-muted-foreground`. Format: `Xs` for < 60s, `Xm Ys` for longer runs.

### Legend

A single-row legend below the chart (or inline with the chart title):
```
● LLM  ● Tool  ● Subagent  ● Idle
```
Implemented as colored dots + text in `text-[10px]`.

### Fallback

When `buildTimeSeries` returns null:
```tsx
<p className="text-[11px] text-muted-foreground/50 italic py-2">
  Time breakdown not available (run predates timestamps)
</p>
```

## Component API

```tsx
// ActivityTimeSeries.tsx
interface ActivityTimeSeriesProps {
  events: RawRunEvent[]
  isRunning: boolean
}
export function ActivityTimeSeries({ events, isRunning }: ActivityTimeSeriesProps)

// buildTimeSeries.ts
export function buildTimeSeries(
  events: RawRunEvent[],
  bucketCount?: number   // default: 20
): TimeSeriesData | null

export interface BucketData {
  startMs: number
  endMs: number
  llm: number
  tool: number
  subagent: number
  idle: number
}
export interface TimeSeriesData {
  buckets: BucketData[]
  runDurationMs: number
}
```

## `RunCard` refactor

`RunCard` currently renders:
```tsx
{expanded && !isPonder && (
  <div className="px-3 pb-3">
    <RunActivityFeed runId={run.id} isRunning={isActive} />
  </div>
)}
```

After this feature, it becomes:
```tsx
{expanded && !isPonder && (
  <RunDetailPanel runId={run.id} isRunning={isActive} />
)}
```

Where `RunDetailPanel` (inline or extracted) calls `useRunTelemetry` once and renders:
```tsx
<div className="px-3 pb-3 space-y-3">
  <ActivityTimeSeries events={telemetry?.events ?? []} isRunning={isActive} />
  <RunActivityFeed events={telemetry?.events ?? []} isRunning={isActive} prompt={telemetry?.prompt} />
</div>
```

This requires a small API change to `RunActivityFeed` — it should accept `events` as a prop instead of (or alongside) `runId`. The simplest migration: add optional `events` prop; if provided, skip the internal `useRunTelemetry` call and use the passed data directly.

## Files changed

| File | Change |
|---|---|
| `frontend/src/components/runs/buildTimeSeries.ts` | New — pure time-series computation |
| `frontend/src/components/runs/ActivityTimeSeries.tsx` | New — SVG chart component |
| `frontend/src/components/layout/RunCard.tsx` | Lift `useRunTelemetry`, add `ActivityTimeSeries` |
| `frontend/src/components/runs/RunActivityFeed.tsx` | Accept optional `events` prop to avoid double-fetch |
| `frontend/src/lib/types.ts` | Add `ts?: string` to `RawRunEvent` (if not already added by `telemetry-wallclock-timestamps`) |

## ASCII wireframe

```
┌─────────────────────────────────────────────────────────────┐
│ ● running   feature:my-slug                    [■ stop] [▼] │
├─────────────────────────────────────────────────────────────┤
│  ● LLM  ● Tool  ● Subagent  ● Idle     Activity breakdown   │
│                                                             │
│  ██                                                         │
│  ██ ▓▓ ▓▓                                                   │
│  ██ ▓▓ ▓▓ ░░ ░░ ░░ ░░                                       │
│  ██ ▓▓ ▓▓ ░░ ░░ ░░ ░░ ██ ██ ██ ▒▒ ▒▒                       │
│  0s          5s          10s         15s                    │
│  ─────────────────────────────────────────────────────────  │
│  [init event card]                                          │
│  [tool call: Bash]                                          │
│  [assistant text]                                           │
│  ...                                                        │
└─────────────────────────────────────────────────────────────┘
```

Legend: `██` = LLM (violet), `▓▓` = Tool (amber), `░░` = Idle (gray), `▒▒` = Subagent (blue)

## Edge cases

| Scenario | Behavior |
|---|---|
| Single event (no duration) | `buildTimeSeries` returns null → fallback text |
| Very short run (< 1s) | Bucket width < 50ms — still renders, labels show ms |
| Subagent with no matching `completed` event (ongoing) | Interval end = last `ts` seen; marked as open (isRunning = true redraws) |
| All time in one type (e.g. pure LLM) | Single solid bar, other segments at 0 |
| Run with no tools (pure reasoning) | All `llm` and `idle`, no `tool` segments |
