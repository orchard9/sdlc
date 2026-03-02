# Spec: run-event-capture

## Problem

`message_to_event()` in `runs.rs` discards most of what the claude-agent stream delivers. Five categories of data flow through the stream today and are silently dropped:

| Stream data | Where it lives | Current handling |
|---|---|---|
| Tool results (what the tool returned) | `Message::User` → `UserContentBlock::ToolResult` | Emitted as `{"type":"user"}`, content dropped |
| Subagent spawned | `SystemPayload::TaskStarted` | Falls into `Unknown` arm, dropped |
| Subagent progress | `SystemPayload::TaskProgress` | Falls into `Unknown` arm, dropped |
| Subagent completed | `SystemPayload::TaskNotification` | Falls into `Unknown` arm, dropped |
| Thinking blocks | `ContentBlock::Thinking` | Not handled in match, dropped |

Additionally, the initial prompt is never persisted. `spawn_agent_run(prompt, ...)` passes the prompt to `query()` but never stores it. `RunRecord` has no `prompt` field. A run with no prompt stored is unreadable — you can see what tools were called but not what was asked.

## Solution

### 1. Extend `message_to_event()`

**`Message::User`** — extract `ToolResult` blocks:

```rust
Message::User(user) => {
    let tool_results: Vec<serde_json::Value> = user.message.content.iter()
        .filter_map(|c| {
            if let UserContentBlock::ToolResult { tool_use_id, content, is_error } = c {
                let text = content.as_ref()
                    .and_then(|blocks| blocks.iter().find_map(|b| {
                        if let ToolResultContent::Text { text } = b { Some(text.as_str()) } else { None }
                    }))
                    .unwrap_or("");
                Some(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": tool_use_id,
                    "is_error": is_error.unwrap_or(false),
                    "content": &text[..text.len().min(2000)]
                }))
            } else { None }
        })
        .collect();
    serde_json::json!({"type": "user", "tool_results": tool_results})
}
```

**`SystemPayload::TaskStarted/TaskProgress/TaskNotification`** — add arms in the `System` match:

```rust
SystemPayload::TaskStarted(t) => serde_json::json!({
    "type": "subagent_started",
    "task_id": t.task_id,
    "tool_use_id": t.tool_use_id,
    "description": t.description,
}),
SystemPayload::TaskProgress(t) => serde_json::json!({
    "type": "subagent_progress",
    "task_id": t.task_id,
    "last_tool_name": t.last_tool_name,
    "total_tokens": t.usage.total_tokens,
    "tool_uses": t.usage.tool_uses,
    "duration_ms": t.usage.duration_ms,
}),
SystemPayload::TaskNotification(t) => serde_json::json!({
    "type": "subagent_completed",
    "task_id": t.task_id,
    "status": t.status,
    "summary": t.summary,
    "total_tokens": t.usage.as_ref().map(|u| u.total_tokens),
    "duration_ms": t.usage.as_ref().map(|u| u.duration_ms),
}),
```

**`ContentBlock::Thinking`** — add arm in the Assistant content match:

```rust
ContentBlock::Thinking { thinking } => {
    serde_json::json!({"type": "thinking", "thinking": thinking})
}
```

### 2. Add `prompt` to `RunRecord`

Add `pub prompt: Option<String>` to the `RunRecord` struct in `state.rs`. In `spawn_agent_run()`, set `record.prompt = Some(prompt.clone())` before writing the initial RunRecord.

## Scope

- Files touched: `crates/sdlc-server/src/routes/runs.rs`, `crates/sdlc-server/src/state.rs`
- No new dependencies
- No API changes (events sidecar format extends backward-compatibly: new event types added, no existing events changed)
- No frontend changes in this feature

## Out of Scope

- Storage backend (Feature: run-events-api)
- UI rendering (Feature: run-activity-ui)
- `TaskNotification.output_file` — not captured (path to subagent output file; add later if needed)
