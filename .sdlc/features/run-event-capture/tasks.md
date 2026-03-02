# Tasks: run-event-capture

- [ ] Extend `Message::User` arm in `message_to_event()` — extract `ToolResult` blocks from `UserContentBlock::ToolResult`, emit `{"type":"user","tool_results":[...]}` with content truncated at 2000 chars
- [ ] Add `SystemPayload::TaskStarted` arm — emit `{"type":"subagent_started","task_id","tool_use_id","description"}`
- [ ] Add `SystemPayload::TaskProgress` arm — emit `{"type":"subagent_progress","task_id","last_tool_name","total_tokens","tool_uses","duration_ms"}`
- [ ] Add `SystemPayload::TaskNotification` arm — emit `{"type":"subagent_completed","task_id","status","summary","total_tokens","duration_ms"}`
- [ ] Add `ContentBlock::Thinking` arm in assistant content match — emit `{"type":"thinking","thinking":thinking}`
- [ ] Add `pub prompt: Option<String>` to `RunRecord` struct in `state.rs`; set it from `spawn_agent_run()` prompt parameter before writing initial record
