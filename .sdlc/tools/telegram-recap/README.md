# telegram-recap

Fetches Telegram messages via the Bot API, optionally persists them to CouchDB, and emails a
digest via [Resend](https://resend.com). Fully self-contained — no `sdlc` CLI calls.

## Prerequisites

1. A Telegram bot created via [@BotFather](https://t.me/botfather) and added to the chats you want to digest
2. A [Resend](https://resend.com) account with a verified sender domain

## Secret setup

Store all configuration in the `telegram` secrets env:

```bash
sdlc secrets env set telegram TELEGRAM_BOT_TOKEN="<your-bot-token>"
sdlc secrets env set telegram RESEND_API_KEY="re_<your-api-key>"
sdlc secrets env set telegram RESEND_FROM="digest@yourdomain.com"
sdlc secrets env set telegram RESEND_TO="you@example.com"

# Optional: CouchDB for message persistence across runs
sdlc secrets env set telegram COUCHDB_URL="http://couchdb.threesix.svc.cluster.local:5984"
sdlc secrets env set telegram COUCHDB_USER="admin"
sdlc secrets env set telegram COUCHDB_PASSWORD="<password>"

# Optional: override default 24-hour window
sdlc secrets env set telegram WINDOW_HOURS="48"
```

| Variable | Description | Required |
|---|---|---|
| `TELEGRAM_BOT_TOKEN` | Bot API token from @BotFather | Yes |
| `RESEND_API_KEY` | Resend API key (starts with `re_`) | Yes |
| `RESEND_FROM` | Verified sender address | Yes |
| `RESEND_TO` | Recipient(s), comma-separated | Yes |
| `COUCHDB_URL` | CouchDB for message history across runs | No |
| `COUCHDB_USER` | CouchDB username | No |
| `COUCHDB_PASSWORD` | CouchDB password | No |
| `WINDOW_HOURS` | Default digest window in hours (default 24) | No |

## Run setup

```bash
sdlc tool run telegram-recap --setup
```

Returns `{ ok: true, data: { bot_username, couchdb: 'connected'|'unavailable'|'not_configured' } }` on success.

## One-shot test

```bash
sdlc tool run telegram-recap --input '{"dry_run":true}'
```

Fetches messages and prints the digest to the log without sending email.

## Schedule daily digest

The canonical use case — recurrence via the orchestrator, not the tool:

```bash
sdlc orchestrate add daily-telegram-digest \
  --tool telegram-recap \
  --input '{}' \
  --at "now" \
  --every 86400
```

## CouchDB optional setup

CouchDB provides message persistence across runs so the digest window can span multiple polling
intervals. Without it, the digest only includes messages fetched in the current invocation.

Deploy via Helm into the cluster:

```bash
helm upgrade --install couchdb k3s-fleet/deployments/helm/couchdb \
  --namespace threesix \
  --set credentials.user=admin \
  --set credentials.password=<password>
```

Then set `COUCHDB_URL=http://couchdb.threesix.svc.cluster.local:5984` in secrets.

If CouchDB is unavailable at runtime, the tool falls back to the current poll and continues —
it never fails because of an optional dependency.

## Output

```json
{
  "ok": true,
  "data": {
    "dry_run": false,
    "total_messages": 42,
    "chat_count": 3,
    "period_start": "2026-03-01T08:00:00.000Z",
    "period_end": "2026-03-02T08:00:00.000Z",
    "sent_to": ["you@example.com"]
  },
  "duration_ms": 1234
}
```
