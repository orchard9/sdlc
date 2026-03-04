# Design: _shared/agent.ts — Agent Invocation from Within Tools

## Architecture Overview

```
Tool Process (bun/deno/node)
  └─ _shared/agent.ts: runAgent()
       └─ HTTP POST /api/tools/agent-call
            └─ sdlc-server (Axum)
                 ├─ token validation (SDLC_AGENT_TOKEN)
                 ├─ spawn_agent_run(..., "tool-agent", ...)
                 └─ wait for completion → return result text
```

The call is synchronous from the tool's perspective: `runAgent()` awaits the HTTP response, which the server only sends after the agent run completes. This is safe because tools with `streaming: true` in their meta run asynchronously in a background Tokio task — they do not block a synchronous thread pool slot.

## Component Changes

### 1. `AppState` — Token Field

Add `agent_token: Arc<String>` to `AppState`:

```rust
// In state.rs
pub struct AppState {
    // ... existing fields ...
    /// Per-instance token for tool-to-server agent calls.
    /// Generated at startup, never persisted.
    pub agent_token: Arc<String>,
}
```

Generate in `AppState::new_with_port()`:

```rust
let agent_token = Arc::new(generate_agent_token());
```

where `generate_agent_token()` reads 16 bytes from `/dev/urandom` (via `std::fs::File`) and hex-encodes them (32 hex chars). Falls back to a timestamp-seeded value if `/dev/urandom` is unavailable (Windows support).

### 2. Token Injection into Tool Subprocess

In `crates/sdlc-server/src/routes/tools.rs`, the `run_tool` and `setup_tool` handlers build an `extra_env` map before invoking `tool_runner::run_tool`. Two new entries are added unconditionally:

```rust
extra_env.insert("SDLC_SERVER_URL".into(), format!("http://localhost:{}", app.port));
extra_env.insert("SDLC_AGENT_TOKEN".into(), (*app.agent_token).clone());
```

The `app.port` field already exists in `AppState`. Since both handlers currently build `extra_env` from `resolve_secrets()`, these two entries are merged into the result map before passing to `tool_runner::run_tool`.

### 3. `POST /api/tools/agent-call` Endpoint

**Location**: New handler in `crates/sdlc-server/src/routes/tools.rs`.

**Route registration** in `lib.rs`:
```rust
.route("/api/tools/agent-call", post(routes::tools::agent_call))
```
Registered before the `{name}` wildcard so it resolves first.

**Request body**:
```json
{ "prompt": "...", "agentFile": "...", "maxTurns": 20 }
```

**Handler logic**:

```rust
pub async fn agent_call(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AgentCallRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Token validation
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| AppError::unauthorized("missing agent token"))?;
    if token != *app.agent_token {
        return Err(AppError::unauthorized("invalid agent token"));
    }

    // 2. Build prompt (optionally prepend agentFile contents)
    let prompt = build_agent_prompt(&app.root, &body)?;

    // 3. Spawn the agent run
    let max_turns = body.max_turns.unwrap_or(20).min(100);
    let opts = sdlc_query_options(app.root.clone(), max_turns);
    let run_key = format!("tool-agent:{}", generate_run_id());
    spawn_agent_run(run_key.clone(), prompt, opts, &app, "tool-agent", "agent-call", None).await?;

    // 4. Poll for completion (max 10 minutes)
    let result = wait_for_run_completion(&run_key, &app, Duration::from_secs(600)).await?;

    Ok(Json(serde_json::json!({
        "result": result.result_text,
        "cost_usd": result.cost_usd,
        "turns": result.turns,
    })))
}
```

**`wait_for_run_completion`**: Subscribes to the SSE broadcast channel for `run_key` and waits for the `RunFinished` event. Uses a Tokio `timeout`. Returns the final `RunRecord` data (cost, turns, error). If the run failed, returns an `AppError`.

The existing `spawn_agent_run` already inserts a `(tx, abort_handle)` into `agent_runs`; the broadcast sender is available before the task completes. `wait_for_run_completion` subscribes to this sender and listens for the `RunFinished` JSON event emitted by the task on completion.

### 4. `_shared/agent.ts`

**Location**: `.sdlc/tools/_shared/agent.ts`

