# QA Plan: run-event-capture

## Unit tests

1. `message_to_event` with a `Message::User` containing a `ToolResult` block — assert the output JSON has `type: "user"` and `tool_results` array with correct `tool_use_id`, `is_error`, and `content`
2. `message_to_event` with a `Message::User` with no `ToolResult` blocks — assert `tool_results` is an empty array
3. `message_to_event` with `Message::System(SystemPayload::TaskStarted(...))` — assert `type: "subagent_started"` with correct fields
4. `message_to_event` with `Message::System(SystemPayload::TaskProgress(...))` — assert `type: "subagent_progress"`
5. `message_to_event` with `Message::System(SystemPayload::TaskNotification(...))` — assert `type: "subagent_completed"`
6. `message_to_event` with a `ContentBlock::Thinking` in assistant content — assert `type: "thinking"` event

## Integration check

- Start a real agent run; read the resulting `.sdlc/.runs/{id}.events.json`; assert it contains at least one `tool_result` event and the `RunRecord` has a non-null `prompt`
