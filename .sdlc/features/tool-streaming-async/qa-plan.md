# QA Plan: Streaming + Async Tool Execution

## Scope

This plan covers functional, regression, and edge-case testing for the `tool-streaming-async` feature. Tests are grouped by layer.

---

## 1. Unit Tests (Rust — run with `SDLC_NO_NPM=1 cargo test --all`)

### 1.1 `tool_runner::run_tool_streaming`

| # | Test | Pass Condition |
|---|------|---------------|
| U1 | `run_tool_streaming` with no runtime available returns `SdlcError::NoToolRuntime` | Error variant matches |
| U2 | `run_tool_streaming` with a trivial `bun`/`node` echo script returns a `Child` with piped stdout | `child.stdout.is_some()` |

### 1.2 `tool_interaction::load_streaming_log`

| # | Test | Pass Condition |
|---|------|---------------|
| U3 | File with 3 valid NDJSON lines returns 3 parsed values | `len() == 3` |
| U4 | File with one invalid JSON line + two valid lines returns 2 parsed values (invalid line dropped) | `len() == 2` |
| U5 | Missing `.log` file returns empty `Vec` (no error) | `Ok(vec![])` |

### 1.3 `routes/tools.rs` (Axum handler tests)

| # | Test | Pass Condition |
|---|------|---------------|
| U6 | Non-streaming tool: `run_tool` still returns HTTP 200 with `ToolResult` JSON | Status 200, body contains `ok` key |
| U7 | Streaming tool meta (`streaming: true`): handler returns HTTP 202 with `{ "job_id": "...", "streaming": true }` | Status 202, body contains `job_id` |
| U8 | Streaming tool: `ToolInteractionRecord` is created with `status: "streaming"` and `streaming_log: true` before HTTP response | Record exists in interactions dir |
| U9 | Invalid JSON line on stdout is skipped without crashing | Handler does not error; log file has fewer entries than raw stdout lines |

---

## 2. Integration Tests (Rust)

| # | Test | Pass Condition |
|---|------|---------------|
| I1 | Full streaming run: script writes 3 NDJSON lines then a final `ToolResult` — record ends with `status: "completed"` and `result` set | Record deserialization succeeds, `result.ok == true` |
| I2 | Streaming script exits non-zero with no valid output — record status is `"failed"` and `error` is set | `status == "failed"` |
| I3 | `.log` sidecar file contains all raw lines written by the script | File line count matches script output count |

---

## 3. SSE Event Tests

| # | Test | Pass Condition |
|---|------|---------------|
| S1 | `ToolRunStarted` is emitted before any `ToolRunProgress` events | SSE event order is correct |
| S2 | Each NDJSON line produces exactly one `ToolRunProgress` SSE event | Event count matches line count |
| S3 | `ToolRunCompleted` is emitted after the last `ToolRunProgress` event and after the record is finalized | SSE event order correct, record saved |
| S4 | `ToolRunFailed` is emitted when the process exits non-zero | SSE event type is `tool_run_failed` |

---

## 4. Frontend Tests (manual / Playwright)

| # | Test | Pass Condition |
|---|------|---------------|
| F1 | Click Run on a streaming tool — spinner appears immediately, result panel shows log lines as they stream in | No full-page block; lines appear progressively |
| F2 | After completion, final result card renders (same as non-streaming tool) | Result card visible |
| F3 | Click Run on a non-streaming tool — behavior unchanged (synchronous, HTTP 200) | Result appears immediately after run |
| F4 | Streaming tool failure — error state renders in the tool run panel | Error message visible |

---

## 5. Regression Tests

| # | Test | Pass Condition |
|---|------|---------------|
| R1 | `SDLC_NO_NPM=1 cargo test --all` passes with zero failures | All tests pass |
| R2 | `cargo clippy --all -- -D warnings` passes with zero warnings | Clean clippy output |
| R3 | `GET /api/tools` — existing tools without `streaming` key continue to appear in the list | Response JSON unchanged for non-streaming tools |
| R4 | `GET /api/tools/:name/interactions/:id` — existing non-streaming interaction records load correctly | `streaming_log: false`, no `.log` sidecar required |

---

## 6. Edge Cases

| # | Test | Pass Condition |
|---|------|---------------|
| E1 | Streaming tool with no stdout output (silent tool) | Record marked `failed`, no sidecar created |
| E2 | Tool writes stdout faster than SSE subscribers can consume | Events are dropped per broadcast semantics; `.log` sidecar still complete |
| E3 | Concurrent runs of the same streaming tool | Each run gets its own `interaction_id` and separate `.log` sidecar |
| E4 | Very large NDJSON line (> 64KB) | Line is read correctly (no line-length limit in `BufReader::lines`) |
