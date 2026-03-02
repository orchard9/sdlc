# Review: run-event-capture

## Summary

All six changes specified in the spec and design are implemented correctly. The implementation is purely additive — no existing behavior is changed, no new dependencies introduced.

## Changes Verified

### 1. `Message::User` arm — ToolResult extraction (runs.rs)

Replaced the stub `serde_json::json!({"type": "user"})` with extraction of all `UserContentBlock::ToolResult` blocks. Each result carries `tool_use_id`, `is_error` (with `unwrap_or(false)` default), and text content truncated at 2000 chars. The closure uses `.map().next()` (not `find_map`) to satisfy clippy's `unnecessary_find_map` lint.

### 2. `SystemPayload::TaskStarted` arm (runs.rs)

Emits `{"type":"subagent_started","task_id","tool_use_id","description"}`. Field names match `TaskStartedPayload` exactly (`tool_use_id: Option<String>` — serializes as null when absent).

### 3. `SystemPayload::TaskProgress` arm (runs.rs)

Emits `{"type":"subagent_progress","task_id","last_tool_name","total_tokens","tool_uses","duration_ms"}`. `last_tool_name` is `Option<String>` on `TaskProgressPayload`, serializes as null when absent. Usage fields come from the embedded `TaskUsage` struct.

### 4. `SystemPayload::TaskNotification` arm (runs.rs)

Emits `{"type":"subagent_completed","task_id","status","summary","total_tokens","duration_ms"}`. `usage` is `Option<TaskUsage>` — both token/duration fields use `.as_ref().map(|u| u.field)` to produce null when absent.

### 5. `ContentBlock::Thinking` arm (runs.rs)

Added to the assistant branch as a collected `thinking` array on the emitted JSON object alongside existing `text` and `tools`. Consistent with how other content block arrays are handled.

### 6. `RunRecord.prompt` field (state.rs)

`pub prompt: Option<String>` added to `RunRecord`. Set from `prompt.clone()` in `spawn_agent_run()` before the initial persist. Existing records deserialize with `null` (backward compatible).

## Quality

- Build: clean, no warnings
- Tests: 473 passed, 0 failed
- Clippy: clean with `-D warnings`
- No `unwrap()` in new code — all optional access uses `and_then`/`map`/`unwrap_or`
- All file writes go through existing `persist_run`/`persist_run_events` infrastructure
