---
session: 1
timestamp: 2026-03-03T20:30:00Z
orientation:
  current: "Shaping the Slack bot architecture: what it creates, how it routes, how it's configured"
  next: "Capture a routing-config spec artifact and decide on Slack API approach (Events API vs socket mode)"
  commit: "When we have a clear config schema, routing algorithm, artifact type mapping, and implementation path"
---

**jordan · Owner**
slack bot tool

we need a slack bot to create threads or ponder messages -we want it to default

ideally the slackbot can route it to the proper webhook through a configuration

so if it is a bug for project 1 it goes to webhook 1, project 2 goes to webhook 2, project unknown asks which project

can configure the bot to know about multiple projects

can configure the tool to be listening for a specific project & webhook

---

## Session 1 — Shaping the Slack Bot Architecture

**Facilitator**
Jordan's brief lands in three layers: (1) a Slack bot that captures user messages, (2) routes them to the right sdlc project webhook, and (3) falls back to asking when the project is ambiguous. Let me bring in the right voices.

**Felix Wagner · Developer Tooling Architect**
My first question is: what is this bot actually creating? The brief says "threads or ponder messages" — but those are two very different artifacts in sdlc. A feedback thread is structured feedback/bug/question with a body and optional metadata. A ponder entry is an open-ended ideation workspace. Getting the classification right is the linchpin.

I'd default the bot to **feedback threads** — because the natural Slack use case is "I have a bug to report" or "I have a request." Ponder entries are deliberate ideation sessions. Let the bot create a feedback thread by default, and add a keyword like `ponder:` to opt into a ponder entry.

?  Open: Does jordan want both artifact types, or just feedback threads first?

**Aria Chen · Agent Ergonomics**
The routing problem is the interesting architecture. Jordan described it clearly — project → webhook — but there are three distinct routing signals worth separating:

1. **Channel-based routing** — the Slack channel tells you the project (`#sdlc-bugs` maps to project `sdlc`)
2. **Keyword-based routing** — the message contains `project: sdlc` or `@sdlc-bot for=sdlc`
3. **Ambient routing** — the bot instance itself is configured to only serve one project (single-project mode)

For the ambiguous case: instead of asking "which project?" in free text (which creates another NLU problem), the bot should reply with a message containing **quick-reply buttons** for each configured project. Slack's Block Kit supports this natively — it's fast, zero-ambiguity, and respects mobile users.

**Felix Wagner**
Let's nail down the config schema. Looking at the telegram-recap tool pattern — it uses `config.yaml` in `.sdlc/tools/<name>/` plus secrets in `sdlc secrets env`. Same pattern here makes sense:

```yaml
# .sdlc/tools/slack-bot/config.yaml
name: slack-bot
version: "1.0.0"

# Global: list of known projects
projects:
  - id: sdlc
    name: "sdlc (Ponder)"
    webhook: https://sdlc.example.com/api/webhooks/slack
    channels: ["C0123456789"]         # Slack channel IDs that auto-route here
    keywords: ["sdlc", "ponder"]      # message keywords that hint this project

  - id: app-backend
    name: "App Backend"
    webhook: https://backend.example.com/api/webhooks/slack
    channels: ["C9876543210"]
    keywords: ["backend", "api", "auth"]

# Optional: pin the bot to one project (single-project mode)
# When set, all messages go here — no routing, no disambiguation
default_project: null  # or "sdlc"

# What artifact to create when routed
default_artifact: "feedback_thread"  # or "ponder"
```

Secrets (bot token, signing secret) go via `sdlc secrets env set slack-bot`:
```
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
```

⚑  Decided: config schema follows telegram-recap pattern — `config.yaml` + `sdlc secrets env`

**Aria Chen**
On the routing algorithm — I'd implement it as a priority chain:

```
1. Is `default_project` set? → route there unconditionally
2. Is the incoming channel ID in a project's `channels` list? → route to that project
3. Does the message body contain a project's keyword? → route to highest-match project
4. None of the above → ask with quick-reply buttons
```

