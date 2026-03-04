# Tasks: Streaming + Async Tool Execution

## T1 — Add `run_tool_streaming` to `sdlc-core/src/tool_runner.rs`

Add an async function that spawns the tool process with `tokio::process::Command` and piped stdout, returning the child handle. Use the same runtime detection, env injection, and `SDLC_ROOT` setup as `run_tool`.

**File:** `crates/sdlc-core/src/tool_runner.rs`
**Test:** Unit test that `run_tool_streaming` returns a child with piped stdout (using a trivial inline script or skip if no runtime).

---

## T2 — Add four new SSE variants to `state.rs`

Add `ToolRunStarted`, `ToolRunProgress`, `ToolRunCompleted`, and `ToolRunFailed` to the `SseMessage` enum.

**File:** `crates/sdlc-server/src/state.rs`

---

## T3 — Wire new SSE variants in `events.rs`

Add match arms for the four new `SseMessage` variants. All emit on the `"tool"` SSE event channel as JSON objects with a `type` discriminator.

**File:** `crates/sdlc-server/src/routes/events.rs`

---

## T4 — Add `load_streaming_log` to `tool_interaction.rs`

Add a function that reads the NDJSON sidecar file (`.sdlc/tool-interactions/<name>/<id>.log`) and returns a `Vec<serde_json::Value>` of valid lines.

**File:** `crates/sdlc-core/src/tool_interaction.rs`
**Test:** Unit test that parses a multiline NDJSON string and skips invalid lines.

---

## T5 — Implement async streaming execution path in `routes/tools.rs`

Modify the `run_tool` handler to branch on `meta.streaming`:
- `streaming: false` (or absent) → existing synchronous path unchanged.
- `streaming: true` → create `ToolInteractionRecord` with `status: "streaming"`, `streaming_log: true`; call `run_tool_streaming`; `tokio::spawn` a task that reads stdout line-by-line, appends to `.log` sidecar, emits `ToolRunProgress` SSE per line; on exit sets final status and emits `ToolRunCompleted` or `ToolRunFailed`; return HTTP 202 `{ "job_id": interaction_id, "streaming": true }`.

**File:** `crates/sdlc-server/src/routes/tools.rs`
**Tests:**
- Streaming handler returns 202 for a tool that declares `streaming: true`.
- Non-streaming handler still returns 200 (regression guard).
- Invalid JSON lines on stdout are silently skipped.

---

## T6 — Frontend: handle 202 response in tool run panel

Update `frontend/src/api/client.ts` `runTool` return type and the tool run panel component to:
- Detect `streaming: true` in the response.
- Subscribe to SSE `tool` events filtered by `interaction_id`.
- Accumulate and render `ToolRunProgress` lines.
- On `ToolRunCompleted`, fetch the full interaction record and render the final result.
- On `ToolRunFailed`, display the error.

**Files:**
- `frontend/src/api/client.ts`
- `frontend/src/lib/types.ts`
- Tool run panel component (wherever the run button + result display lives)

---

## T7 — Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`; fix all failures

All tests must pass and clippy must produce no warnings.
