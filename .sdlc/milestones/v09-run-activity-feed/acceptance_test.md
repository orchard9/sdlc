# Acceptance Test: v09-run-activity-feed

## Scenario: Full run timeline is readable and complete

1. Start an agent run (any feature, any ponder run)
2. Wait for the run to complete
3. Open the run in the UI
4. Verify the timeline contains:
   - [ ] Initial prompt — the exact text passed to the agent
   - [ ] All tool calls with their names and inputs visible
   - [ ] All tool results paired with their tool calls (same card or adjacent)
   - [ ] Any subagent lifecycle events: started (with description), progress updates, completed (with summary)
   - [ ] Cost and turn count in the summary footer

## Scenario: Tool errors are visible

1. Trigger a run that produces a tool error
2. Open the run in the UI
3. Verify the failed tool call shows an error badge and the error content

## Scenario: Multi-subagent run shows full delegation tree

1. Trigger a run that spawns at least one subagent
2. Open the run in the UI
3. Verify each subagent appears as a card with its description
4. Verify each subagent's completed card shows summary and token count

## Scenario: Past runs are still accessible after server restart

1. Complete a run
2. Restart sdlc-server
3. Open the same run in the UI
4. Verify all events are still present (persisted in redb)
