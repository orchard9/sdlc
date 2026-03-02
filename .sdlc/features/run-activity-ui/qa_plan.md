# QA Plan: run-activity-ui

## Component tests

1. `pairEvents` with interleaved tool_call/tool_result events — assert pairs are matched by `tool_use_id`; unmatched tool_call renders without result
2. `pairEvents` with subagent lifecycle events — assert `subagent_progress` events are nested under their `subagent_started` by `task_id`
3. `ToolCallCard` renders error badge when `is_error: true`
4. `ThinkingBlock` is collapsed by default; expands on click

## UAT scenarios (Playwright)

1. Open a completed run — assert activity feed renders with at least one tool call card
2. Run with a subagent — assert subagent card shows description and status badge
3. Run init card shows the prompt text
4. Run result card shows cost and turn count

## Live update check

- Start a run; open the activity feed before it completes; assert new events appear without page reload
