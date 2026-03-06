# Spec: slack-bot-core

## Summary

A TypeScript/Node.js Slack bot that runs in HTTP Events API mode (using `@slack/bolt`), receives Slack events via a public webhook endpoint, verifies HMAC signatures, routes messages to the correct sdlc project webhook based on a priority-chain routing algorithm, and uses Block Kit buttons for disambiguation when the target project is ambiguous.

## Milestone Context

Part of **v39-slack-bot-tool** ("Slack bot with context-windowed capture for multi-project feedback routing"). This feature covers the bot runtime, config schema, routing logic, and HMAC verification. Sibling features `slack-context-capture` and `slack-feedback-endpoint` handle conversation history capture and the server-side feedback endpoint respectively.

## Requirements

### R1: TypeScript/Bolt HTTP Mode Runtime

- Bot is a TypeScript file (`tool.ts`) in `.sdlc/tools/slack-bot/` following the existing tool convention (see `telegram-recap` for prior art).
- Uses `@slack/bolt` in HTTP (Events API) mode -- not Socket Mode.
- Receives events via the sdlc server's existing tunnel infrastructure or any publicly reachable URL.
- Handles the Slack URL verification challenge (`url_verification` event type).
- Listens for `app_mention` events in channels and direct messages.
- Runs as a long-lived process: `npx tsx .sdlc/tools/slack-bot/tool.ts` (or equivalent).

### R2: Config Schema

Config file at `.sdlc/tools/slack-bot/config.yaml`:

```yaml
name: slack-bot
version: "1.0.0"

projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.example.com/api/webhooks/slack
    channels: ["C0123456789"]
    keywords: ["sdlc", "ponder"]

  - id: app-backend
    name: "App Backend"
    webhook: https://backend.example.com/api/webhooks/slack
    channels: ["C9876543210"]
    keywords: ["backend", "api"]

default_project: null
default_artifact: "feedback_thread"
```

Secrets via environment variables (loaded from `sdlc secrets env export slack-bot` or injected directly):

| Variable | Required | Purpose |
|---|---|---|
| `SLACK_BOT_TOKEN` | Yes | Bot OAuth token (`xoxb-...`) |
| `SLACK_SIGNING_SECRET` | Yes | Slack app signing secret for HMAC verification |
| `SLACK_APP_TOKEN` | No | App-level token (only if Socket Mode is added later) |

The `config.yaml` also declares secrets metadata following the `telegram-recap` pattern.

### R3: HMAC Signature Verification

- Every incoming HTTP request from Slack is verified using HMAC-SHA256.
- The bot computes `HMAC-SHA256(signing_secret, "v0:" + timestamp + ":" + raw_body)` and compares against the `X-Slack-Signature` header.
- Requests with invalid signatures are rejected with HTTP 401.
- Requests with timestamps older than 5 minutes are rejected to prevent replay attacks.
- `@slack/bolt` handles this automatically when `signingSecret` is configured; the spec documents the expectation so tests can verify it.

### R4: Multi-Project Routing Algorithm

Priority-chain routing (deterministic, no LLM):

1. **Default project** -- If `default_project` is set in config, route unconditionally.
2. **Channel match** -- If the incoming Slack channel ID appears in a project's `channels` list, route to that project.
3. **Keyword match** -- If the message body contains one or more of a project's `keywords`, route to the project with the most keyword matches. Ties broken by config order.
4. **Disambiguation** -- If none of the above match, reply with Block Kit buttons (one per configured project + Cancel).

### R5: Block Kit Disambiguation

When routing is ambiguous (no default, no channel match, no keyword match):

- Bot replies in the originating channel/DM with a message: "Which project should this go to?" followed by Block Kit action buttons -- one per configured project, plus a "Cancel" button.
- Pending disambiguation state is held in-memory (a `Map<string, PendingMessage>`), keyed by the Slack `message_ts`.
- When the user clicks a project button, the bot routes the original message to that project's webhook and updates the disambiguation message to confirm: "Routed to [Project Name]".
- When the user clicks Cancel, the bot updates the message to "Cancelled" and discards the pending message.
- Pending messages expire after 5 minutes; expired entries are cleaned up lazily on the next event cycle.

### R6: Normalized Webhook Payload

When routing resolves, the bot POSTs a JSON payload to the target project's `webhook` URL:

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

- `artifact_type` defaults to `config.default_artifact` (initially `"feedback_thread"`).
- The bot resolves user display names using `users.info` API where possible, falling back to the raw user ID.
- The bot resolves channel names using `conversations.info` API, falling back to the raw channel ID.

### R7: Confirmation Reply

After successfully POSTing to the webhook:
- Bot replies in the Slack thread (or channel) with a confirmation: "Created feedback thread for [Project Name]".
- If the webhook POST fails (non-2xx response), the bot replies with an error: "Failed to route to [Project Name]: [status code]".

## Out of Scope

- **Socket Mode**: HTTP Events API only for v1. Socket Mode can be added as a future enhancement.
- **Ponder entry creation**: v1 only creates feedback threads. `ponder:` keyword routing is deferred.
- **Conversation history capture**: Handled by sibling feature `slack-context-capture`.
- **Server-side feedback endpoint**: Handled by sibling feature `slack-feedback-endpoint`.
- **Persistent disambiguation state**: In-memory only; does not survive bot restarts.

## Success Criteria

1. Bot starts, connects to Slack via HTTP Events API, and handles URL verification.
2. An `@mention` in a channel mapped to a project routes the message to that project's webhook within 2 seconds.
3. An `@mention` in an unmapped channel triggers Block Kit disambiguation buttons.
4. Clicking a project button routes the message and confirms in Slack.
5. Invalid HMAC signatures are rejected with 401.
6. Config with `default_project` set routes all messages to that project without disambiguation.