This is deterministic and testable. No LLM involved in routing — just string matching and config lookup.

**Felix Wagner**
The Slack API choice matters for operational simplicity. Two viable options:

**Option A: Slack Events API (HTTP webhook)**
- Slack sends HTTP POST to a public URL when events occur
- Requires a public URL (ngrok for dev, real domain for prod)
- Standard, well-documented
- sdlc server adds a `/api/webhooks/slack/events` endpoint

**Option B: Slack Socket Mode**
- Bot opens a WebSocket to Slack — no public URL needed
- Better for private/local networks
- Slightly more complex reconnect logic
- Works as an sdlc tool daemon: `sdlc tool run slack-bot --listen`

Given that sdlc already runs a server with real public URLs (via the tunnel), Option A (HTTP Events API) is simpler and more reliable. But Socket Mode is better for developers running locally without a tunnel.

?  Open: Does the user expect this to run locally (dev mode) or always server-side?

**Facilitator**
Let me push on what "routing to a webhook" actually means. Right now, sdlc-server doesn't have a Slack webhook receiver — we'd need to build `/api/webhooks/slack/events`. What does the webhook handler do?

**Aria Chen**
The webhook handler:
1. Verifies Slack signature (HMAC-SHA256 on the request body using `SLACK_SIGNING_SECRET`)
2. Handles the URL verification challenge (Slack sends this when first configuring)
3. Parses the event payload (message, app_mention, shortcut, etc.)
4. Runs the routing algorithm
5. Calls `sdlc api feedback create` or `sdlc api ponder create` — whichever applies
6. Posts a reply to Slack confirming: "Created feedback thread #42 for project sdlc ↗️"

The `webhook` URL in the config isn't the Slack webhook — it's the **sdlc server's REST endpoint** for creating artifacts. So the bot is the bridge: Slack → bot process → sdlc server REST.

Wait — actually re-reading jordan's brief: "ideally the slackbot can route it to the proper webhook through a configuration." The `webhook` in config might literally be the sdlc server's `/api/...` endpoint. So the bot POSTs JSON to the configured webhook URL. That's cleanly decoupled — the bot doesn't know what sdlc is, it just routes payloads.

⚑  Decided: `webhook` in config is the target REST endpoint — bot POSTs a normalized JSON payload; recipient endpoint is responsible for creating the artifact

**Felix Wagner**
That decoupling is valuable. It means the bot works with any webhook URL — not just sdlc. And it means we can test routing without needing a live sdlc server.

The normalized payload shape:
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

⚑  Decided: normalized payload shape (above) — bot is a routing layer, not an artifact creator

**Aria Chen**
On "project unknown — asks which project": the bot should:
1. Post a message with Block Kit buttons — one per configured project + a "Cancel" option
2. Set a timeout (e.g., 5 minutes) — if no reply, drop the message and reply "Got it, ignored this one"
3. Store pending messages in memory (or a local SQLite, like the telegram MessageStore) keyed by `message_ts`
4. When user clicks a button → Slack sends an interaction event → bot looks up pending message → routes it

?  Open: Should pending messages survive bot restarts? SQLite persistence vs. in-memory?

**Felix Wagner**
For v1, in-memory is fine. State that lives longer than a bot restart creates operational complexity. Users who miss the window just re-send.

**Facilitator**
Let me summarize the shape that's emerging and check the commit signal:

### What's Clear

1. **Artifact type**: Default to `feedback_thread`. Add keyword `ponder:` to create ponder entries. Config has `default_artifact` override.

2. **Routing algorithm** (priority chain):
   - `default_project` set → unconditional route
   - Channel ID in project's `channels` list → route to that project
   - Message keyword match → route to highest-match project
   - None → quick-reply disambiguation buttons

3. **Config schema**: `config.yaml` + `sdlc secrets env set slack-bot` (token + signing secret)

