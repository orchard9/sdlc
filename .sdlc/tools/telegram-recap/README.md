# telegram-recap

Fetches Telegram messages from the sdlc webhook store and emails a daily digest via Resend.

## Architecture

Telegram is configured to push updates to the sdlc server via `setWebhook`. The sdlc server
stores every incoming Telegram update as a `store_only` webhook payload. The telegram-recap tool
queries those stored payloads, builds a digest, and sends it via Resend.

```
Telegram → POST /webhooks/telegram (store_only route)
                    ↓
         sdlc orchestrator stores payload
                    ↓
         telegram-recap (scheduled daily)
         queries GET /api/webhooks/telegram/data?since=<24h ago>
                    ↓
         builds digest → Resend email
```

## Prerequisites

1. sdlc server accessible (default: http://localhost:7777)
2. A Telegram bot created via @BotFather
3. A Resend account with a verified sender domain

## Setup

### 1. Register the webhook route on the sdlc server

```bash
# Register telegram as a store_only route (no dispatch — just capture)
curl -X POST http://localhost:7777/api/orchestrator/webhooks/routes \
  -H 'Content-Type: application/json' \
  -d '{
    "path": "/telegram",
    "tool_name": "telegram-recap",
    "input_template": "{}",
    "store_only": true,
    "secret_token": "your-secret-here"
  }'
```

### 2. Configure secrets

```bash
sdlc secrets env set telegram TELEGRAM_BOT_TOKEN="<your-bot-token>"
sdlc secrets env set telegram RESEND_API_KEY="re_<your-api-key>"
sdlc secrets env set telegram RESEND_FROM="digest@yourdomain.com"
sdlc secrets env set telegram RESEND_TO="you@example.com"

# Optional
sdlc secrets env set telegram SDLC_SERVER_URL="https://your-sdlc-server.example.com"
sdlc secrets env set telegram TELEGRAM_WEBHOOK_ROUTE="telegram"
```

### 3. Point Telegram at your server

```bash
curl "https://api.telegram.org/bot<TOKEN>/setWebhook" \
  -d "url=https://your-sdlc-server.example.com/webhooks/telegram" \
  -d "secret_token=your-secret-here"
```

### 4. Run setup check

```bash
sdlc tool run telegram-recap --setup
```

Returns `{ ok: true, data: { bot_username, webhook_store: 'reachable'|'unavailable' } }` on success.

### 5. Schedule daily digest

```bash
sdlc orchestrate add daily-telegram-digest \
  --tool telegram-recap \
  --input '{}' \
  --at "now" \
  --every 86400
```

## One-shot test

```bash
sdlc tool run telegram-recap --input '{"dry_run":true}'
```

## Secret reference

| Variable | Description | Required |
|---|---|---|
| TELEGRAM_BOT_TOKEN | Bot API token from @BotFather | Setup only |
| RESEND_API_KEY | Resend API key (starts with re_) | Yes |
| RESEND_FROM | Verified sender address | Yes |
| RESEND_TO | Recipient(s), comma-separated | Yes |
| SDLC_SERVER_URL | sdlc server URL (default: http://localhost:7777) | No |
| TELEGRAM_WEBHOOK_ROUTE | Webhook route name (default: telegram) | No |
| WINDOW_HOURS | Digest window in hours (default: 24) | No |

## Output

```json
{
  "ok": true,
  "data": {
    "dry_run": false,
    "total_messages": 42,
    "chat_count": 3,
    "period_start": "2026-03-05T08:00:00.000Z",
    "period_end": "2026-03-06T08:00:00.000Z",
    "sent_to": ["you@example.com"]
  },
  "duration_ms": 1234
}
```
