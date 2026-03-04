# Review: Streaming + Async Tool Execution

## Summary

The backend implementation of streaming/async tool execution is complete and production-quality. The Rust server side correctly branches on `meta.streaming`, spawns background tasks, emits all four SSE variants, and persists interaction records with sidecar log files. Tests pass and clippy is clean. However, the frontend has not yet been updated to consume the 202 response, subscribe to SSE events, or render streaming progress — this is a known gap tracked as T6 in tasks.md.

## Findings

### Backend (crates/sdlc-server/src/routes/tools.rs)

**PASS** — Streaming branch at line 180 correctly checks `meta.streaming == Some(true)`.

**PASS** — SDLC_SERVER_URL and SDLC_AGENT_TOKEN are injected as env vars for every tool run (both streaming and non-streaming paths), at lines 172-173 and 538-539.

**PASS** — `ToolInteractionRecord` is created with `status: "streaming"` and `streaming_log: true` before the background task is spawned, ensuring the record exists from the moment the job is accepted.

**PASS** — HTTP 202 with `{ "job_id": ..., "streaming": true }` is returned immediately (line 369-375).

**PASS** — Background task uses `tokio::io::BufReader::lines()` to read stdout line-by-line without buffering full output.

**PASS** — Each line is appended to the `.log` sidecar file and emitted as `ToolRunProgress` SSE event on the `"tool"` channel.

**PASS** — Non-JSON lines are silently dropped (debug-logged) at line 322-325.

**PASS** — On process exit, the last valid JSON line with an `"ok"` key is used as the final result; record status is updated to `"completed"` or `"failed"`.

**PASS** — `enforce_interaction_retention` is called after completion to cap stored records at 200.

**OBSERVATION** — The streaming path does not currently use `run_tool_streaming` from `tool_runner.rs` (the async helper described in tasks.md T1). Instead, it constructs the tokio process command inline in the route handler using `tool_spawn_args`. The design specified `run_tool_streaming` as a separate function in `tool_runner.rs`, but the implementation inlines it in the handler. This is functionally correct and keeps the logic co-located, but deviates from the layered design. Captured as a follow-up task for refactoring if desired.

### SSE Wiring (crates/sdlc-server/src/state.rs + routes/events.rs)

**PASS** — All four variants (`ToolRunStarted`, `ToolRunProgress`, `ToolRunCompleted`, `ToolRunFailed`) are defined in `SseMessage` at state.rs lines 266-287.

**PASS** — All four variants are matched in events.rs at lines 227-278. Each emits on the `"tool"` SSE channel with a `type` discriminator (`tool_run_started`, `tool_run_progress`, `tool_run_completed`, `tool_run_failed`).

### Tool Interaction Persistence (crates/sdlc-core/src/tool_interaction.rs)

**PASS** — `load_streaming_log` function exists and correctly reads the NDJSON sidecar, parses valid JSON lines, and skips invalid lines.

**PASS** — `streaming_log_path` helper provides a stable path convention: `.sdlc/tool-interactions/<name>/<id>.log`.

**PASS** — Unit tests cover: save/load, list newest-first, retention, `load_streaming_log` with valid and invalid JSON lines, and empty log file.

### Frontend (frontend/src/pages/ToolsPage.tsx + api/client.ts + lib/types.ts)

**GAP** — `api.runTool` in `client.ts` (line 208) is typed to return `ToolResult`, not a union. When the server returns HTTP 202 with `{ "job_id": ..., "streaming": true }`, the frontend interprets it as a `ToolResult`. The UI will show this job_id object as the result rather than subscribing to SSE progress events. This is the T6 gap from tasks.md.

**GAP** — `handleRun` in `ToolsPage.tsx` (line 831) unconditionally does `setResult(res)` after `api.runTool`. There is no branch to detect `streaming: true` and subscribe to SSE events.

**GAP** — `ToolInteractionRecord.status` type in `types.ts` (line 848) only lists `'running' | 'completed' | 'failed'`. The value `'streaming'` (set during active streaming runs) is not in the union.

**ACCEPTABLE** — The `ToolMeta.streaming?: boolean` field already exists in `types.ts` (line 827). No change needed there.

### Tests

**PASS** — `SDLC_NO_NPM=1 cargo test --all` passes with 812+ tests across all crates (no failures).

**PASS** — `cargo clippy --all -- -D warnings` passes with no warnings.

## Verdict

The backend is complete and correct. The frontend gap (T6) means streaming tools currently deliver a degraded UX (the job_id JSON appears as the "result" instead of streaming progress). This does not break non-streaming tools or the server, but the feature's streaming capability is invisible to users until T6 is implemented.

**Decision:** Accept the review. Add the frontend gap as a tracked task so it's addressed in a follow-up cycle.

## Action Items

1. Add task: "Frontend: update runTool return type and ToolsPage to handle 202/streaming response, subscribe to SSE tool events, and render progress lines" — follow-up feature or task in the next cycle.
2. Optional refactor: Extract inline streaming spawn logic from `routes/tools.rs` into `tool_runner::run_tool_streaming` to match the design's layered architecture.
