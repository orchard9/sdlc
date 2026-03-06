# Design: slack-bot-core

## Architecture Overview

The Slack bot is a standalone TypeScript process that bridges Slack Events API to sdlc project webhooks. It follows the existing sdlc tool convention (`.sdlc/tools/slack-bot/`).

```
Slack Cloud                  sdlc-server (tunnel)              sdlc tool
+-----------+   HTTP POST   +----------------------+          +------------------+
|  Events   | ------------> | /api/webhooks/slack   | -------> | tool.ts (Bolt)   |
|  API      |   (signed)    | (proxy pass-through)  |          |                  |
+-----------+               +----------------------+          +-----|------------+
                                                                    |
                                                          +---------v---------+
                                                          | Routing Engine    |
                                                          | (config.yaml)    |
                                                          +---------+---------+
                                                                    |
                                                          +---------v---------+
                                                          | POST to project   |
                                                          | webhook URL       |
                                                          +-------------------+
```

**Correction to the above**: In HTTP mode, `@slack/bolt` itself is the HTTP server. The bot listens on its own port and Slack POSTs directly to it (or via the sdlc tunnel). There is no proxying through sdlc-server for the bot's event receiver.

```
Slack Cloud                        tool.ts (Bolt HTTP server)
+-----------+    HTTP POST        +---------------------------+
|  Events   | -----------------> | :3100 (configurable)       |
|  API      |    (HMAC-signed)   | @slack/bolt receiver       |
+-----------+                    | - URL verification         |
      ^                          | - HMAC validation          |
      |                          | - Event dispatch           |
      |                          +-------------|-------------+
      |                                        |
      |    Interaction payloads                |  Routing engine
      +----------------------------------------+
                                               |
                                     +---------v---------+
                                     | POST normalized   |
                                     | payload to project|
                                     | webhook URL       |
                                     +-------------------+
```

## Module Structure

```
.sdlc/tools/slack-bot/
  config.yaml      # Project routing config + secrets metadata
  tool.ts          # Entry point -- Bolt app setup, event handlers
  routing.ts       # Routing engine: resolve project from event
  types.ts         # TypeScript interfaces: Config, Project, PendingMessage, NormalizedPayload
  README.md        # Setup instructions
  package.json     # Dependencies: @slack/bolt, @slack/web-api, yaml
```

## Key Components

### 1. Config Loader (`tool.ts`)

Reads `config.yaml` from the tool directory using the `yaml` npm package. Validates required fields at startup:
- At least one project configured
- Each project has `id`, `name`, `webhook`
- `SLACK_BOT_TOKEN` and `SLACK_SIGNING_SECRET` env vars present

Exits with a clear error message if validation fails.

### 2. Bolt App Setup (`tool.ts`)

```typescript
const app = new App({
  token: process.env.SLACK_BOT_TOKEN,
  signingSecret: process.env.SLACK_SIGNING_SECRET,
  // HTTP mode (not socket mode)
});
```

Registers two handlers:
- `app.event('app_mention', ...)` -- handles @mentions in channels
- `app.action('project_select_*', ...)` -- handles disambiguation button clicks

### 3. Routing Engine (`routing.ts`)

Pure function: `resolveProject(config: Config, channelId: string, messageText: string): Project | null`

Priority chain:
1. `config.default_project` set? Return that project.
2. `channelId` in any project's `channels` array? Return that project.
3. Count keyword matches in `messageText` per project. Return project with most matches (ties: first in config order). Minimum 1 match required.
4. Return `null` (triggers disambiguation).

This is a pure, stateless function -- easy to unit test with no Slack API dependency.

### 4. Disambiguation Manager (`tool.ts`)

In-memory `Map<string, PendingMessage>`:

```typescript
interface PendingMessage {
  channelId: string;
  userId: string;
  text: string;
  messageTs: string;
  threadTs: string | null;
  disambiguationTs: string;  // ts of the bot's button message
  expiresAt: number;         // Date.now() + 5 * 60 * 1000
}
```

On ambiguous route:
- Store the pending message
- Post Block Kit message with buttons

On button click:
- Look up pending message by action `value` (which encodes `messageTs`)
- Route to selected project
- Update the disambiguation message to show confirmation
- Delete from pending map

Lazy expiry: on each new event, sweep entries where `Date.now() > expiresAt`.

### 5. Webhook Poster (`tool.ts`)

After routing resolves:
1. Resolve user display name via `client.users.info({ user })` (cache results in-memory Map)
2. Resolve channel name via `client.conversations.info({ channel })` (cache results)
3. Build `NormalizedPayload` object
4. `fetch(project.webhook, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) })`
5. Reply in Slack thread with confirmation or error

### 6. Block Kit Message Structure

```json
{
  "text": "Which project should this go to?",
  "blocks": [
    {
      "type": "section",
      "text": { "type": "mrkdwn", "text": "Which project should this go to?" }
    },
    {
      "type": "actions",
      "elements": [
        {
          "type": "button",
          "text": { "type": "plain_text", "text": "sdlc (Ponder)" },
          "action_id": "project_select_sdlc",
          "value": "1234567890.123456"
        },
        {
          "type": "button",
          "text": { "type": "plain_text", "text": "Cancel" },
          "action_id": "project_select_cancel",
          "value": "1234567890.123456",
          "style": "danger"
        }
      ]
    }
  ]
}
```

## Data Flow

```
1. Slack sends POST /slack/events to bot HTTP server
2. Bolt verifies HMAC signature (automatic)
3. Bolt parses event type
4. If url_verification: respond with challenge (automatic)
5. If app_mention:
   a. Extract channel, user, text, message_ts, thread_ts
   b. Call resolveProject(config, channel, text)
   c. If project found: post to webhook, reply with confirmation
   d. If null: post disambiguation buttons, store pending
6. If block_actions (button click):
   a. Look up pending message
   b. If "cancel": update message, discard pending
   c. If project: post to webhook, update message with confirmation
```

## Error Handling

| Scenario | Behavior |
|---|---|
| Invalid HMAC | Bolt rejects with 401 (built-in) |
| Stale timestamp (>5 min) | Bolt rejects (built-in) |
| Config missing/invalid | Exit with error message on startup |
| Webhook POST fails | Reply in Slack with error status code |
| User/channel info API fails | Fall back to raw IDs in payload |
| Expired disambiguation click | Reply "This request has expired" |

## Configuration Reference

### config.yaml

```yaml
name: slack-bot
version: "1.0.0"
description: "Slack bot for multi-project feedback routing"

port: 3100  # HTTP server port for receiving Slack events

projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.threesix.ai/api/webhooks/slack
    channels: ["C0123456789"]
    keywords: ["sdlc", "ponder"]

default_project: null
default_artifact: "feedback_thread"

secrets:
  - env_var: SLACK_BOT_TOKEN
    description: "Bot OAuth token (xoxb-...)"
    required: true
  - env_var: SLACK_SIGNING_SECRET
    description: "Slack app signing secret for HMAC verification"
    required: true
```

## Testing Strategy

- **Unit tests** for `resolveProject()`: test each priority chain step, ties, empty config.
- **Integration test**: mock Slack events, verify webhook POST payload shape.
- **Manual test**: deploy with a real Slack workspace, @mention bot, verify routing + disambiguation.
