# Spec: Rich Live Activity Renderer

## Summary

Replace `<AgentLog>` with `<RunActivityFeed>` for active (running) agent runs in `RunCard.tsx`, so live SSE events render with the same rich UI (tool call cards, assistant text blocks, init cards) used for completed runs.

## Problem

Active runs currently render via `<AgentLog>`, which shows a plain monospace log of raw event lines. Completed runs use `<RunActivityFeed>` with structured paired-event cards (tool exchanges, assistant text, result summaries). This creates a jarring UX gap: watching a live run gives a degraded view compared to reviewing it after completion.

## Solution

1. **Change `liveEvents` type** from `AgentEvent[]` to `RawRunEvent[]` in `RunCard.tsx`
2. **Cast SSE data** as `RawRunEvent` instead of `AgentEvent` (line 87)
3. **Replace `<AgentLog>`** with `<RunActivityFeed>` for active runs, passing `events={liveEvents}` and `isRunning={true}`
4. **Add `<ActivityTimeSeries>`** above the feed for active runs (partial data is fine)
5. **Handle spawning state** in `RunActivityFeed`: when `isRunning && events.length === 0`, show a "Spawning agent..." spinner instead of "No activity recorded yet"
6. **Add auto-scroll** to `RunActivityFeed` when `isRunning` — scroll to bottom as new events arrive (same UX as current `AgentLog`)

## Scope

- Frontend only — no backend changes
- Files changed: `RunCard.tsx`, `RunActivityFeed.tsx`
- `AgentEvent` type and `AgentLog`/`AgentEventLine` components become dead code (handled by sibling feature `activity-dead-code-cleanup`)

## Acceptance Criteria

- Expanding an active run shows rich paired-event cards (tool calls, assistant text) identical to the completed-run view
- Activity time series chart renders above the live feed
- When an active run has no events yet, a "Spawning agent..." spinner displays
- The feed auto-scrolls to the latest event as new SSE events arrive
- Completed/failed/stopped runs continue to render identically (no regression)
