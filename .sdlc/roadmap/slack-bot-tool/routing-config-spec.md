# Slack Bot Routing Config Spec

## Config File: `.sdlc/tools/slack-bot/config.yaml`

```yaml
name: slack-bot
version: "1.0.0"

# List of known projects with routing rules
projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.example.com/api/webhooks/slack  # target REST endpoint
    channels: ["C0123456789"]   # Slack channel IDs that auto-route here
    keywords: ["sdlc", "ponder"] # message keywords that hint this project

  - id: app-backend
    name: "App Backend"
    webhook: https://backend.example.com/api/webhooks/slack
    channels: ["C9876543210"]
    keywords: ["backend", "api", "auth"]

# Optional: pin bot to one project (single-project mode)
# When set, all messages route here — no routing, no disambiguation
default_project: null  # or "sdlc"

# Default artifact type created at the target webhook
default_artifact: "feedback_thread"
```

## Secrets: `sdlc secrets env set slack-bot`

| Variable | Description |
|---|---|
| `SLACK_BOT_TOKEN` | Bot OAuth token (xoxb-...) |
| `SLACK_APP_TOKEN` | App-level token for Socket Mode (xapp-...) |
| `SLACK_SIGNING_SECRET` | For HTTP Events API (optional v2) |

## Routing Algorithm (priority chain)

1. **Single-project mode**: `default_project` is set → route unconditionally
2. **Channel match**: incoming Slack channel ID is in a project's `channels` list → route
3. **Keyword match**: message text contains a project's keyword → route to highest-match project
4. **Ambiguous**: none of the above → reply with Block Kit buttons (one per project + Cancel)

## Disambiguation UX

- Bot replies with Block Kit buttons, one per configured project + "Cancel"
- Pending messages stored in-memory keyed by `message_ts`
- 5-minute timeout — if no selection, bot replies "Got it, ignored" and drops the message
- On selection → bot looks up pending message → routes to selected project's webhook

## Normalized Payload (POST to webhook URL)

```json
{
  "source": "slack",
  "project": "sdlc",
  "channel": "C0123456789",
  "channel_name": "sdlc-bugs",
  "user": "U0123456789",
  "user_name": "jordan",
  "text": "login button is broken on mobile",
  "message_ts": "1234567890.123456",
  "thread_ts": null,
  "artifact_type": "feedback_thread",
  "slack_team": "T0123456789"
}
```

## Trigger Scope

- **App mentions** in channels: `@slack-bot login is broken`
- **Direct messages** to the bot: always routed
- **NOT**: all channel messages (too noisy, breaks team workflows)

## Architecture

- **Bot runtime**: TypeScript/Node.js with `@slack/bolt`
- **Primary mode**: Socket Mode (no public URL required)
- **HTTP Events API**: optional v2 — sdlc server adds `/api/webhooks/slack/events`
- **Bot is a routing layer only** — it POSTs normalized JSON; the target endpoint creates artifacts
- **No Rust code for bot logic** — TypeScript per Architecture Principle: Rust = Data, Skills = Logic

## v1 Scope

- Feedback threads only (default artifact)
- Socket Mode only
- App mentions + DMs
- Block Kit disambiguation

## v2 Additions

- `ponder:` keyword → create ponder entry instead
- HTTP Events API support
- Persistence for pending messages (SQLite, like telegram MessageStore)
