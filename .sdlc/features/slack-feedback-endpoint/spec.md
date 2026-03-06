# Spec: slack-feedback-endpoint

## Summary

Add a `POST /api/feedback/slack` endpoint to sdlc-server that receives a normalized Slack payload from the Slack bot tool and creates a feedback thread with full conversation context. This endpoint is the server-side receiver that completes the Slack bot's routing pipeline: bot captures message -> bot resolves context -> bot POSTs normalized payload -> **this endpoint** creates the feedback thread.

## Motivation

The v39-slack-bot-tool milestone enables team members to @mention a Slack bot to create pre-contextualized feedback threads in sdlc projects. The bot (TypeScript/@slack/bolt) routes messages to configured project webhooks. This feature provides the receiving endpoint that the bot POSTs to. Without it, the bot has no target to deliver Slack-captured feedback.

## Normalized Payload Schema

The bot sends a JSON payload with the following shape:

```json
{
  "source": "slack",
  "channel_id": "C0123456789",
  "channel_name": "sdlc-bugs",
  "user_id": "U0123456789",
  "user_name": "jordan",
  "text": "login button is broken on mobile",
  "message_ts": "1234567890.123456",
  "thread_ts": null,
  "context_messages": [
    {
      "user_name": "alice",
      "text": "I noticed the same thing yesterday",
      "ts": "1234567880.123456"
    }
  ]
}
```

### Required fields
- `source` — must be `"slack"`
- `text` — the triggering message content
- `user_name` — who sent the message

### Optional fields
- `channel_id`, `channel_name` — Slack channel metadata
- `user_id` — Slack user ID
- `message_ts` — Slack message timestamp (used as dedup key)
- `thread_ts` — if the message was in a Slack thread
- `context_messages` — array of preceding messages the bot captured (up to 100)

## Behavior

1. **Validate** the payload: `source` must be `"slack"`, `text` must be non-empty, `user_name` must be non-empty.

2. **Dedup check**: If `message_ts` is provided, check whether a feedback thread with matching `slack:message_ts` metadata already exists. If so, return 409 Conflict with the existing thread ID.

3. **Build context markdown**: Render `context_messages` (if present) into a markdown block prefixed with `## Conversation Context`, showing each message as `**user_name** (ts): text`. This becomes the thread body.

4. **Create feedback thread**: Call `sdlc_core::feedback_thread::create_thread` with:
   - `context`: `"slack:{channel_name}"` (or `"slack:dm"` if no channel)
   - `title`: First 120 chars of `text`, suffixed with ` (via Slack)` if space allows
   - `body`: The rendered context markdown, with the triggering message appended at the end

5. **Add first post**: Add a post to the thread with author=`user_name` and content=`text`.

6. **Return 201 Created** with the thread JSON (same shape as `POST /api/threads`).

## Route Registration

- Path: `POST /api/feedback/slack`
- No auth required beyond the standard tunnel auth middleware (the bot authenticates via the same token/cookie gate as any other API client)
- Handler lives in `crates/sdlc-server/src/routes/feedback.rs` alongside existing feedback routes

## Error Responses

| Condition | Status | Body |
|---|---|---|
| Missing/empty `text` | 400 | `{ "error": "text is required" }` |
| Missing/empty `user_name` | 400 | `{ "error": "user_name is required" }` |
| `source` not `"slack"` | 400 | `{ "error": "source must be 'slack'" }` |
| Duplicate `message_ts` | 409 | `{ "error": "duplicate", "existing_thread_id": "..." }` |

## Non-Goals

- Slack signature verification (HMAC-SHA256) — that is the bot's responsibility, not the server's. The bot runs trusted.
- Webhook routing logic — the bot decides which project to POST to.
- Block Kit disambiguation — handled by the bot before it reaches this endpoint.
- Socket Mode / Events API — those are bot-side concerns.

## Testing

- Unit tests in `feedback.rs`: valid payload creates thread, missing fields return 400, duplicate `message_ts` returns 409
- Integration: bot can POST a realistic payload and receive a valid thread back