```typescript
import { getEnv } from './runtime.ts'

export interface RunAgentOptions {
  prompt: string
  agentFile?: string
  maxTurns?: number
}

export interface AgentResult {
  result: string
  cost_usd?: number
  turns?: number
}

/**
 * Invoke a Claude agent run from inside a tool.
 *
 * IMPORTANT: Only safe to call from tools with `streaming: true` in their
 * meta. Calling runAgent() from a synchronous (blocking) tool will cause
 * the server's thread pool to deadlock.
 *
 * Reads SDLC_SERVER_URL and SDLC_AGENT_TOKEN from the environment.
 * Both are injected by the server for every tool run.
 */
export async function runAgent(opts: RunAgentOptions): Promise<string> {
  const serverUrl = getEnv('SDLC_SERVER_URL')
  const token = getEnv('SDLC_AGENT_TOKEN')

  if (!serverUrl || !token) {
    throw new Error(
      'runAgent: SDLC_SERVER_URL or SDLC_AGENT_TOKEN not set. ' +
      'This function only works when the tool is invoked by sdlc-server.'
    )
  }

  const response = await fetch(`${serverUrl}/api/tools/agent-call`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
    },
    body: JSON.stringify({
      prompt: opts.prompt,
      agentFile: opts.agentFile,
      maxTurns: opts.maxTurns ?? 20,
    }),
  })

  if (!response.ok) {
    const body = await response.text()
    throw new Error(`agent-call failed (${response.status}): ${body}`)
  }

  const data = await response.json() as { result?: string; error?: string }
  if (data.error) throw new Error(`agent-call error: ${data.error}`)
  return data.result ?? ''
}
```

Uses the native `fetch` API available in Bun, Deno, and Node 18+. No additional dependencies.

## Data Flow — Agent Call Sequence

```
Tool --run process
  1. runAgent({ prompt: "..." })
  2. POST /api/tools/agent-call
       Bearer: <SDLC_AGENT_TOKEN>
  3. Server validates token
  4. Server calls spawn_agent_run("tool-agent:...", ...)
       → inserts (tx, abort_handle) into agent_runs map
       → tokio::spawn(async { query(prompt, opts)... })
  5. Server calls wait_for_run_completion(run_key)
       → subscribes to broadcast tx
       → waits for "result" event
  6. Agent task runs to completion
       → emits "result" JSON to broadcast tx
       → emits RunFinished SSE
       → removes run from agent_runs map
  7. wait_for_run_completion receives "result" event
  8. Server responds 200 { result, cost_usd, turns }
  9. Tool receives result string
```

## Error Handling

| Failure | Server response | Tool behavior |
|---|---|---|
| Missing/invalid token | 401 `{ "error": "..." }` | `runAgent` throws |
| agentFile not found | 400 `{ "error": "..." }` | `runAgent` throws |
| Agent run fails | 500 `{ "error": "..." }` | `runAgent` throws |
| Agent run times out (10 min) | 504 `{ "error": "timed out" }` | `runAgent` throws |
| Server not running | Network error | `runAgent` throws |

## Token Security

- Token is 32 hex chars (128 bits entropy) — brute-force infeasible.
- Token is only injected as a subprocess env var — not written to disk or logs.
- The tunnel auth middleware already requires the session cookie for all `/api/*` routes from non-local hosts. Since tools run locally (same machine as server), they can reach `localhost:port` directly without the cookie.
- The agent_token field in `AppState` is an `Arc<String>` — cloning the state does not copy the bytes, just increments the refcount. The token identity remains stable for the server's lifetime.

## Test Coverage

- Unit test: `validate_agent_token` rejects empty/wrong/correct tokens.
- Unit test: `agent_call` handler returns 401 for missing Authorization header.
- Unit test: `agent_call` handler returns 401 for wrong token.
- Integration test (mocked): `runAgent` in `_shared/agent.ts` is covered by the existing tool integration test infra (not Rust tests — documented in QA plan).

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/src/state.rs` | Add `agent_token: Arc<String>` to `AppState`; generate in `new_with_port()` |
| `crates/sdlc-server/src/routes/tools.rs` | Inject `SDLC_SERVER_URL` + `SDLC_AGENT_TOKEN` in `run_tool` and `setup_tool`; add `agent_call` handler |
| `crates/sdlc-server/src/lib.rs` | Register `/api/tools/agent-call` route |
| `.sdlc/tools/_shared/agent.ts` | New file: `runAgent()` export |
