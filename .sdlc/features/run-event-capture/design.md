# Design: run-event-capture

## Architecture

Purely additive changes to two existing files. No new abstractions, no new modules.

### `message_to_event()` extension

The function is a `match` over `Message` variants. Three sites need changes:

**1. `Message::User` arm** (currently: `{"type":"user"}`)

Replace the stub with actual extraction of `ToolResult` blocks. The `UserContentBlock::ToolResult` variant carries `tool_use_id: String`, `content: Option<Vec<ToolResultContent>>`, `is_error: Option<bool>`. Extract text content from the `ToolResultContent::Text` variant. Truncate at 2000 chars to keep events sidecar file sizes reasonable (full content already captured in the raw stream).

Emit a single `{"type":"user", "tool_results": [...]}` event with the array of extracted results.

**2. `SystemPayload` match** — add three new arms before `Unknown`

`TaskStarted` → `{"type":"subagent_started", "task_id", "tool_use_id", "description"}`
`TaskProgress` → `{"type":"subagent_progress", "task_id", "last_tool_name", "total_tokens", "tool_uses", "duration_ms"}`
`TaskNotification` → `{"type":"subagent_completed", "task_id", "status", "summary", "total_tokens", "duration_ms"}`

**3. `ContentBlock` match inside `Message::Assistant`** — add `Thinking` arm

Currently only `Text` and `ToolUse` are matched; anything else is skipped. Add:
`ContentBlock::Thinking { thinking }` → `{"type":"thinking", "thinking": thinking}`

### `RunRecord.prompt` field

Add `pub prompt: Option<String>` to the `RunRecord` struct. In `spawn_agent_run()`, set this field on the initial record before writing it to disk. No migration needed — `Option<String>` deserializes as `null` for existing records.

## Data shape changes

The existing events sidecar (`.sdlc/.runs/{id}.events.json`) gains new event objects. All existing event shapes are unchanged. New event types:

```
{"type":"user", "tool_results": [{"type":"tool_result", "tool_use_id":"...", "is_error":false, "content":"..."}]}
{"type":"subagent_started", "task_id":"...", "tool_use_id":"...", "description":"..."}
{"type":"subagent_progress", "task_id":"...", "last_tool_name":"Edit", "total_tokens":8420, ...}
{"type":"subagent_completed", "task_id":"...", "status":"success", "summary":"...", ...}
{"type":"thinking", "thinking":"..."}
```

## Error handling

All new extraction is via `filter_map` / `and_then` — no `unwrap()`. Missing optional fields produce `null` JSON values. The `Unknown` arm still catches any unrecognized `SystemPayload` variants.
