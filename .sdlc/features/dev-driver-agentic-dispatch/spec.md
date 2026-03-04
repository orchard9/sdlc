# Spec: dev-driver-agentic-dispatch

## Problem

`dev-driver` currently dispatches Claude agent runs by calling `spawn('claude', ['--print', command], { detached: true })` — a raw shell subprocess. This pattern bypasses all server infrastructure:

- No `RunRecord` is created — the run is invisible to `GET /api/runs`
- No SSE events are emitted — the activity feed shows nothing
- The frontend has no way to know a run is in progress
- A TTL-based lock file (`.sdlc/.dev-driver.lock`) was introduced as a workaround because there is no other flight detection mechanism

The lock file is fragile: it uses process PID (which may be reused) and a 2-hour TTL that is unrelated to actual run duration. It cannot detect stale locks reliably. Most importantly, the whole pattern is architecturally inconsistent with every other agent dispatch in the codebase (ponder, investigation, milestone UAT, feature runs, etc.) which all go through `spawn_agent_run`.

## Solution

Rewrite `dev-driver` dispatch to use the existing server infrastructure:

1. Add a `POST /api/tools/agent-call` endpoint to `sdlc-server` that accepts a command string (e.g. `/sdlc-next my-feature`) and dispatches it via `spawn_agent_run`. The endpoint returns a RunRecord ID and is keyed under `dev-driver:<slug>` or `dev-driver:wave:<milestone>` for duplicate detection.

2. Add `_shared/agent.ts` — a shared helper for tools that need to call back to the server to spawn agent runs. It exports a single `runAgent(command, root)` function that calls `POST /api/tools/agent-call`.

3. Rewrite `dev-driver/tool.ts` dispatch levels 3 and 4 to call `runAgent()` instead of `spawnClaude()`.

4. Remove the lock file entirely. The server's `agent_runs` map provides flight detection — `spawn_agent_run` returns HTTP 409 Conflict when a key is already running. The `hasActiveRuns()` check (which already queries `sdlc run list --status running`) covers the remaining case.

5. Update `dev-driver/README.md` to document the new dispatch pattern.

## New Endpoint: POST /api/tools/agent-call

```
POST /api/tools/agent-call
Content-Type: application/json

{
  "command": "/sdlc-next my-feature",
  "key": "dev-driver:feature:my-feature",
  "label": "dev-driver: advance my-feature",
  "run_type": "dev_driver"
}
```

Response (202 Accepted):
```json
{
  "run_id": "20260303-120000-abc",
  "key": "dev-driver:feature:my-feature",
  "status": "started"
}
```

If a run with the same key is already in flight, returns 409 Conflict:
```json
{
  "error": "Agent already running for 'dev-driver:feature:my-feature'"
}
```

The agent prompt is: `You are an autonomous SDLC agent. Execute: <command>`. This wraps the slash command in a brief system context so the agent executes it directly.

## New File: _shared/agent.ts

```typescript
export interface AgentCallResult {
  run_id: string
  key: string
  status: 'started' | 'conflict'
}

export async function runAgent(
  command: string,
  key: string,
  label: string,
  serverUrl: string,
): Promise<AgentCallResult>
```

The function POSTs to `${serverUrl}/api/tools/agent-call` and returns the result. If the server returns 409, it returns `{ status: 'conflict', ... }` rather than throwing — the caller can decide whether to log or ignore.

The server URL is derived from the `SDLC_SERVER_URL` environment variable (default: `http://127.0.0.1:3141`).

## Behavior After Change

| Scenario | Before | After |
|---|---|---|
| Feature dispatch | `spawn('claude', ['--print', '/sdlc-next slug'])` detached | `POST /api/tools/agent-call { command, key, label }` |
| Wave dispatch | `spawn('claude', ['--print', '/sdlc-run-wave milestone'])` detached | `POST /api/tools/agent-call { command, key, label }` |
| Flight detection | TTL lock file (fragile) | HTTP 409 from `spawn_agent_run` (exact) |
| Run visibility | Invisible | RunRecord in activity feed, SSE events |
| Lock file | Written to `.sdlc/.dev-driver.lock` | Removed entirely |

## Scope

- `crates/sdlc-server/src/routes/runs.rs` — add `agent_call` handler
- `crates/sdlc-server/src/lib.rs` — register `POST /api/tools/agent-call` route
- `.sdlc/tools/_shared/agent.ts` — new shared helper
- `.sdlc/tools/dev-driver/tool.ts` — rewrite dispatch + remove lock file
- `.sdlc/tools/dev-driver/README.md` — update docs

## Out of Scope

- Changing the 5-level priority waterfall logic
- Changing quality-check behavior
- Changing feature selection or wave detection
- Adding new output types
