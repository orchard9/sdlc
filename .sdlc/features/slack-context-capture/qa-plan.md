# QA Plan: Context-Windowed Capture

## Test Strategy

This module is a pure TypeScript library with no server dependencies — all tests use mocked Slack API responses. Tests run with the project's standard `npx tsx` runner or via direct import in a test harness.

## Test Cases

### TC1: Happy path — standard channel capture

**Setup**: Mock `conversations.history` returning 10 messages (8 human, 1 bot, 1 trigger). Mock `users.info` for 3 unique users.

**Steps**:
1. Call `captureContext` with a valid request
2. Verify result

**Expected**:
- `context_message_count` = 8 (bot + trigger filtered)
- `resolved_users` = 3
- `context` starts with `## Channel Context (last 8 messages)`
- Messages are in chronological order (oldest first)
- Each line matches `**name** (HH:MM): text` format

### TC2: Empty channel

**Setup**: Mock `conversations.history` returning empty `messages` array.

**Expected**:
- `context` = `""`
- `context_message_count` = 0
- `resolved_users` = 0

### TC3: All messages filtered (bot-only channel)

**Setup**: Mock returning 5 messages, all with `bot_id` set.

**Expected**: Same as TC2 — empty context, count 0.

### TC4: System messages filtered

**Setup**: Mock returning messages with subtypes: `channel_join`, `channel_leave`, `channel_topic`, `channel_purpose`, plus 2 normal messages.

**Expected**: `context_message_count` = 2. System messages absent from output.

### TC5: User resolution failure graceful fallback

**Setup**: Mock `users.info` to fail (HTTP 500) for one user ID, succeed for others.

**Expected**:
- No exception thrown
- Failed user appears as `<@U...>` in output
- Other users show resolved display names
- `resolved_users` counts only successfully resolved users

### TC6: Slack mrkdwn conversion

**Setup**: Message text containing:
- `<@U123>` (user mention)
- `<#C456|general>` (channel link)
- `<https://example.com|Example>` (URL with label)
- `<https://bare.com>` (bare URL)
- `~strikethrough~`

**Expected**:
- `@resolved_name` (or `<@U123>` if unresolved)
- `#general`
- `[Example](https://example.com)`
- `https://bare.com`
- `~~strikethrough~~`

### TC7: Messages with attachments

**Setup**: Message with `text: "check this"` and `attachments: [{}]`.

**Expected**: Output line includes `check this [attachment]`.

### TC8: Messages with blocks but no text

**Setup**: Message with `text: ""`, `blocks: [{ type: "rich_text" }]`.

**Expected**: Output line includes `[rich content]`.

### TC9: Trigger message excluded

**Setup**: 5 messages, one with `ts` matching `trigger_ts`.

**Expected**: `context_message_count` = 4. Trigger message text absent from output.

### TC10: Chronological ordering

**Setup**: Mock returns messages with ts values `1000.0`, `900.0`, `800.0` (newest first per Slack API).

**Expected**: Output has `800.0` message first, `1000.0` message last.

### TC11: Duplicate user IDs deduplicated

**Setup**: 5 messages from 2 unique users. One user also mentioned via `<@U...>` in another message.

**Expected**: `users.info` called exactly 2 times (not 6).

### TC12: conversations.history API error

**Setup**: Mock `conversations.history` to return `{ ok: false, error: "channel_not_found" }`.

**Expected**: `captureContext` throws an error containing `channel_not_found`.

## Verification Method

All tests are unit tests using mocked HTTP responses. No live Slack API calls. Tests validate return values and can inspect mock call counts for deduplication verification (TC11).
