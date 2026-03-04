# Spec: Streaming + Async Tool Execution

## Problem

Tool execution via `POST /api/tools/:name/run` is fully synchronous. The server blocks a thread in `spawn_blocking` until the tool process exits, then returns the entire result as a single JSON response. This creates several problems:

1. **No progress visibility** — long-running tools (quality-check, telegram-recap, citadel tools) run silently; the UI shows a spinner with no feedback until completion or timeout.
2. **Thread starvation** — `spawn_blocking` holds a Tokio blocking thread for the full tool duration. Tools with `timeout_seconds > 30` risk exhausting the thread pool under concurrent use.
3. **No interruption** — there is no way to cancel a running tool from the frontend.
4. **ToolMeta already declares intent** — `ToolMeta.streaming: Option<bool>` exists as a field in `tool_runner.rs` but is never acted upon. Tools that set `streaming: true` get the same synchronous behavior as tools that don't.

## Proposal

Wire the `streaming: true` meta flag end-to-end so that streaming tools:

1. Return a job ID immediately from `POST /api/tools/:name/run` (non-blocking HTTP response).
2. Execute in a background `tokio::spawn` task.
3. Emit NDJSON progress lines to stdout; the server captures each line and relays it as an SSE event on the existing `/api/events` channel.
4. Produce a `RunRecord` entry (same as agent runs) for history and observability.
5. Emit `ToolRunStarted` and `ToolRunCompleted` / `ToolRunFailed` SSE messages when execution begins and ends.

Non-streaming tools (`streaming: absent/false`) continue to use the existing synchronous path — no behavior change.

## Scope

### In scope
- New async execution path for tools with `streaming: true` in their `--meta` output.
- NDJSON line protocol: each line the tool writes to stdout during `--run` is a JSON object; the server relays it as an SSE event.
- `ToolInteractionRecord` updated with a `streaming_log` sidecar file (`.sdlc/tool-interactions/<name>/<id>.log`) that accumulates raw NDJSON lines.
- New SSE variants: `ToolRunStarted { name, interaction_id }` and `ToolRunCompleted { name, interaction_id }` / `ToolRunFailed { name, interaction_id, error }`.
- `POST /api/tools/:name/run` response changes for streaming tools: returns `{ "job_id": "<interaction_id>", "streaming": true }` with HTTP 202.
- `GET /api/tools/:name/interactions/:id` already exists and now returns `streaming_log: true` when the sidecar is present.
- Frontend ToolRunPanel subscribes to SSE and renders streaming progress lines as they arrive.

### Out of scope
- Cancellation endpoint (tracked as a follow-up task).
- Streaming for `--setup` mode.
- Changes to non-streaming tool execution (fully backward-compatible).
- Tool runtime changes (tools must already write NDJSON to stdout line-by-line; no changes to `_shared/types.ts` protocol for non-streaming tools).

## Protocol: Streaming Tool stdout

A streaming tool writes **one JSON object per line** to stdout during `--run`. Each line is an NDJSON progress event. The final line must be the standard `ToolResult` envelope:

```
{"type":"progress","message":"Scanning 312 files...","pct":10}
{"type":"progress","message":"Found 5 issues in auth module","pct":60}
{"ok":true,"data":{"issues_found":5},"duration_ms":4200}
```

- Lines that are not valid JSON are silently dropped (logged to stderr).
- The server accumulates all lines into the `.log` sidecar as raw NDJSON.
- The final `ToolResult` line is parsed as the interaction `result` field.

## Data Changes

### `ToolInteractionRecord` (sdlc-core)
- `streaming_log: bool` already exists — when `true`, a sidecar file `<id>.log` is present.
- No new fields needed; the `status` field gains a new value: `"streaming"` (set while the background task is running, updated to `"completed"` or `"failed"` when done).

### SSE variants (state.rs)
```rust
ToolRunStarted { name: String, interaction_id: String },
ToolRunCompleted { name: String, interaction_id: String },
ToolRunFailed { name: String, interaction_id: String, error: String },
/// A single NDJSON progress line emitted by a streaming tool.
ToolRunProgress { name: String, interaction_id: String, line: serde_json::Value },
```

### File layout
```
.sdlc/tool-interactions/<name>/
  <id>.yaml        ← ToolInteractionRecord (status: streaming → completed)
  <id>.log         ← raw NDJSON lines, one per line, appended as they arrive
```

## API Changes

### `POST /api/tools/:name/run`

**Non-streaming tools (unchanged):**
```
HTTP 200 OK
Content-Type: application/json
{ ...ToolResult... }
```

**Streaming tools:**
```
HTTP 202 Accepted
Content-Type: application/json
{ "job_id": "20260303-120000-abc", "streaming": true }
```

The frontend polls `GET /api/tools/:name/interactions/:id` for status, or listens to SSE for `ToolRunCompleted` / `ToolRunFailed` events. No new endpoint required.

## Implementation Notes

- Use `tokio::task::spawn` (not `spawn_blocking`) for the streaming execution path. The process is launched with `stdout: Stdio::piped()` and lines are read via `tokio::io::BufReader::lines()`.
- The `run_tool_streaming` function in `tool_runner.rs` returns a `tokio::process::Child` with piped stdout; the server handler owns the line-reading loop.
- Secrets resolution and meta fetching remain synchronous (fast, single `--meta` call) before the async run starts.
- The `streaming_log` sidecar is appended atomically per-line using a buffered file writer in the spawned task.
- SSE event channel capacity is already 512 per subscriber — `ToolRunProgress` events use the existing `event_tx` broadcast.

## Acceptance Criteria

1. A tool with `streaming: true` in its `--meta` output receives HTTP 202 with a `job_id` instead of blocking.
2. Each NDJSON line the tool writes to stdout appears as a `ToolRunProgress` SSE event within 500ms.
3. On completion, a `ToolRunCompleted` SSE event is emitted and the `ToolInteractionRecord` status changes from `streaming` to `completed`.
4. The `.log` sidecar file contains all NDJSON lines in order.
5. `GET /api/tools/:name/interactions/:id` returns the record with `streaming_log: true` and `status: "completed"`.
6. Non-streaming tools continue to return HTTP 200 with the full result synchronously.
7. `SDLC_NO_NPM=1 cargo test --all` passes.
8. `cargo clippy --all -- -D warnings` passes.
