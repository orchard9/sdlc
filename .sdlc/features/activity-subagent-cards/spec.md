# Spec: Subagent Input/Output Cards in Activity Feed

## Problem

When Claude spawns subagents during a run, the backend already emits `subagent_started`, `subagent_progress`, and `subagent_completed` events with rich data (task_id, description, tool progress, tokens, duration, summary). The `buildTimeSeries.ts` uses these for the stacked time-series chart, but `pairEvents.ts` skips them entirely (they fall through to the `default` case). Users see a gap in the activity feed where subagent work happened — no cards, no indication of delegation or results.

## Solution

Add a `PairedSubagentExchange` type and a `SubagentCard` component that renders subagent lifecycle events as collapsible cards in the activity feed, consistent with the existing `ToolCallCard` pattern.

## Requirements

1. **New paired event type** (`PairedSubagentExchange`): Groups a `subagent_started` event with its matching `subagent_progress` and `subagent_completed` events by `task_id`. Fields: `kind: 'subagent_exchange'`, `taskId`, `description`, `status`, `summary`, `lastToolName`, `totalTokens`, `durationMs`.

2. **pairEvents update**: Handle `subagent_started`, `subagent_progress`, and `subagent_completed` in the event loop. Track open subagent exchanges by `task_id`. On `subagent_started`, create a pending exchange. On `subagent_progress`, update the pending exchange with latest tool name and token/duration data. On `subagent_completed`, finalize the exchange with status and summary, then emit it as a `PairedSubagentExchange`.

3. **SubagentCard component**: A new component in `frontend/src/components/runs/SubagentCard.tsx` that renders the paired subagent exchange. Visual design:
   - Left border color: steel blue (`border-blue-400`) to match the time series subagent color (`hsl(210 55% 55%)`)
   - Header: Bot icon + description text + status badge (completed/running/failed)
   - Collapsible detail section showing: last tool name, total tokens, duration
   - Summary text below (from `subagent_completed`)
   - While in-progress (no completed event yet): show a spinner

4. **PairedEventRow update**: Add a `case 'subagent_exchange'` that renders `<SubagentCard />`.

5. **PairedEvent union update**: Add `PairedSubagentExchange` to the `PairedEvent` type union.

## Out of Scope

- Nested activity feed within subagent cards (showing the subagent's own tool calls) — future enhancement
- Subagent event filtering/search
- Changes to `buildTimeSeries.ts` or `ActivityTimeSeries.tsx` (already working)

## Acceptance Criteria

- Subagent events appear as cards in the activity feed between the tool/text events that bracket them
- Cards show description, status, summary, duration, and token count
- Cards are collapsible (detail section) matching the ToolCallCard interaction pattern
- In-progress subagents show a spinner; completed ones show a status badge
- No regressions in existing activity feed rendering (init, tool, text, result cards)
