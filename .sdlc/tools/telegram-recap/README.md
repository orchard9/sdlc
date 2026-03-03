# telegram-recap

Fetches Telegram chat messages from the configured time window and emails a digest via SMTP.
Delegates all logic to `sdlc telegram digest` — no duplicate implementation.

## Prerequisites

1. A Telegram bot created via [@BotFather](https://t.me/botfather)
2. The bot added to the chats you want to digest
3. `sdlc telegram poll` running (or having run) to populate the local message database
4. SMTP credentials (e.g. [Resend](https://resend.com), SendGrid, Gmail SMTP)

## Setup

### 1. Configure secrets

The tool requires these environment variables. Set them in your orchestrator secrets or shell:

| Variable | Description |
|---|---|
| `TELEGRAM_BOT_TOKEN` | Bot API token from @BotFather |
| `SMTP_HOST` | SMTP server hostname (e.g. `smtp.resend.com`) |
| `SMTP_PORT` | SMTP port (`587` for STARTTLS, `465` for SSL) |
| `SMTP_USERNAME` | SMTP auth username |
| `SMTP_PASSWORD` | SMTP auth password or API key |
| `SMTP_FROM` | From address (e.g. `digest@yourdomain.com`) |
| `SMTP_TO` | Recipient(s), comma-separated |

### 2. Run setup to verify

```bash
sdlc tool run telegram-recap --setup
```

This calls `sdlc telegram status` to verify the bot token and database connectivity.
Returns `{ ok: true, data: { status_output: "..." } }` on success.

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

### Specific chat IDs

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

This removes the need for systemd timers. The orchestrator runs the tool on schedule
and streams results to the sdlc UI via SSE.

## Troubleshooting

**"Bot token check failed"**
→ `TELEGRAM_BOT_TOKEN` is missing or invalid. Verify with `sdlc telegram status`.

**"sdlc telegram digest failed"**
→ Check that `sdlc telegram poll` has been running and the database exists at
`.sdlc/telegram/messages.db`. Run `sdlc telegram status` for diagnostics.

**SMTP errors**
→ Check `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`. Common mistake:
using port 465 with STARTTLS or port 587 with SSL — match port to protocol.

**No messages in digest**
→ The configured chat IDs may not match what the bot has access to, or the time
window contains no messages. Try `--input '{"window_hours": 168}'` (7 days) to widen the window.
