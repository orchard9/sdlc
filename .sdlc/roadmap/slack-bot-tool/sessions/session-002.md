---
session: 2
timestamp: 2026-03-03T21:00:00Z
orientation:
  current: "Architecture finalized: HTTP Events API + context-windowed capture (50-200 msgs on @mention)"
  next: "Commit to features — /sdlc-ponder-commit slack-bot-tool"
  commit: "Met — HTTP mode confirmed, context-gathering behavior defined, config schema complete"
---

**jordan · Owner**
1. feedback thread is fine
2. we do not need socket mode - we have tunnels we can use for dev
3. if we @ bot it should look at that channel and read back 50-200 messages to understand the bug

---

## Session 2 — Locking the Architecture: HTTP Mode + Context Gathering

**Facilitator**
Three sharp corrections from jordan. Let me work through each one with the team and update the spec accordingly.

**Correction 1 is confirmation** — feedback thread is the artifact type. That decision from Session 1 holds.

**Correction 2 flips our primary architecture decision.** Session 1 landed on Socket Mode as primary, arguing "simpler for users without a public URL." Jordan just told us: we already have tunnels. The sdlc tunnel infrastructure (orch-tunnel, QR printing, token generation) already solves the public URL problem for dev. HTTP Events API is primary, socket mode is gone.

**Correction 3 is the most interesting new capability.** The bot isn't just a passive relay — when @mentioned, it actively reads channel history (50-200 messages) and includes that context in the feedback thread. This changes what a feedback thread becomes: self-contained, pre-loaded with the conversation that led to the report.

---

**Felix Wagner · Developer Tooling Architect**

