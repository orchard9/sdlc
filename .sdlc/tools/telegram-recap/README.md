# telegram-recap

Fetches Telegram chat messages from the configured time window and emails a digest via [Resend](https://resend.com).
Delegates all logic to `sdlc telegram digest` — no duplicate implementation.

## Prerequisites

1. A Telegram bot created via [@BotFather](https://t.me/botfather)
2. The bot added to the chats you want to digest
3. `sdlc telegram poll` running (or having run) to populate the local message database
4. A [Resend](https://resend.com) account with a verified sender domain

## Setup

### 1. Configure secrets (all from `sdlc secrets env export telegram`)

Store all configuration in the `telegram` secrets env so the tool can source it from one place:

```bash
# Bot token (from @BotFather)
sdlc secrets env set telegram TELEGRAM_BOT_TOKEN="<your-bot-token>"

# Resend credentials
sdlc secrets env set telegram RESEND_API_KEY="re_<your-api-key>"
sdlc secrets env set telegram RESEND_FROM="digest@yourdomain.com"   # must be verified in Resend
sdlc secrets env set telegram RESEND_TO="you@example.com"           # comma-separated for multiple

# Optional: pre-configure chat IDs (see "Finding chat IDs" below)
sdlc secrets env set telegram TELEGRAM_CHAT_IDS="-100123456789,-100987654321"
```

Verify secrets are stored:

```bash
sdlc secrets env export telegram
```

| Variable | Description | Required |
|---|---|---|
| `TELEGRAM_BOT_TOKEN` | Bot API token from @BotFather | Yes |
| `RESEND_API_KEY` | Resend API key (starts with `re_`) | Yes |
| `RESEND_FROM` | Verified sender address (e.g. `digest@yourdomain.com`) | Yes |
| `RESEND_TO` | Recipient(s), comma-separated | Yes |
| `TELEGRAM_CHAT_IDS` | Comma-separated chat IDs to digest | No (can pass at runtime) |

### 2. Run setup to verify

```bash
sdlc tool run telegram-recap --setup
```

Returns `{ ok: true, data: { status_output: "..." } }` on success.

## Finding Chat IDs

Every Telegram chat has a numeric ID. Groups and channels use **negative** IDs
(e.g. `-100123456789`); private chats use positive IDs.

**Step-by-step:**

1. **Add the bot to the chat** — the bot must be a member to receive messages.
2. **Send a message** in the chat (the bot needs at least one update).
3. **Start polling** — run for a few seconds then Ctrl-C:
   ```bash
   sdlc telegram poll
   ```
4. **Query the message DB** to see what the bot received:
   ```bash
   sqlite3 .sdlc/telegram/messages.db \
     "SELECT DISTINCT chat_id, chat_title FROM messages LIMIT 20;"
   ```
5. **Store the IDs** in secrets:
   ```bash
   sdlc secrets env set telegram TELEGRAM_CHAT_IDS="-100123456789,-100987654321"
   ```

> **Tip:** For supergroups and channels, the ID always starts with `-100`. If you
> only see a short negative ID (e.g. `-1001234`), it's a regular group — use it as-is.

## Usage

### Dry run (preview without sending)

```bash
sdlc tool run telegram-recap --input '{"dry_run": true}'
```

### Send digest

```bash
sdlc tool run telegram-recap --input '{}'
```

### Custom window

```bash
# Last 48 hours instead of the default 24
sdlc tool run telegram-recap --input '{"window_hours": 48}'
```

### Specific chat IDs (runtime override)

```bash
sdlc tool run telegram-recap --input '{"chat_ids": ["-100123456789"]}'
```

## Output

```json
{
  "ok": true,
  "data": {
    "dry_run": false,
    "total_messages": 42,
    "chat_count": 3,
    "period_start": "2026-03-01T08:00:00Z",
    "period_end": "2026-03-02T08:00:00Z",
    "sent_to": ["you@example.com"]
  },
  "duration_ms": 1234
}
```

## Scheduling with the orchestrator

Use the sdlc orchestrator for recurrence instead of systemd:

```bash
# Daily digest at the current time
sdlc orchestrate add telegram-recap \
  --tool telegram-recap \
  --input '{}' \
  --at "now" \
  --every 86400
```

## Troubleshooting

**"Bot token check failed"**
→ `TELEGRAM_BOT_TOKEN` is missing or invalid. Check with `sdlc telegram status`
and verify the value in `sdlc secrets env export telegram`.

**"Resend delivery failed"**
→ Check `RESEND_API_KEY` is valid and `RESEND_FROM` is a verified sender domain
in your Resend account. Test with `dry_run: true` first to confirm the digest
builds correctly before attempting to send.

**"sdlc telegram digest failed"**
→ Check that `sdlc telegram poll` has been running and the database exists at
`.sdlc/telegram/messages.db`. Run `sdlc telegram status` for diagnostics.

**No messages in digest**
→ The configured chat IDs may not match what the bot has access to, or the time
window contains no messages. Try `{"window_hours": 168}` (7 days) to widen the
window, or re-run the sqlite3 query above to confirm the bot is receiving messages.

**Chat ID is wrong format**
→ Group/channel IDs are always negative (e.g. `-100123456789`). If you pass a
positive ID for a group, no messages will match. Use the sqlite3 query to get the
exact IDs stored in the database.
