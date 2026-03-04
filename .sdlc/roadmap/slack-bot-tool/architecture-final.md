# Slack Bot Architecture — Final Spec (Session 2)

## Core Capability: Context-Windowed Capture

When @mentioned, the bot reads the last N messages from that channel and includes them as rendered markdown in the feedback thread. This is the key differentiator — feedback threads are self-contained with full conversation context.

## Event Flow on @mention

1. Slack sends POST to bot public URL (sdlc tunnel in dev, direct URL in prod)
2. Bot verifies HMAC-SHA256 signature (`SLACK_SIGNING_SECRET`)
3. Bot runs routing algorithm → determines project (or asks with Block Kit buttons)
4. Bot calls `conversations.history` — fetches last N messages (`context_messages` from config, default 100)
5. Bot filters: remove bot messages, remove the trigger @mention itself
6. Bot reverses messages to chronological order (oldest first)
7. Bot resolves user IDs to display names via `users.info`
8. Bot renders context as markdown
9. Bot POSTs normalized payload (trigger + rendered context) to project webhook URL
10. Bot replies: "Created feedback thread for [project]. Read N messages for context. ↗️"

## Config Schema

```yaml
name: slack-bot
version: "1.0.0"

projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.example.com/api/feedback/create
    channels: ["C0123456789"]
    keywords: ["sdlc", "ponder"]

default_project: null
default_artifact: "feedback_thread"
context_messages: 100   # messages to read for context (50-200 range)
```

## Secrets

```
SLACK_BOT_TOKEN=xoxb-...           # Bot token — all API calls
SLACK_SIGNING_SECRET=...           # HTTP signature verification
```

No SLACK_APP_TOKEN — Socket Mode not used.

## Required Slack Scopes

| Scope | Purpose |
|---|---|
| `app_mentions:read` | receive @mention events |
| `channels:history` | read public channel messages |
| `groups:history` | read private channel messages |
| `im:history` | read DM messages |
| `users:read` | resolve user IDs to display names |
| `chat:write` | post replies and Block Kit messages |

## Normalized Payload

```json
{
  "source": "slack",
  "project": "sdlc",
  "channel": "C0123456789",
  "channel_name": "sdlc-bugs",
  "user": "U0123456789",
  "user_name": "jordan",
  "text": "login is broken on mobile",
  "message_ts": "1234567890.123456",
  "thread_ts": null,
  "artifact_type": "feedback_thread",
  "slack_team": "T0123456789",
  "context": "## Channel Context (last 87 messages)\n\n**jordan** (20:10): I can't login...",
  "context_message_count": 87
}
```

## Routing Algorithm (unchanged from Session 1)

1. `default_project` set → route unconditionally
2. Channel ID in project's `channels` list → route
3. Message keyword match → route to highest-match project
4. Ambiguous → Block Kit buttons (one per project + Cancel)

## Disambiguation UX (unchanged from Session 1)

- Block Kit buttons, one per project + Cancel
- In-memory pending messages keyed by `message_ts`
- 5-minute timeout → bot replies "Got it, ignored" and drops

## Architecture

- **Bot runtime**: TypeScript/Node.js with `@slack/bolt` (HTTP mode)
- **API mode**: HTTP Events API — Socket Mode dropped
- **Dev URL**: sdlc existing tunnel infrastructure (no ngrok needed)
- **Bot role**: Routing layer + context gatherer — not artifact creator
- **Artifact creation**: Handled by target webhook (sdlc feedback thread endpoint)

## Trigger Scope

- App mentions in channels: `@slack-bot text`
- Direct messages to bot: always routed
- NOT: all channel messages

## Behavior on @mention

- Always creates feedback thread — no confirmation step (fire and iterate ethos)
- Replies with thread link + message count read

## v1 Scope

- Feedback threads only
- HTTP Events API
- Context-windowed capture (default 100 messages)
- App mentions + DMs
- Block Kit disambiguation

## Key Changes from Session 1

| Decision | Session 1 | Session 2 (Final) |
|---|---|---|
| Slack API mode | Socket Mode (primary) | HTTP Events API (primary, Socket Mode dropped) |
| Dev URL strategy | Needs Socket Mode | Uses existing sdlc tunnel |
| Secrets | BOT_TOKEN + APP_TOKEN + SIGNING_SECRET | BOT_TOKEN + SIGNING_SECRET |
| Bot capability | Passive relay | Context-windowed capture |
| Payload | Trigger message only | Trigger + rendered channel history |
| Confirmation step | None | None (always create) |