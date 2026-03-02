# Design: run-activity-ui

## Component hierarchy

```
RunActivityFeed
  RunInitCard          (init event)
  ToolCallCard         (tool_call + tool_result pair)
    ToolCallHeader     (name badge, input preview)
    ToolResultBody     (content, error badge)
  AssistantTextBlock   (assistant_text event)
  ThinkingBlock        (thinking event, collapsed by default)
  SubagentCard         (subagent_started + progress updates + subagent_completed)
    SubagentHeader     (description, status badge)
    SubagentProgress   (token count, last tool)
    SubagentResult     (summary, tokens, duration)
  RunResultCard        (run_result event — footer)
```

## Data flow

1. `RunActivityFeed` receives `runId: string` as a prop
2. Fetches `GET /api/runs/:id/telemetry` on mount
3. Processes the flat event array into a structured list:
   - Pair each `tool_call` with its `tool_result` by matching `tool_use_id`
   - Nest `subagent_progress` and `subagent_completed` under their `subagent_started` by `task_id`
4. Renders paired/nested structures rather than the raw flat array
5. While `run.status === 'running'`, re-fetch every 2 seconds (or on `RunFinished` SSE event)

## Pairing logic

```typescript
type PairedEvent =
  | { kind: 'init'; event: InitEvent }
  | { kind: 'tool_exchange'; call: ToolCallEvent; result?: ToolResultEvent }
  | { kind: 'assistant_text'; event: AssistantTextEvent }
  | { kind: 'thinking'; event: ThinkingEvent }
  | { kind: 'subagent'; started: SubagentStartedEvent; progress: SubagentProgressEvent[]; completed?: SubagentCompletedEvent }
  | { kind: 'run_result'; event: RunResultEvent }

function pairEvents(events: RawEvent[]): PairedEvent[]
```

Pass through events in order; accumulate tool_calls in a `Map<tool_use_id, ToolCallEvent>` until the matching tool_result arrives. Same for subagent started/progress/completed keyed by task_id.

## Visual design

Consistent with existing dark UI conventions. Each event type gets a left-border accent color:
- Tool exchanges: blue border
- Subagent cards: purple border (indented 1 level)
- Assistant text: no border
- Thinking: gray border, italic, collapsed by default
- Run init: no border, full-width header
- Run result: green border (success) / red (failed), sticky at bottom

Collapsible sections use a `<details>` element or a simple boolean state toggle — no animation library needed.

## Integration point

The existing run list UI (`frontend/src/pages/`) shows runs but has no detail view. Add a click handler on each run row that expands `RunActivityFeed` inline (accordion style) or navigates to a `/runs/:id` route. Prefer the accordion to avoid routing complexity.

## API hook

```typescript
function useRunTelemetry(runId: string) {
  // fetches GET /api/runs/:id/telemetry
  // returns { events: RawEvent[], isLoading, error }
}
```
