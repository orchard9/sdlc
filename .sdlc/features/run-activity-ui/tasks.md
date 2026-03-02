# Tasks: run-activity-ui

- [ ] Add `useRunTelemetry(runId)` hook to frontend API layer — fetches `GET /api/runs/:id/telemetry`, returns `{ events, isLoading, error }`
- [ ] Implement `pairEvents(events)` utility — pairs `tool_call`+`tool_result` by `tool_use_id`, nests `subagent_progress`/`subagent_completed` under `subagent_started` by `task_id`
- [ ] Build `RunActivityFeed.tsx` — top-level component; fetches events, calls `pairEvents`, renders ordered event list; re-fetches every 2s while run is active
- [ ] Build event card components: `RunInitCard`, `ToolCallCard` (with collapsible input JSON + result body + error badge), `SubagentCard` (started + progress + completed), `AssistantTextBlock`, `ThinkingBlock` (collapsed by default), `RunResultCard`
- [ ] Wire `RunActivityFeed` into run detail view — add accordion expand on run row click or navigate to `/runs/:id` route; show the activity feed as the primary content
