# Code Review — Activity Dead Code Cleanup

## Summary

Removed `AgentLog.tsx`, `AgentEventLine.tsx`, and the `AgentEvent` interface — all dead code after the active-run rendering path was switched to `RunActivityFeed` with `RawRunEvent[]`.

## Changes

| File | Change |
|---|---|
| `frontend/src/components/layout/RunCard.tsx` | Removed `AgentLog` import, replaced `AgentEvent` with `RawRunEvent`, swapped `<AgentLog>` for `<RunActivityFeed>` + `<ActivityTimeSeries>` on active runs |
| `frontend/src/components/shared/AgentLog.tsx` | **Deleted** — no remaining consumers |
| `frontend/src/components/shared/AgentEventLine.tsx` | **Deleted** — no remaining consumers |
| `frontend/src/lib/types.ts` | Removed `AgentEvent` interface (lines 367-392) — fully superseded by `RawRunEvent` |
| `frontend/src/api/client.ts` | Updated `getRun` return type from `AgentEvent[]` to `RawRunEvent[]` |

## Findings

1. **No regressions** — `npm run build` succeeds cleanly with zero TypeScript errors.
2. **No dangling references** — grep for `AgentLog`, `AgentEventLine`, and `AgentEvent` across `frontend/src` returns zero results.
3. **Active run rendering preserved** — `RunActivityFeed` handles the `isRunning=true` case with a "Running..." spinner, and the SSE live event stream still feeds `RawRunEvent[]` into the feed.
4. **`getRun` API function is unused** — `api.getRun` (singular) in `client.ts` is never called. Left in place as it's a valid API surface, but updated its typing to `RawRunEvent[]` for consistency. Could be removed in a future cleanup pass if desired.

## Verdict

All changes are safe deletions and type narrowing. No behavioral changes. **Approved.**
