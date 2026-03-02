# Spec: run-activity-ui

## Problem

There is no UI to inspect what happened during an agent run. The run list shows status (running/completed/failed), cost, and turn count. That's it. You cannot see:

- What the agent was asked to do (initial prompt)
- Which tools it called and what they returned
- Whether any tool calls failed
- Which subagents were spawned, what they were asked, and what they returned
- The agent's reasoning (thinking blocks)

## Solution

Add a Run Activity Feed: a chronological event timeline rendered in the run detail view. The feed fetches from `GET /api/runs/:id/telemetry` and renders each event type with appropriate formatting.

### Event rendering

| Event type | Rendering |
|---|---|
| `init` | Header card: model name, MCP servers, initial prompt text (from `RunRecord.prompt`) |
| `tool_call` | Card with tool name badge, collapsible JSON input |
| `tool_result` | Paired with the matching `tool_call` card (matched by `tool_use_id`) — shows result content, error badge if `is_error: true` |
| `subagent_started` | Indented card with description text and task_id |
| `subagent_progress` | Inline token/duration update under the subagent card |
| `subagent_completed` | Closes the subagent card with status badge, summary text, token+duration stats |
| `assistant_text` | Prose block with the assistant's text |
| `thinking` | Dimmed/italic block, collapsible (collapsed by default) |
| `run_result` | Footer card: total cost, total turns, duration |

### Live updates

During an active run, the feed updates as events arrive. Implementation: re-fetch from `/api/runs/:id/telemetry` on each `RunFinished` SSE event matching the current run's id. Alternatively, poll at a low interval (2s) while `run.status === 'running'`.

### Component structure

- `RunActivityFeed.tsx` — top-level component, fetches and renders event list
- `ToolCallCard.tsx` — renders a `tool_call` + `tool_result` pair
- `SubagentCard.tsx` — renders `subagent_started` + optional progress + `subagent_completed`
- `AssistantTextBlock.tsx` — renders `assistant_text`
- `ThinkingBlock.tsx` — renders `thinking` (collapsible)
- `RunInitCard.tsx` — renders `init` event with prompt
- `RunResultCard.tsx` — renders `run_result` summary footer

### Integration point

The feed is added to the run detail view. Placement: new "Activity" tab alongside any existing run detail tabs, or as the primary content if no existing detail view exists.

## Scope

- New components in `frontend/src/components/runs/`
- New API hook for `GET /api/runs/:id/telemetry`
- Wired into existing run detail route/view
- No backend changes (all backend work in run-event-capture and run-events-api)

## Out of Scope

- Diff/compare between two runs
- Filtering/searching within a run's event feed
- Export to JSON/markdown
