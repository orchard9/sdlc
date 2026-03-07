# Implementation Plan

## Approach: Frontend-Only Fix (No Backend Changes)

The data is already streaming. We just need to render it with the right component.

### Changes

**1. RunCard.tsx — Swap renderer for active runs**
- Change `liveEvents` type from `AgentEvent[]` to `RawRunEvent[]`
- Cast SSE parsed data as `RawRunEvent` (line 87)
- Replace `<AgentLog>` with `<RunActivityFeed>` for active runs (line 164-165)
- Pass liveEvents directly: `<RunActivityFeed runId={run.id} isRunning={true} events={liveEvents} />`
- Optionally include `<ActivityTimeSeries events={liveEvents} isRunning={true} />` above the feed

**2. RunActivityFeed.tsx — Handle spawning state**
- When `isRunning && events.length === 0`, show 'Spawning agent...' spinner instead of 'No activity recorded yet'

**3. RunActivityFeed.tsx — Add auto-scroll**
- Add ref + useEffect to scroll to bottom when new events arrive (same pattern as AgentLog)

**4. Delete dead code**
- Remove `AgentLog.tsx` and `AgentEventLine.tsx` (only used by RunCard's old active-run path)

### Non-Changes
- No backend modifications needed
- No new SSE event types
- No changes to `pairEvents.ts` (it already handles partial event sequences)
- `useRunTelemetry` hook stays as-is (only used by CompletedRunPanel)

### Risk Assessment
- **pairEvents() performance**: Called on every render via useMemo. With 200+ events, rebuilding paired events from scratch is O(n) which is fine. React's useMemo dependency on the events array reference means it only recalculates when new events arrive.
- **ActivityTimeSeries with partial data**: May look sparse during early run. Could defer showing it until N events are accumulated, or show it from the start. Low risk either way.

## Decided
- Frontend-only change, no backend work
- Use RunActivityFeed for both active and completed runs  
- Delete AgentLog/AgentEventLine as dead code
- Add auto-scroll to RunActivityFeed for isRunning mode

## Open
- Should ActivityTimeSeries show during active runs? (probably yes, it animates well with partial data)
- Should we unify AgentEvent and RawRunEvent types? (probably yes, AgentEvent becomes dead type)