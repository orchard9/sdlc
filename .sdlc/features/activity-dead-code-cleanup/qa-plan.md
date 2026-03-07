# QA Plan — Activity Dead Code Cleanup

## Verification Steps

1. **Build succeeds** — `cd frontend && npm run build` completes with zero errors
2. **No dangling imports** — grep the frontend source for `AgentLog`, `AgentEventLine`; expect zero results
3. **AgentEvent type cleanup** — if removed, grep confirms zero references; if retained, all remaining references compile
4. **Active run rendering** — expand a running agent card in the UI; it renders activity (not blank/broken)
5. **Completed run rendering** — expand a completed run card; `RunActivityFeed` + `ActivityTimeSeries` render as before (no regression)
6. **No dead files** — `AgentLog.tsx` and `AgentEventLine.tsx` no longer exist on disk
