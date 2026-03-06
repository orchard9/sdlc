# Tasks: Subagent Input/Output Cards in Activity Feed

## Tasks

1. **Add PairedSubagentExchange type** — Add the new interface to `types.ts` and extend the `PairedEvent` union type.

2. **Update pairEvents to handle subagent events** — Add `subagent_started`, `subagent_progress`, `subagent_completed` handling in `pairEvents.ts` with task_id-based grouping.

3. **Create SubagentCard component** — New component in `SubagentCard.tsx` with collapsible details, status badges, and summary display.

4. **Wire SubagentCard into RunActivityFeed** — Import SubagentCard and add the `subagent_exchange` case to `PairedEventRow`.
