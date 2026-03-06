# Spec: Context-Windowed Capture

## Summary

Implement the context-windowed capture module for the Slack bot tool. When the bot receives an @mention or DM, this module reads the last N messages from the Slack channel via `conversations.history`, filters out bot messages and the trigger mention itself, resolves user IDs to display names via `users.info`, renders the conversation as structured markdown, and produces an enriched payload ready for POST to the target webhook.

## Problem

When a user @mentions the Slack bot to report a bug or request, the trigger message alone lacks context. The surrounding conversation — who said what, what was discussed — is critical for the receiving project to understand and act on the feedback. Without context capture, every feedback thread requires manual copy-paste of relevant messages.

## Solution

A TypeScript module (`context.ts`) in `.sdlc/tools/slack-bot/` that:

1. **Fetches channel history** — Calls Slack `conversations.history` with `limit` set to the configured `context_messages` count (default: 100). Uses the bot token (`SLACK_BOT_TOKEN`).

2. **Filters messages** — Removes:
   - Bot messages (`subtype === 'bot_message'` or message from bot's own user ID)
   - The trigger @mention message itself (matched by `message_ts`)
   - System/join/leave messages (`subtype` in `['channel_join', 'channel_leave', 'channel_topic', 'channel_purpose']`)

3. **Reverses to chronological order** — Slack API returns newest-first; reverse to oldest-first for readable context.

4. **Resolves user IDs** — For each unique `<@U...>` user ID in messages, calls `users.info` to get `display_name` or `real_name`. Caches resolved names for the duration of the capture to avoid redundant API calls.

5. **Renders as markdown** — Each message becomes:
   ```
   **display_name** (HH:MM): message text with @mentions replaced by display names
   ```
   Grouped under a heading: `## Channel Context (last N messages)`

6. **Produces enriched payload** — Adds two fields to the normalized payload:
   - `context`: the rendered markdown string
   - `context_message_count`: integer count of messages included

## Interfaces

### Input
```typescript
interface CaptureRequest {
  channel_id: string       // Slack channel ID
  trigger_ts: string       // message_ts of the @mention that triggered capture
  bot_user_id: string      // Bot's own Slack user ID (to filter out)
  bot_token: string        // xoxb-... token for API calls
  context_messages: number // Max messages to fetch (from config, default 100)
}
```

### Output
```typescript
interface CaptureResult {
  context: string              // Rendered markdown
  context_message_count: number // How many messages were included after filtering
  resolved_users: number       // How many unique users were resolved
}
```

### Exported Function
```typescript
export async function captureContext(req: CaptureRequest): Promise<CaptureResult>
```

## Slack API Calls

| Endpoint | Purpose | Rate Limit Tier |
|---|---|---|
| `conversations.history` | Fetch channel messages | Tier 3 (50+/min) |
| `users.info` | Resolve user ID → name | Tier 4 (100+/min) |

Rate limits are not a concern for v1 — a single capture makes 1 history call + N user lookups (where N is unique users, typically < 20).

## Edge Cases

- **Empty channel**: Return empty context string, `context_message_count: 0`
- **Bot-only channel**: After filtering, if no messages remain, return empty context
- **Private channel**: `groups:history` scope required; same API call works transparently
- **User lookup failure**: If `users.info` fails for a user, fall back to the raw `<@U...>` format
- **Message with attachments/files**: Include the `text` field only; attachments are noted as `[attachment]`
- **Thread replies**: Only top-level channel messages are captured (not thread replies)
- **Messages with blocks**: Extract `text` fallback from blocks; if no text, note `[rich content]`

## Non-Goals

- Thread reply capture (v2)
- Image/file content extraction
- Message reactions or emoji handling
- Pagination beyond the configured limit (single API call)
- Persistent caching of user names across captures

## Dependencies

- `slack-bot-core` feature provides the bot runtime and config loading
- Slack Web API (`@slack/web-api` package, included with `@slack/bolt`)

## Acceptance Criteria

1. Given a channel with 50 human messages and 10 bot messages, capture returns exactly 50 messages in chronological order
2. User IDs in message text (`<@U123>`) are replaced with resolved display names
3. The trigger message (matched by `message_ts`) is excluded from context
4. System messages (join/leave/topic) are excluded
5. Output markdown follows the format: `**name** (HH:MM): text`
6. When user lookup fails, raw `<@U...>` format is preserved (no crash)
7. Empty channels produce `context_message_count: 0` with empty context string
