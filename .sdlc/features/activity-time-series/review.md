# Review: activity-time-series

## Summary

All five implementation tasks are complete. TypeScript compiles with zero errors (`npx tsc --noEmit` clean). The implementation is frontend-only as specified.

## Files changed

| File | Change |
|---|---|
| `frontend/src/lib/types.ts` | Extended `RawRunEvent` with subagent event types (`subagent_started`, `subagent_completed`, `subagent_progress`), `tool_use_id`, `tool_use_ids`, `task_id`, `description`, `last_tool_name`, `total_tokens`, `duration_ms`, and `ts?: string` |
| `frontend/src/components/runs/buildTimeSeries.ts` | New — pure `buildTimeSeries()` function with `BucketData` and `TimeSeriesData` types |
| `frontend/src/components/runs/ActivityTimeSeries.tsx` | New — SVG stacked bar chart component with ResizeObserver, legend, and tooltip |
| `frontend/src/components/layout/RunCard.tsx` | Added `CompletedRunPanel` that owns `useRunTelemetry` and renders both `ActivityTimeSeries` and `RunActivityFeed` with shared telemetry data |
| `frontend/src/components/runs/RunActivityFeed.tsx` | Added optional `events` and `prompt` props for data sharing; backward compatible (falls back to internal `useRunTelemetry` when not provided) |

## Findings

### Finding 1 — `RunActivityFeed` `useRunTelemetry('')` when `skipFetch` is true

When `events` prop is provided, `useRunTelemetry` is called with `runId = ''`. The hook checks `if (!runId) return` early, so no HTTP request fires. This is correct behavior but slightly implicit. **Accepted** — the pattern is clear from context and adding a `null` parameter type would require more invasive changes to the hook signature.

### Finding 2 — `CompletedRunPanel` always fetches (even for stopped/failed runs)

`CompletedRunPanel` calls `useRunTelemetry(runId, false)` whenever expanded. For runs with no event sidecar (very old runs), this will 404 and `events` will be `[]`, which correctly triggers the "no activity recorded" message and the "Time breakdown not available" fallback in the chart. **No action needed** — graceful degradation is correct.

### Finding 3 — Bar heights normalized to `maxBucketTotal`, not `bucketWidthMs`

The chart normalizes bar heights to the maximum across all buckets rather than to the absolute bucket width (ms). This means bars visually fill the chart height even for very short runs. This is intentional — it maximizes readability across runs of all durations. **Accepted by design.**

### Finding 4 — Tooltip position clamp uses `chartWidth - 120` hardcoded

The tooltip clamp (`Math.min(tooltip.x, chartWidth - 120)`) uses a hardcoded pixel offset. This could clip the tooltip on very narrow containers (< 120px). **Accepted** — the sidebar run panel is always wider than 120px in practice, and the tooltip is dismissible by mouse leave.

### Finding 5 — `subagent` intervals not subtracted from llm/tool overlap

Per the design, subagent intervals can overlap with llm/tool intervals. The bucket fill currently adds subagent ms independently and then computes idle as `bucketWidth - llm - tool - subagent`. If subagent completely overlaps a llm interval, the idle could go negative (clamped to 0), and the visual stack height for that bucket will be shorter than others. This is correct behavior — the idle clamp prevents negative display values. **Accepted** — documented in the design as v1 behavior.

### Finding 6 — `pairEvents.ts` uses `tool_use_id` on `RawRunEvent` fields

After the `RawRunEvent` type extension in `types.ts`, `pairEvents.ts` correctly reads `tool_use_id` and `tool_use_ids` fields that are now typed. Previously these were read as dynamic property accesses (would have been `any`). **Improvement** — the type extension makes this safer.

## Quality checks

- `npx tsc --noEmit`: **clean** (0 errors)
- No `console.log` or debugging artifacts left in new files
- No `unwrap()` or unsafe operations (frontend, no Rust)
- All new components use existing Tailwind utility classes; no new CSS files
- No third-party chart library added; SVG primitives only
- `ResizeObserver` cleanup (`ro.disconnect()`) is correctly performed in the `useEffect` return

## Verdict

**Approved.** Implementation matches spec and design. All findings are accepted or informational. TypeScript is clean. The fallback behavior for legacy runs (no `ts` fields) is correctly handled.
