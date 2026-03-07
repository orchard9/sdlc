# Tasks — Activity Dead Code Cleanup

## T1: Replace AgentLog with RunActivityFeed in RunCard active-run path
- **File**: `frontend/src/components/layout/RunCard.tsx`
- Replace the `<AgentLog running={isActive} events={liveEvents} />` (line 165) with `<RunActivityFeed>` using the live SSE events
- The SSE currently streams `AgentEvent` format; `RunActivityFeed` expects `RawRunEvent[]` — add a lightweight adapter that maps AgentEvent → RawRunEvent in RunCard, or stream RawRunEvents directly from the SSE
- Remove `AgentLog` import, `AgentEvent` import, `liveEvents` state, and the SSE `AgentEvent` parsing logic if no longer needed
- Ensure the "Spawning agent..." loading state is preserved (RunActivityFeed should handle empty events with `isRunning=true`)

## T2: Delete AgentLog.tsx and AgentEventLine.tsx
- **Files**: `frontend/src/components/shared/AgentLog.tsx`, `frontend/src/components/shared/AgentEventLine.tsx`
- Delete both files — they have no remaining consumers after T1

## T3: Remove AgentEvent type if unused
- **File**: `frontend/src/lib/types.ts`
- Check if `AgentEvent` interface (line 371) is still referenced anywhere after T1
- If only referenced in `client.ts` getRun response typing, evaluate whether that API still returns AgentEvent format or has migrated to RawRunEvent — clean up accordingly

## T4: Verify build and no regressions
- Run `cd frontend && npm run build` to confirm no TypeScript errors or broken imports
- Visually confirm active runs still render correctly (if dev server available)
