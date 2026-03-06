# Tasks: Context-Windowed Capture

## T1: Create context.ts module with types and CaptureRequest/CaptureResult interfaces

Create `.sdlc/tools/slack-bot/context.ts` with the TypeScript interfaces (`SlackMessage`, `CaptureRequest`, `CaptureResult`, `UserCache`) and the exported `captureContext` function skeleton.

## T2: Implement fetchHistory — conversations.history API call

Implement `fetchHistory(token, channel, limit)` that calls the Slack `conversations.history` endpoint using `fetch()`, handles error responses (throw with Slack error code), and returns the raw message array.

## T3: Implement filterMessages — remove bots, trigger, system messages

Implement `filterMessages(messages, triggerTs, botUserId)` that filters out: the trigger message by `ts`, bot messages by `user` or `bot_id`, and system subtypes (join/leave/topic/purpose/name/archive/unarchive).

## T4: Implement resolveUsers — user ID to display name resolution

Implement `resolveUsers(messages, token)` that collects unique user IDs from message `user` fields and `<@U...>` patterns in text, calls `users.info` for each, and returns a `Map<string, string>`. Gracefully handles per-user failures by falling back to raw ID.

## T5: Implement slackMrkdwnToMarkdown — Slack markup conversion

Implement `slackMrkdwnToMarkdown(text, userMap)` that converts Slack-specific formatting: `<@U123>` to `@name`, `<#C123|name>` to `#name`, `<url|label>` to `[label](url)`, `<url>` to bare URL, and `~text~` to `~~text~~`.

## T6: Implement renderMarkdown — full context rendering

Implement `renderMarkdown(messages, userMap)` that formats each message as `**name** (HH:MM): text`, handles attachments/blocks edge cases, and produces the full markdown string with the `## Channel Context` heading.

## T7: Wire captureContext orchestrator function

Connect all pipeline steps in `captureContext`: fetchHistory → filterMessages → reverse → resolveUsers → renderMarkdown → return CaptureResult. Handle empty channel and all-filtered edge cases.

## T8: Add shared types.ts for slack-bot tool

Create `.sdlc/tools/slack-bot/types.ts` with shared type definitions used by both `context.ts` and the future `bot.ts` (from slack-bot-core): `SlackMessage`, `ProjectConfig`, `BotConfig`, normalized payload type.
