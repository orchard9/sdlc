# Problem Analysis

## The Gap
`RunCard.tsx:164` has a hard conditional fork:
- **Active runs** → `AgentLog` (raw monospace log via `AgentEventLine`)
- **Completed runs** → `CompletedRunPanel` (rich `RunActivityFeed` + `ActivityTimeSeries`)

The SSE stream already delivers full `RawRunEvent` data (timestamps, tool_use_ids, subagent fields) during execution, but:
1. Events are typed as `AgentEvent` (subset type) — drops useful fields
2. Events are rendered by `AgentEventLine` — a primitive text renderer
3. The rich `RunActivityFeed` with paired event cards is only shown after completion

## Root Cause
**Not a data problem — purely a rendering problem.** The backend already streams full structured events during execution via `message_to_event()`. The telemetry store (`append_raw`) persists them in real-time. The frontend just chooses the wrong renderer.

## Evidence
- `AgentEvent` (types.ts:371) — 11 event types, no timestamps, no subagent fields
- `RawRunEvent` (types.ts:610) — 14 event types, timestamps, tool_use_ids, subagent fields
- Both come from the same `message_to_event()` function in `runs.rs:1346`
- `RunActivityFeed` already accepts `events?: RawRunEvent[]` prop and has `isRunning` mode