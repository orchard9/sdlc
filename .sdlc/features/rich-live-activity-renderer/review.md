# Review: Rich Live Activity Renderer

## Changes Summary

Two files modified:

### RunCard.tsx
- Replaced `AgentEvent` type with `RawRunEvent` for `liveEvents` state and SSE cast
- Replaced `<AgentLog>` with `<ActivityTimeSeries>` + `<RunActivityFeed>` for active runs
- Removed `AgentLog` import and `AgentEvent` type import
- Completed run path (`CompletedRunPanel`) unchanged

### RunActivityFeed.tsx
- Moved `useRef`/`useEffect` hooks above early returns to comply with Rules of Hooks
- Added spawning state: shows "Spawning agent..." spinner when `isRunning && pairedEvents.length === 0`
- Added auto-scroll: `useEffect` scrolls feed container to bottom when `pairedEvents` changes during active runs
- Added `max-h-80 overflow-y-auto` to feed container when `isRunning` for scroll containment

## Findings

1. **Rules of Hooks compliance** — Initially placed hooks after conditional returns; caught and fixed during implementation. Final code is correct.
2. **No regressions to completed runs** — `CompletedRunPanel` is untouched; `RunActivityFeed` behavior for `isRunning=false` is unchanged (no scroll ref used, no max-height applied).
3. **Type compatibility** — `RawRunEvent` and `AgentEvent` share the same `type` discriminator values relevant to SSE parsing. The cast is safe.
4. **Dead code created** — `AgentLog.tsx` and `AgentEventLine.tsx` are now unused imports only in themselves. Cleanup is tracked by sibling feature `activity-dead-code-cleanup`.

## Verdict

All findings addressed. Approved.
