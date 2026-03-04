# Tasks: _shared/agent.ts — Agent Invocation from Within Tools

## T1 — Add `agent_token` field to `AppState`

**File**: `crates/sdlc-server/src/state.rs`

Add `pub agent_token: Arc<String>` to the `AppState` struct. Add a `generate_agent_token()` function that reads 16 bytes from the OS CSPRNG (`std::fs::read("/dev/urandom")` on Unix, fallback to timestamp-seeded value on Windows) and hex-encodes them to produce a 32-char token. Call this in `AppState::new_with_port()` and store the result.

No `unwrap()` — use `unwrap_or_else` with the fallback for the file-read path.

---

## T2 — Inject `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` into every tool subprocess

**File**: `crates/sdlc-server/src/routes/tools.rs`

In both `run_tool` and `setup_tool` handlers, after `resolve_secrets()` builds `extra_env`, insert two additional entries unconditionally:

- `SDLC_SERVER_URL` → `format!("http://localhost:{}", app.port)`
- `SDLC_AGENT_TOKEN` → `(*app.agent_token).clone()`

Both handlers already have access to `app: State<AppState>`. The `State<AppState>` extractor must be added to `setup_tool`'s signature if not already present (it currently uses `State(app)`).

---

## T3 — Add `POST /api/tools/agent-call` endpoint

**File**: `crates/sdlc-server/src/routes/tools.rs`

Add:

1. `AgentCallRequest` struct: `{ prompt: String, agent_file: Option<String>, max_turns: Option<u32> }`
2. `agent_call` async handler:
   - Extracts `Authorization: Bearer <token>` from headers; returns 401 if missing or mismatched with `app.agent_token`
   - Optionally reads `agent_file` from disk and prepends to prompt
   - Calls `spawn_agent_run` with `run_type = "tool-agent"`, `label = "agent-call"`, `opts = sdlc_query_options(root, max_turns.min(100))`
   - Subscribes to the broadcast sender from `agent_runs` map (the entry inserted by `spawn_agent_run`)
   - Waits (via `tokio::time::timeout(10 min)`) for a JSON message of type `"result"` on the broadcast channel
   - Extracts `result_text`, `cost_usd`, `turns` from the result event
   - Returns `200 { result, cost_usd, turns }` or `500 { error }` / `504 { error }` on timeout

Helper `extract_bearer_token(headers: &HeaderMap) -> Option<String>` — parses `Authorization` header.

---

## T4 — Register `/api/tools/agent-call` route in `lib.rs`

**File**: `crates/sdlc-server/src/lib.rs`

Add before the `{name}` wildcard routes (i.e. before `.route("/api/tools/{name}", ...)`):

```rust
.route("/api/tools/agent-call", post(routes::tools::agent_call))
```

This must appear before the wildcard so Axum resolves it first.

---

## T5 — Create `.sdlc/tools/_shared/agent.ts`

**File**: `.sdlc/tools/_shared/agent.ts`

Implement `runAgent(opts: RunAgentOptions): Promise<string>` per the design. Uses the native `fetch` API (available in Bun, Deno, and Node 18+). Reads `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` via `getEnv` from `./runtime.ts`. Throws a descriptive error if either env var is absent, if the HTTP call fails, or if the response body contains `{ error }`.

Include a JSDoc comment warning that `runAgent` is only safe from async (streaming) tools.

---

## T6 — Unit tests for token validation and `agent_call` handler

**File**: `crates/sdlc-server/src/routes/tools.rs` (existing `#[cfg(test)]` block)

Add:

- `extract_bearer_token_parses_valid_header` — asserts correct extraction
- `extract_bearer_token_returns_none_for_missing_header` — asserts `None`
- `agent_call_returns_401_for_missing_token` — calls handler with no `Authorization` header, asserts 401
- `agent_call_returns_401_for_wrong_token` — calls handler with wrong token, asserts 401

These tests use the existing `AppState::new()` test pattern (tempdir) and do not spawn a real agent.

---

## Completion Gate

All six tasks done, `SDLC_NO_NPM=1 cargo test --all` passes, `cargo clippy --all -- -D warnings` passes.
