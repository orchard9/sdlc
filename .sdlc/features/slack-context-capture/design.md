# Design: Context-Windowed Capture

## Module Location

```
.sdlc/tools/slack-bot/
├── context.ts          # This feature — context capture module
├── bot.ts              # Bot runtime (slack-bot-core feature)
├── config.yaml         # Project routing config
└── types.ts            # Shared types
```

## Data Flow

```
@mention event
    │
    ▼
bot.ts receives app_mention event
    │
    ├── extracts channel_id, trigger_ts, user from event
    │
    ▼
captureContext(request)          ◄── context.ts entry point
    │
    ├── 1. fetchHistory(channel_id, limit)
    │       └── Slack conversations.history → Message[]
    │
    ├── 2. filterMessages(messages, trigger_ts, bot_user_id)
    │       └── Remove bots, trigger, system messages
    │
    ├── 3. messages.reverse()
    │       └── Newest-first → oldest-first (chronological)
    │
    ├── 4. resolveUsers(messages, bot_token)
    │       └── Collect unique <@U...> IDs
    │       └── Batch users.info calls → Map<string, string>
    │       └── Cache within this capture invocation
    │
    ├── 5. renderMarkdown(messages, userMap)
    │       └── Format each message: **name** (HH:MM): text
    │       └── Replace <@U...> with resolved names in text
    │       └── Join under heading
    │
    └── 6. Return CaptureResult { context, count, resolved_users }
            │
            ▼
        bot.ts merges into normalized payload
            │
            ▼
        POST to project webhook URL
```

## Key Types

```typescript
// Internal — Slack API response shape (subset)
interface SlackMessage {
  type: string
  subtype?: string
  user?: string          // User ID (absent for bot messages without user field)
  bot_id?: string        // Present for bot messages
  text: string
  ts: string             // Message timestamp (unique ID)
  attachments?: unknown[]
  blocks?: unknown[]
}

// User resolution cache
type UserCache = Map<string, string>  // user_id → display_name

// Public interface (from spec)
interface CaptureRequest {
  channel_id: string
  trigger_ts: string
  bot_user_id: string
  bot_token: string
  context_messages: number
}

interface CaptureResult {
  context: string
  context_message_count: number
  resolved_users: number
}
```

## Function Breakdown

### `captureContext(req: CaptureRequest): Promise<CaptureResult>`

Orchestrator function. Calls the pipeline steps in sequence and returns the result.

### `fetchHistory(token: string, channel: string, limit: number): Promise<SlackMessage[]>`

Calls `conversations.history` with:
- `channel`: the channel ID
- `limit`: from config (default 100)
- `inclusive`: false (exclude exact boundary)

Returns raw Slack message array (newest first).

Error handling: If the API returns an error (e.g., `channel_not_found`, `not_in_channel`), throw with the Slack error code. The caller (bot.ts) handles this by replying to the user.

### `filterMessages(messages: SlackMessage[], triggerTs: string, botUserId: string): SlackMessage[]`

Filters out:
1. Messages where `ts === triggerTs` (the trigger mention)
2. Messages where `user === botUserId` (bot's own messages)
3. Messages where `bot_id` is present (other bots)
4. Messages with `subtype` in `SYSTEM_SUBTYPES` set:
   - `channel_join`, `channel_leave`, `channel_topic`, `channel_purpose`,
     `channel_name`, `channel_archive`, `channel_unarchive`

Returns filtered array (still newest-first at this point).

### `resolveUsers(messages: SlackMessage[], token: string): Promise<UserCache>`

1. Collect all unique user IDs:
   - From `message.user` field
   - From `<@U...>` patterns in `message.text` via regex `/(<@(U[A-Z0-9]+)>)/g`
2. Deduplicate into a Set
3. For each user ID, call `users.info` with the bot token
4. Extract `display_name` → fallback `real_name` → fallback `name` → fallback raw ID
5. Return `Map<user_id, display_name>`

On per-user failure: log to stderr, map the user to their raw `<@U...>` format.

### `renderMarkdown(messages: SlackMessage[], userMap: UserCache): string`

1. For each message (already in chronological order after reverse):
   - Look up `message.user` in `userMap` for the author name
   - Parse `message.ts` to extract time: `HH:MM` format (Unix timestamp → Date)
   - Replace all `<@U...>` references in `message.text` with resolved names
   - Handle Slack mrkdwn → standard markdown:
     - `<@U123>` → `@display_name`
     - `<#C123|channel-name>` → `#channel-name`
     - `<http://url|label>` → `[label](http://url)`
     - `<http://url>` → `http://url`
   - Note attachments: if `message.attachments?.length`, append ` [attachment]`
   - Note rich blocks without text: if no text but has blocks, use `[rich content]`
2. Format: `**author_name** (HH:MM): processed_text`
3. Join all lines with `\n`
4. Prepend heading: `## Channel Context (last N messages)\n\n`
5. Return the full markdown string

### `slackMrkdwnToMarkdown(text: string, userMap: UserCache): string`

Converts Slack-specific markup to standard markdown:
- User mentions: `<@U123>` → `@resolved_name`
- Channel links: `<#C123|name>` → `#name`
- URL links: `<url|label>` → `[label](url)`
- Bare URLs: `<url>` → `url`
- Bold: already `*text*` in Slack, same in markdown
- Italic: `_text_` same in both
- Strikethrough: `~text~` → `~~text~~`
- Code: backticks same in both

## Error Handling Strategy

| Error | Handling |
|---|---|
| `conversations.history` fails | Throw — bot.ts catches and replies to user with error |
| Individual `users.info` fails | Log to stderr, use raw `<@U...>` in output |
| Empty channel (no messages) | Return `{ context: "", context_message_count: 0, resolved_users: 0 }` |
| All messages filtered out | Same as empty channel |
| Rate limit (429) | Not expected for v1 volumes; if hit, Slack SDK auto-retries |

## No Persistent State

This module is stateless — no caching between invocations, no database, no files. The user cache lives only for the duration of a single `captureContext` call.
