# Design: Streaming + Async Tool Execution

## Overview

This design covers the full stack changes to wire the `streaming: true` meta flag from TypeScript tool metadata through the Rust server to SSE events and frontend progress display. The change is additive: non-streaming tools are untouched.

## Architecture

```
Tool process (TypeScript)
  stdout → NDJSON lines (one JSON object per line)
  stderr → inherited (terminal/run panel logs)

sdlc-server (Rust)
  POST /api/tools/:name/run
    ├─ streaming=false → spawn_blocking, wait_with_output → HTTP 200 JSON
    └─ streaming=true  → tokio::spawn, read lines → HTTP 202 { job_id }
                              │
                              ├─ line read → broadcast event_tx (ToolRunProgress)
                              ├─ line append → .sdlc/tool-interactions/<name>/<id>.log
                              └─ on exit → save ToolInteractionRecord, emit ToolRunCompleted

GET /api/events (SSE)
  ToolRunStarted { name, interaction_id }
  ToolRunProgress { name, interaction_id, line }   ← one per NDJSON line
  ToolRunCompleted { name, interaction_id }
  ToolRunFailed { name, interaction_id, error }

Frontend
  POST /api/tools/:name/run
    ├─ response.streaming=false → render result directly (unchanged)
    └─ response.streaming=true  → subscribe SSE, render ToolRunProgress lines
```

## Rust Changes

### 1. `crates/sdlc-core/src/tool_runner.rs`

Add `run_tool_streaming` — launches the tool process with `tokio::process::Command` and returns the child handle with piped stdout. Secrets injection and env setup are identical to `run_tool`.

```rust
pub async fn run_tool_streaming(
    script: &Path,
    stdin_json: Option<String>,
    root: &Path,
    extra_env: Option<&HashMap<String, String>>,
) -> Result<tokio::process::Child>
```

The caller (server route) owns the `Child` and drives the line-reading loop. This keeps `tool_runner.rs` a thin spawn layer with no policy logic.

### 2. `crates/sdlc-server/src/state.rs`

Add four new `SseMessage` variants:

```rust
/// A streaming tool run has started.
ToolRunStarted { name: String, interaction_id: String },
/// A single NDJSON progress line from a streaming tool.
ToolRunProgress { name: String, interaction_id: String, line: serde_json::Value },
/// A streaming tool run completed successfully.
ToolRunCompleted { name: String, interaction_id: String },
/// A streaming tool run failed.
ToolRunFailed { name: String, interaction_id: String, error: String },
```

### 3. `crates/sdlc-server/src/routes/events.rs`

Add match arms for the four new variants. All four are emitted on the `"tool"` SSE event channel:

```rust
Ok(SseMessage::ToolRunStarted { name, interaction_id }) => {
    let data = serde_json::json!({
        "type": "tool_run_started",
        "name": name,
        "interaction_id": interaction_id,
    }).to_string();
    Some(Ok(Event::default().event("tool").data(data)))
}
// ... similar for ToolRunProgress, ToolRunCompleted, ToolRunFailed
```

### 4. `crates/sdlc-server/src/routes/tools.rs`

Modify `run_tool` handler to branch on `meta.streaming`:

```
1. Fetch --meta, parse ToolMeta (existing logic)
2. Resolve secrets (existing logic)
3. if meta.streaming != Some(true):
     → existing spawn_blocking path, HTTP 200
4. else:
     a. Create ToolInteractionRecord (status: "streaming", streaming_log: true)
     b. Save record (best-effort)
     c. Emit ToolRunStarted SSE
     d. tokio::spawn {
          - call run_tool_streaming(...)
          - open .log sidecar file for append
          - loop: read stdout line by line
              - parse as JSON (skip invalid lines)
              - append raw line to .log sidecar
              - emit ToolRunProgress SSE
          - on exit: parse last line as ToolResult
          - update record status, save record
          - emit ToolRunCompleted or ToolRunFailed
        }
     e. HTTP 202 { "job_id": interaction_id, "streaming": true }
```

Key implementation details:
- The spawned task uses `tokio::io::BufReader::lines()` to read stdout line-by-line without buffering the full output.
- The `.log` sidecar is opened with `tokio::fs::OpenOptions::new().append(true).create(true)` — each line is written with a trailing newline.
- The final `ToolResult` line is identified as the last valid JSON line after the process exits (or the last line with `"ok"` key). The record `result` field is set from it.
- If no valid `ToolResult` is found, the record is marked `failed` with an appropriate error.

