# Data Availability Analysis

## What RunRecord Already Has
- `id`, `key` (e.g. `sdlc-run:my-feature`), `run_type`, `target` (slug)
- `status`: running / completed / failed / stopped
- `started_at`, `completed_at`, `cost_usd`, `turns`
- `label`: human-readable description
- `session_id`: for ponder/investigation runs

## What's Missing
- **No `created_by` / `user` field** — RunRecord doesn't track who started it
- Heartbeat only sends `agent_running: bool` — no per-run detail to hub

## Frontend Infrastructure Already In Place
- `AgentRunContext` with `isRunning(key)` and `getRunForKey(key)`
- SSE subscription for `run_started` / `run_finished` events
- `MilestoneDigestRow` already checks `isRunning(nextFeature.slug)` — but only for the *next* feature, not all features
- `useMilestoneUatRun()` hook for UAT status

## Gap
The data and plumbing exist. The gap is:
1. Surfacing run status consistently on ALL feature cards and milestone rows
2. Adding user identity to RunRecord (requires backend change)
3. Showing which specific runs are active per milestone (aggregate view)