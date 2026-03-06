# QA Plan: Subagent Input/Output Cards in Activity Feed

## Test Strategy

This is a frontend-only feature. Testing focuses on the pairing logic (unit-testable) and visual rendering (manual/visual inspection).

## Test Cases

### 1. pairEvents grouping

- **Input**: Event array with `subagent_started` (task_id=A), `subagent_progress` (task_id=A), `subagent_completed` (task_id=A)
- **Expected**: Output contains one `PairedSubagentExchange` with `isComplete: true`, correct description/summary/tokens/duration

### 2. Multiple concurrent subagents

- **Input**: Interleaved events for task_id=A and task_id=B
- **Expected**: Two separate `PairedSubagentExchange` entries, each with correct fields

### 3. In-progress subagent (no completed event)

- **Input**: `subagent_started` + `subagent_progress` with no `subagent_completed`
- **Expected**: One `PairedSubagentExchange` with `isComplete: false`

### 4. Subagent with no progress events

- **Input**: `subagent_started` then immediately `subagent_completed`
- **Expected**: One `PairedSubagentExchange` with `isComplete: true`, no lastToolName

### 5. Existing event types unaffected

- **Input**: Standard event stream (init, assistant, tool_progress, tool_summary, result) with no subagent events
- **Expected**: Output identical to current behavior — no regressions

### 6. Visual: SubagentCard renders correctly

- Completed subagent shows green status badge, description, summary
- In-progress subagent shows amber spinner
- Failed subagent shows red badge
- Details section toggles on click
- Card uses steel-blue left border

## Build Verification

- `cd frontend && npm run build` succeeds with no TypeScript errors
- No console warnings related to subagent card rendering
