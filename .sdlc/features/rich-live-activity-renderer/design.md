# Design: Rich Live Activity Renderer

## Overview

This is a component-swap feature. No new UI components are created. The existing `RunActivityFeed` and `ActivityTimeSeries` components (already used for completed runs) are reused for active runs.

## Changes

### RunCard.tsx

```
Before:
  liveEvents: AgentEvent[]
  SSE data cast as AgentEvent
  Active → <AgentLog running={true} events={liveEvents} />
  Completed → <CompletedRunPanel />

After:
  liveEvents: RawRunEvent[]
  SSE data cast as RawRunEvent
  Active → <ActivityTimeSeries events={liveEvents} isRunning={true} />
           <RunActivityFeed runId={run.id} isRunning={true} events={liveEvents} />
  Completed → <CompletedRunPanel /> (unchanged)
```

The `AgentLog` import is removed. The `AgentEvent` type import is removed (only `RawRunEvent` needed).

### RunActivityFeed.tsx

Two additions:

1. **Spawning state**: When `isRunning && pairedEvents.length === 0`, render a spinner with "Spawning agent..." instead of the italic "No activity recorded yet" message.

2. **Auto-scroll**: Add a `useRef` + `useEffect` that scrolls the container to the bottom when `pairedEvents` changes and `isRunning` is true. Uses the same pattern as `AgentLog` (set `scrollTop = scrollHeight`). Requires wrapping the feed in a scrollable container with `max-h-80 overflow-y-auto`.

## Data Flow

```
SSE /api/run/:key/events
  → EventSource in RunCard
    → JSON.parse as RawRunEvent
      → append to liveEvents state
        → passed to RunActivityFeed as events prop
          → pairEvents() groups into PairedEvent[]
            → PairedEventRow renders tool_exchange / assistant_text / init / run_result cards
```

No new data flows. The SSE stream already emits the same event shape as `RawRunEvent` — `AgentEvent` and `RawRunEvent` have nearly identical fields. The cast change is a type correction, not a data transformation.