The tunnel flip is clean. HTTP Events API means:
- Slack sends POST to `https://<your-tunnel>/api/webhooks/slack/events`
- sdlc-server can add `/api/webhooks/slack/events` OR the TypeScript bot handles it directly
- Secrets simplify: `SLACK_BOT_TOKEN` + `SLACK_SIGNING_SECRET` only. No `SLACK_APP_TOKEN` (that's Socket Mode only)
- For dev: `sdlc tunnel` gives you the public URL. For prod: the real server URL.

**Aria Chen · Agent Ergonomics**

Where does the bot live? With HTTP Events API the question is sharpened:

- **Option A**: Standalone TypeScript bot process — receives Slack events on its own port, calls Slack API for history, POSTs normalized payload to sdlc webhook. Decoupled, independent.
- **Option B**: Endpoint inside sdlc-server — Rust handles the Slack event, calls Slack API via reqwest, routes internally. Single process.

Given the bot needs to call `conversations.history`, post Block Kit messages, and handle interaction payloads (button clicks for disambiguation) — the Slack API surface is wide. `@slack/bolt` in TypeScript handles all of this cleanly. Option A wins.

⚑  Decided: Standalone TypeScript/Node.js bot process with `@slack/bolt` in HTTP mode.

---

**Felix Wagner**
On the channel history capability — the new behavior on @mention:

```
1. User: @sdlc-bot login is broken on mobile
2. Bot receives app_mention event
3. Bot calls conversations.history → fetches last N messages
4. Bot filters: remove bot messages, remove the @mention itself
5. Bot reverses to chronological order (oldest-first)
6. Bot resolves user IDs → display names via users.info
7. Bot renders as markdown
8. Bot POSTs normalized payload (trigger + context) to project webhook
9. Bot replies: "Created feedback thread for [project]. Read 87 messages for context. ↗️"
```

The Slack API:
```
GET https://slack.com/api/conversations.history
  ?channel=C0123456789
  &limit=200
  &latest=<message_ts>
```

Required scopes: `channels:history` (public), `groups:history` (private), `im:history` (DMs).

**Aria Chen**
Rendered markdown in the payload body is the right call for v1:

```markdown
## Channel Context (last 87 messages)

**jordan** (2026-03-03 20:10): I can't login on mobile, getting 401
**bob** (2026-03-03 20:11): Same here, started about an hour ago
**alice** (2026-03-03 20:12): I cleared cache, still broken
```

Feedback thread body = trigger message + rendered context block. The target endpoint doesn't need to know about Slack message formats.

⚑  Decided: Bot renders channel history as markdown before including in payload. Body = trigger + rendered context.
⚑  Decided: Filter — remove bot messages AND the @mention trigger itself. Reverse to oldest-first.

---

**Felix Wagner**
Config updated:

```yaml
name: slack-bot
version: "1.0.0"

projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.example.com/api/feedback/create
    channels: ["C0123456789"]
    keywords: ["sdlc", "ponder"]

  - id: app-backend
    name: "App Backend"
    webhook: https://backend.example.com/api/feedback/create
    channels: ["C9876543210"]
    keywords: ["backend", "api"]

default_project: null
default_artifact: "feedback_thread"
context_messages: 100   # how many messages to read for context (50-200 range)
```

Secrets (2, down from 3):
```
SLACK_BOT_TOKEN=xoxb-...           # all API calls
SLACK_SIGNING_SECRET=...           # HTTP signature verification
```

No SLACK_APP_TOKEN — Socket Mode dropped.

⚑  Decided: `context_messages: 100` default. Configurable. Range 50-200 per jordan's brief.

---

**Felix Wagner**
Updated normalized payload:

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
  "context": "## Channel Context (last 87 messages)\n\n**jordan** (20:10): ...\n**bob** (20:11): ...",
  "context_message_count": 87
}
```

⚑  Decided: Payload gains `context` (rendered markdown string) and `context_message_count` (int) fields.

---

**Aria Chen**
One behavioral question: should every @mention create a feedback thread, or should there be a confirmation step? What if someone @mentions the bot to ask a question rather than report a bug?

**Felix Wagner**
Always create. Wrong is fixable, blocked is not. Spurious feedback threads are low cost — the user can close them. A confirmation step adds friction to every legitimate use case.

⚑  Decided: Always create on @mention. No confirmation step. Fire and iterate.

---

**Facilitator — Architecture Summary**

### Final Architecture

**Stack:**
- `@slack/bolt` TypeScript bot, HTTP mode
- Existing sdlc tunnel → public URL for dev (no new infra needed)
- Bot is a routing layer + context gatherer — POSTs to configured webhook URLs

**Required Scopes:**

| Scope | Purpose |
|---|---|
| `app_mentions:read` | receive @mention events |
| `channels:history` | read public channel messages |
| `groups:history` | read private channel messages |
| `im:history` | read DM messages |
| `users:read` | resolve user IDs to display names |
| `chat:write` | post replies and Block Kit messages |

### What Changed from Session 1

| Decision | Session 1 | Session 2 (Final) |
|---|---|---|
| Slack API mode | Socket Mode (primary) | HTTP Events API (only) |
| Dev URL | Socket Mode (no URL needed) | sdlc tunnel |
| Secrets | BOT_TOKEN + APP_TOKEN + SIGNING_SECRET | BOT_TOKEN + SIGNING_SECRET |
| Bot capability | Passive relay | Context-windowed capture |
| Payload | Trigger message only | Trigger + rendered channel history |
| Confirmation step | None | None (always create) |

### The Core Capability: Context-Windowed Capture

This is the name for what this bot does. It's what makes it more than a webhook bridge: the feedback thread arrives at the sdlc server pre-loaded with the full conversation that surfaced the bug. An agent reading the thread sees what happened, who saw it, what they tried — without anyone having to write a bug report.

⚑  Decided: HTTP Events API is primary — Socket Mode dropped entirely.
⚑  Decided: Context-windowed capture is the core capability.
⚑  Decided: Default context depth = 100 messages (configurable, 50-200 range).
⚑  Decided: Payload gains `context` + `context_message_count` fields.
⚑  Decided: Always create on @mention — no confirmation step.
⚑  Decided: 2 secrets only — SLACK_BOT_TOKEN + SLACK_SIGNING_SECRET.