### 5. `crates/sdlc-core/src/tool_interaction.rs`

Add `load_streaming_log` helper:

```rust
pub fn load_streaming_log(root: &Path, tool_name: &str, id: &str) -> Result<Vec<serde_json::Value>>
```

Reads `.sdlc/tool-interactions/<name>/<id>.log`, parses each line as JSON, returns valid entries. Used by the GET interaction endpoint to include log data when `streaming_log: true`.

## Frontend Changes

### `frontend/src/api/client.ts`

Extend `runTool` to return the full response shape:

```typescript
type RunToolResponse =
  | { streaming: false; result: ToolResult }
  | { streaming: true; job_id: string }
```

### `frontend/src/lib/types.ts`

Add `ToolRunProgressEvent` type and extend `SseToolEvent` union.

### Tool run panel (existing component)

When `runTool` returns `streaming: true`:
1. Subscribe to SSE events filtered by `tool_run_*` type and matching `interaction_id`.
2. Accumulate `ToolRunProgress` lines into a log array (state).
3. Render each line as a monospace log entry (no special parsing — render `JSON.stringify(line, null, 2)` or a `message` field if present).
4. On `ToolRunCompleted`: stop SSE subscription, fetch the full interaction record and render the final result.
5. On `ToolRunFailed`: stop subscription, show error.

## Sidecar File Format

`.sdlc/tool-interactions/<name>/<id>.log`:
```
{"type":"progress","message":"Scanning files...","pct":10}
{"type":"progress","message":"Found 3 issues","pct":80}
{"ok":true,"data":{"issues":3},"duration_ms":2100}
```

One JSON object per line, raw NDJSON. No framing or envelope — identical to what the tool writes to stdout.

## Sequence Diagram

```
Frontend          Server (Rust)          Tool (TS process)
   |                   |                        |
   |--POST /run------->|                        |
   |                   |--spawn tool----------->|
   |                   |<--job_id (202)------   |
   |<--202 { job_id }--|                        |
   |                   |                        |
   |--SSE subscribe--->|   {"type":"progress"}  |
   |                   |<-stdout line-----------|
   |<--ToolRunProgress-|                        |
   |                   |   {"type":"progress"}  |
   |                   |<-stdout line-----------|
   |<--ToolRunProgress-|                        |
   |                   |   {"ok":true,"data":{}}|
   |                   |<-stdout (final line)---|
   |                   |<-process exit----------|
   |<--ToolRunCompleted|                        |
```

## Error Handling

| Condition | Behavior |
|---|---|
| Tool exits non-zero | `status: "failed"`, emit `ToolRunFailed` with stderr excerpt |
| No valid JSON lines on stdout | `status: "failed"`, emit `ToolRunFailed` with "no output" |
| Line is not valid JSON | Drop line silently, log at DEBUG level |
| SSE channel full (subscriber slow) | `ToolRunProgress` events are lagged/dropped — the `.log` sidecar is the authoritative record |
| Tool runtime not found | Return 503 synchronously before spawning (same as non-streaming path) |
| Missing required secrets | Return 422 synchronously before spawning (same as non-streaming path) |

## Backward Compatibility

- `ToolMeta.streaming` is `Option<bool>` and defaults to `None` — existing tools see no behavior change.
- `ToolInteractionRecord.streaming_log` is already in the struct (defaulting to `false`) — no schema migration.
- HTTP 200 / 202 distinction: callers that ignore the status code still receive JSON with `ok`/`data` keys for non-streaming tools. Streaming tool responses contain `job_id` and `streaming: true` — callers must opt in by checking these fields.

## Test Plan (unit)

1. `run_tool_streaming` spawns a child and returns a piped stdout handle.
2. Streaming route: given a `meta.streaming = true` tool, returns HTTP 202 with a `job_id`.
3. Streaming route: given a `meta.streaming = false` tool, returns HTTP 200 with result (unchanged).
4. Sidecar append: each stdout line appears in the `.log` file after the task completes.
5. `load_streaming_log` returns parsed lines from a `.log` file.
6. Invalid JSON lines on stdout are skipped without crashing.
