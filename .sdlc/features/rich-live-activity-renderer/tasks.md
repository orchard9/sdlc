# Tasks: Rich Live Activity Renderer

## T1: Swap renderer in RunCard.tsx

- Change `liveEvents` state type from `AgentEvent[]` to `RawRunEvent[]`
- Cast SSE parsed data as `RawRunEvent` instead of `AgentEvent`
- Replace `<AgentLog running={isActive} events={liveEvents} />` with `<RunActivityFeed runId={run.id} isRunning={true} events={liveEvents} />`
- Add `<ActivityTimeSeries events={liveEvents} isRunning={true} />` above the feed for active runs
- Remove `AgentLog` import; remove `AgentEvent` from type imports

## T2: Add spawning state to RunActivityFeed.tsx

- When `isRunning && pairedEvents.length === 0`, show spinner with "Spawning agent..." instead of "No activity recorded yet"

## T3: Add auto-scroll to RunActivityFeed.tsx

- Add `useRef` on the feed container div
- Add `useEffect` that scrolls to bottom when `pairedEvents` changes and `isRunning` is true
- Wrap the feed content in a scrollable container (`max-h-80 overflow-y-auto`)

## T4: Verify build compiles

- Run `cd frontend && npm run build` to confirm no type errors or build failures
