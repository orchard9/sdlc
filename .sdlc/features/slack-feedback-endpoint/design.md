# Design: slack-feedback-endpoint

## Architecture

This is a backend-only feature. No UI changes required — the created feedback threads appear in the existing threads UI automatically.

```
Slack Bot (TypeScript)
    |
    | POST /api/feedback/slack
    | Content-Type: application/json
    | { source, channel_id, channel_name, user_name, text, context_messages, ... }
    v
sdlc-server (Rust/Axum)
    |
    | routes/feedback.rs :: receive_slack_feedback()
    |   1. Validate payload
    |   2. Dedup check (message_ts -> existing thread lookup)
    |   3. Render context_messages into markdown body
    |   4. create_thread(context="slack:{channel}", title, body)
    |   5. add_post(thread_id, user_name, text)
    |   6. Return 201 + thread JSON
    v
.sdlc/threads/{id}/
    thread.yaml  (metadata)
    posts/       (individual post files)
```

## Request/Response Flow

### Happy Path

```
POST /api/feedback/slack HTTP/1.1
Content-Type: application/json

{
  "source": "slack",
  "channel_name": "sdlc-bugs",
  "user_name": "jordan",
  "text": "login button is broken on mobile",
  "message_ts": "1234567890.123456",
  "context_messages": [
    { "user_name": "alice", "text": "same issue here", "ts": "1234567880.000" }
  ]
}

HTTP/1.1 201 Created
{
  "id": "T-20260305-001",
  "context": "slack:sdlc-bugs",
  "title": "login button is broken on mobile (via Slack)",
  "body": "## Conversation Context\n\n**alice**: same issue here\n\n---\n\n**jordan**: login button is broken on mobile",
  "status": "open",
  "comment_count": 1,
  "created_at": "...",
  "updated_at": "..."
}
```

### Duplicate Detection

```
POST /api/feedback/slack  (same message_ts as above)

HTTP/1.1 409 Conflict
{ "error": "duplicate", "existing_thread_id": "T-20260305-001" }
```

## Data Model

### SlackFeedbackPayload (deserialization struct)

```rust
#[derive(Deserialize)]
pub struct SlackFeedbackPayload {
    pub source: String,                         // must be "slack"
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub user_id: Option<String>,
    pub user_name: String,
    pub text: String,
    pub message_ts: Option<String>,
    pub thread_ts: Option<String>,
    pub context_messages: Option<Vec<SlackContextMessage>>,
}

#[derive(Deserialize)]
pub struct SlackContextMessage {
    pub user_name: String,
    pub text: String,
    pub ts: Option<String>,
}
```

## Dedup Strategy

The dedup key is `message_ts` from Slack. On receipt:

1. If `message_ts` is `None`, skip dedup — always create a new thread.
2. If `message_ts` is `Some(ts)`, scan existing threads with `context` starting with `"slack:"` and check if the thread body contains a dedup marker `<!-- slack:message_ts={ts} -->`.
3. If found, return 409 with the existing thread ID.

The dedup marker is embedded as an HTML comment at the top of the thread body. This avoids needing a new index or database column — it piggybacks on the existing thread storage.

## Context Markdown Rendering

```rust
fn render_context_markdown(
    context_messages: &[SlackContextMessage],
    trigger_user: &str,
    trigger_text: &str,
) -> String {
    let mut md = String::new();
    if !context_messages.is_empty() {
        md.push_str("## Conversation Context\n\n");
        for msg in context_messages {
            md.push_str(&format!("**{}**: {}\n\n", msg.user_name, msg.text));
        }
        md.push_str("---\n\n");
    }
    md.push_str(&format!("**{}**: {}", trigger_user, trigger_text));
    md
}
```

## Module Placement

All new code goes in `crates/sdlc-server/src/routes/feedback.rs`, adding to the existing module. The route is registered in `lib.rs` alongside the existing `/api/feedback` routes.

## Dependencies

No new crate dependencies. Uses existing:
- `axum` (routing, JSON extraction)
- `sdlc_core::feedback_thread` (create_thread, add_post, list_threads)
- `serde` (deserialization)