4. **Architecture**: Slack Events API (HTTP) for production; Socket Mode optional for local dev. sdlc-server gains `/api/webhooks/slack/events`.

5. **Bot is a routing layer**: POSTs normalized JSON to configured webhook URLs. Projects register their own endpoint — decoupled from sdlc internals.

6. **Disambiguation UX**: Block Kit buttons (one per project). Pending messages in-memory with 5-minute timeout.

### What's Still Open

?  Open (1): Single artifact type MVP (feedback_thread only) vs. supporting ponder from day 1?
?  Open (2): Socket Mode support for local dev — scope it into v1 or punt?
?  Open (3): Message triggers — only app_mention (`@slack-bot`) or also DMs and channel messages?

**Felix Wagner**
My recommendations:
- **v1 scope**: feedback_thread only. Ponder routing is a v2 addition (one extra keyword check, trivial to add later).
- **Socket Mode**: Include it as the primary mode — it's simpler for users who don't have a public URL. HTTP mode is the optional extra.
- **Triggers**: App mentions (`@slack-bot`) for public channels, DMs always pass through. This prevents the bot from intercepting every channel message.

**Aria Chen**
Agreed on Socket Mode first. The sdlc server's tunnel already exists for when users want HTTP mode.

⚑  Decided: Socket Mode is the primary architecture; HTTP Events API is optional/v2
⚑  Decided: v1 creates feedback_thread only; ponder is keyword-triggered in v2
⚑  Decided: Triggers = app mentions + DMs only (not all channel messages)

**Facilitator**
Implementation path for the sdlc tool:

```
.sdlc/tools/slack-bot/
├── config.yaml         # project routing config
├── README.md           # setup guide (token, socket mode, config)
└── (agent runtime)

# CLI: sdlc tool run slack-bot [--dry-run] [--once]
# Runs as a daemon, connects via Socket Mode, routes messages
```

For the sdlc server side: the sdlc server needs a feedback thread creation endpoint that accepts the normalized payload. That's the `feedback-thread-core` feature set. The bot is a client of that endpoint.

**Felix Wagner**
The tool itself could be a TypeScript agent (like `.sdlc/tools/_shared/agent.ts`) or a Rust binary embedded in sdlc-cli. Given we already have the telegram polling loop in Rust (`telegram/poll.rs`), the Slack polling loop follows the same pattern.

But: Slack's Socket Mode requires a WebSocket library, and there's prior art in TypeScript (`@slack/bolt`) that makes it 10x simpler. I'd recommend the tool runs as a TypeScript/Node.js script that:
1. Is scaffolded by `sdlc tool build slack-bot`
2. Lives in `.sdlc/tools/slack-bot/` as `bot.ts`
3. Uses `@slack/bolt` for Socket Mode
4. Reads config from `config.yaml` and env from `sdlc secrets env export slack-bot`

This keeps Rust focused on data — the bot logic (routing, disambiguation, retry) lives in the skill/agent layer, just like the architecture principle says.

⚑  Decided: Bot implementation = TypeScript/Node.js with @slack/bolt, not Rust — keeps routing logic in the skill layer per the Architecture Principle: Rust = Data, Skills = Logic

**Facilitator**
That's a clean boundary. The Rust layer would only need:
- A REST endpoint to create feedback threads (already in feedback-thread-core)
- No new Rust code for the bot itself

### Commit Signal Assessment

We have:
- Clear artifact type (feedback_thread v1)
- Clear routing algorithm (4-step priority chain)
- Clear config schema
- Clear architecture (TypeScript/Bolt, Socket Mode)
- Clear integration points (normalized payload → feedback thread REST endpoint)
- Clear disambiguation UX (Block Kit buttons, in-memory pending, 5-min timeout)
- Clear trigger scope (app mentions + DMs)

The commit signal is **met**. This is ready to crystallize into a feature.

**Next:** Capture the routing-config spec as a scrapbook artifact, then mark status as `converging`.
