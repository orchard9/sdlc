# Spec: _shared/agent.ts — Agent Invocation from Within Tools

## Overview

Tools need to be able to invoke Claude agent runs from inside their execution without deadlocking the server. This feature adds `_shared/agent.ts` to the tools shared library, which exports a `runAgent()` function that tools use to delegate work to an agent. The agent call is routed through a new `POST /api/tools/agent-call` endpoint on the local sdlc-server.

## Background

The existing tool execution model (`run_tool` in `tool_runner.rs`) is synchronous and blocking — the server waits for the tool process to complete before returning. If a tool invokes an agent call that also waits, and the agent call routes back through the same server, a deadlock occurs: the server's blocking thread pool is exhausted waiting on itself.

The `tool-streaming-async` feature (in parallel development) solves this by making tool execution non-blocking. **This feature depends on `tool-streaming-async`**: the agent call endpoint is only safe to use from tools that run asynchronously (i.e. tools with `streaming: true` in their meta). The spec notes this dependency and the server-side safeguard.

## User-Facing Behavior

A tool developer writes:

```typescript
import { runAgent } from '../_shared/agent.ts'

const result = await runAgent({
  prompt: 'Analyze the last 5 git commits and summarize themes',
  maxTurns: 10,
})
```

The tool receives the agent's result text as a string. The agent run is visible in the run history panel in the UI (run_type: `"tool-agent"`).

## Functional Requirements

### 1. `_shared/agent.ts` API

Exports one function:

```typescript
export interface RunAgentOptions {
  prompt: string
  /** Optional path to an agent definition file (.md). If provided, content
      is prepended to the system prompt. */
  agentFile?: string
  /** Maximum turns for the agent. Defaults to 20. */
  maxTurns?: number
}

export async function runAgent(opts: RunAgentOptions): Promise<string>
```

- Reads `SDLC_SERVER_URL` from env (set by the server for every tool run).
- Reads `SDLC_AGENT_TOKEN` from env (set by the server per-instance; validates the call is from a legitimate tool subprocess).
- POSTs `{ prompt, agentFile, maxTurns }` to `${SDLC_SERVER_URL}/api/tools/agent-call`.
- Sends `Authorization: Bearer ${SDLC_AGENT_TOKEN}` header.
- Waits for the HTTP response (which blocks until the agent run completes).
- Returns the agent's result text on success.
- Throws an `Error` with the server's error message on failure.

### 2. `POST /api/tools/agent-call` Endpoint

A new server endpoint that:

1. **Validates the `SDLC_AGENT_TOKEN`**: The `Authorization: Bearer <token>` header must match the per-instance token stored in `AppState`. Returns 401 if missing or invalid. The token is a random 32-char hex string generated at server startup and regenerated on each restart. Unlike the tunnel token (which gates UI access), the agent token gates only this endpoint and is never exposed to tunnel/browser clients.

2. **Spawns the agent** via `spawn_agent_run` with:
   - `run_type`: `"tool-agent"`
   - `label`: `"agent-call"`
   - `opts`: `sdlc_query_options(root, max_turns)` — no extra tools beyond the standard set
   - No `completion_event`

3. **Waits** for the run to complete before responding. The HTTP response is **synchronous** — the caller (tool) blocks on it. This works without deadlock only because the tool itself runs asynchronously (via `tool-streaming-async`). The endpoint spawns the agent then waits on the broadcast channel for the RunFinished event, with a 10-minute timeout.

4. **Returns** `{ "result": "<agent result text>", "cost_usd": <number>, "turns": <number> }` on success, or `{ "error": "<message>" }` on failure.

### 3. `SDLC_AGENT_TOKEN` and `SDLC_SERVER_URL` Injection

The server injects two new environment variables for **every** tool run (both `--run` and `--setup`):

- `SDLC_SERVER_URL`: `http://localhost:<port>` — the local server base URL
- `SDLC_AGENT_TOKEN`: a 32-char hex token stored in `AppState`

The token is generated once at `AppState::new_with_port()` time (server startup) and stored as `Arc<String>` in `AppState`. It is never persisted to disk — a server restart generates a new token.

The injection happens in `crates/sdlc-server/src/routes/tools.rs` (both `run_tool` and `setup_tool` handlers), passed as additional entries in `extra_env`.

### 4. Token Validation

The `/api/tools/agent-call` endpoint validates the `Authorization: Bearer <token>` header inline (not via tunnel auth middleware). Returns 401 JSON on mismatch.

This endpoint is **local-only by design**: tool processes run locally and reach `http://localhost:port` directly. The tunnel auth middleware blocks all non-local `/api/*` routes lacking a session cookie, so external callers cannot abuse this endpoint.

## Non-Requirements

- No streaming of agent output back to the tool — the tool receives the final result only.
- No support for custom MCP servers or tool lists in the agent call.
- No support for calling `runAgent` from a synchronous (blocking) tool.

## Security Considerations

- `SDLC_AGENT_TOKEN` is a per-server-instance secret, not user-configurable.
- The token prevents cross-process injection from rogue subprocesses.
- The endpoint is unreachable via the public tunnel.

## Acceptance Criteria

1. `_shared/agent.ts` exists with the `runAgent` function exported.
2. A tool using `runAgent` successfully receives an agent result string.
3. The agent call appears in the run history with `run_type: "tool-agent"`.
4. Calls to `/api/tools/agent-call` without the correct bearer token return 401.
5. `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` are present in the subprocess env for every tool run.
6. Build passes with no new `unwrap()` in library code.
