# Review: Subagent Input/Output Cards in Activity Feed

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/types.ts` | Added `PairedSubagentExchange` interface, extended `PairedEvent` union |
| `frontend/src/components/runs/pairEvents.ts` | Added subagent event handling with task_id-based grouping |
| `frontend/src/components/runs/SubagentCard.tsx` | New component (103 lines) |
| `frontend/src/components/runs/RunActivityFeed.tsx` | Import + case for `subagent_exchange` |

## Findings

### 1. Type safety - PASS
The `PairedSubagentExchange` interface uses proper optional fields matching the backend event data. The `PairedEvent` union is exhaustive in the switch statement.

### 2. Event pairing logic - PASS
- `subagent_started` creates a pending exchange keyed by `task_id`
- `subagent_progress` updates the pending exchange (guarded by `task_id` presence and map lookup)
- `subagent_completed` finalizes and emits the exchange, cleaning up the map
- Orphaned in-progress subagents are flushed at the end of processing
- Events without `task_id` are safely ignored (no crash)

### 3. Component design - PASS
- `SubagentCard` follows the same visual pattern as `ToolCallCard` (left border, collapsible details, summary)
- Uses `Bot` icon (vs `Wrench`) for visual distinction
- Three-state status badge: running (amber spinner), completed (green check), failed (red X)
- Helper functions `formatDuration` and `formatTokens` handle edge cases (sub-second, >1k tokens)

### 4. No regressions - PASS
- Existing event types (`init`, `assistant`, `tool_progress`, `tool_summary`, `user`, `result`, `error`) are unchanged
- Default case still skips `system`, `stream_event`, `auth_status`, `status`
- TypeScript type check passes with no errors

### 5. Doc comment - MINOR
The JSDoc at the top of `pairEvents.ts` still says "Other event types -> skipped" but subagent events are now handled. Not blocking.

## Verdict

APPROVED. Clean implementation, consistent with existing patterns, no regressions.
