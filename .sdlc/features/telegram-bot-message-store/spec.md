# Spec: telegram-bot-message-store

## Feature

**Slug:** telegram-bot-message-store
**Title:** Bot registration, polling, and SQLite message storage
**Milestone:** telegram-digest-bot — Telegram Daily Digest Bot

## Problem

The sdlc project needs a Telegram integration that can receive and store messages for later processing. Before a daily digest can be generated and sent, there must be a reliable, persistent store of incoming Telegram messages. Currently, no such infrastructure exists.

## Goal

Implement the foundational layer for the Telegram bot: bot token registration/configuration, a polling loop that receives updates from the Telegram Bot API, and a SQLite database that persistently stores received messages. This layer is the data source that downstream features (`telegram-digest-cron`) will query to generate digests.

## Scope

### In Scope

- **Bot configuration**: Store the Telegram bot token in sdlc's config/secrets system (environment variable or `.sdlc/secrets.yaml`)
- **Telegram polling**: A polling worker that calls `getUpdates` on the Telegram Bot API with long-polling, handles offset tracking, and is resilient to transient network errors
- **SQLite message store**: A lightweight SQLite database (`.sdlc/telegram/messages.db`) that stores received messages with full metadata (chat_id, user_id, message_id, text, date, raw JSON)
- **Deduplication**: Ensure messages are never stored twice using message_id as unique key
- **CLI command**: `sdlc telegram poll` — starts the polling loop in the foreground; `sdlc telegram status` — shows bot info and message count
- **Graceful shutdown**: SIGINT/SIGTERM stops the poller cleanly after finishing the current batch

### Out of Scope

- Digest generation (handled by `telegram-digest-cron`)
- Message formatting or filtering (digest layer concern)
- Web UI for messages
- Multiple bot support (single bot per project)

## Data Model

### SQLite schema (`.sdlc/telegram/messages.db`)

```sql
CREATE TABLE IF NOT EXISTS messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id  INTEGER NOT NULL UNIQUE,
    chat_id     INTEGER NOT NULL,
    user_id     INTEGER,
    username    TEXT,
    first_name  TEXT,
    text        TEXT,
    date        INTEGER NOT NULL,  -- unix timestamp from Telegram
    raw_json    TEXT NOT NULL,
    stored_at   TEXT NOT NULL      -- ISO 8601 UTC
);

CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date);
```

### Config

Bot token stored in environment variable `TELEGRAM_BOT_TOKEN` or `.sdlc/config.yaml` under:

```yaml
telegram:
  bot_token: "<token>"          # or pulled from env
  poll_timeout_secs: 30         # long-poll timeout (default 30)
  db_path: .sdlc/telegram/messages.db
```

## User Flow

1. User sets `TELEGRAM_BOT_TOKEN` environment variable or adds token to `.sdlc/config.yaml`
2. User adds the bot to their Telegram group or channel
3. User runs `sdlc telegram poll` — the polling loop starts, logs received messages
4. Messages accumulate in SQLite as the bot receives them
5. `sdlc telegram status` shows: bot username, message count, oldest/newest message timestamps
6. `telegram-digest-cron` queries the SQLite store to generate digests

## Technical Notes

- Use `rusqlite` for SQLite access (already common in the Rust ecosystem, no server required)
- Use `reqwest` (already in the workspace) for Telegram API HTTP calls
- Polling loop: call `getUpdates?offset=<last+1>&timeout=30`, process results, update offset, repeat
- Offset persistence: store last seen update_id in SQLite or a small YAML file (`.sdlc/telegram/offset.yaml`)
- Error handling: on Telegram API error, log and retry with exponential backoff (max 60s)
- The feature is a new CLI subcommand in `sdlc-cli`, with data layer in `sdlc-core`

## Acceptance Criteria

1. `sdlc telegram poll` starts successfully when `TELEGRAM_BOT_TOKEN` is set
2. Messages sent to the bot appear in the SQLite database within the poll timeout
3. Restarting the poller does not duplicate messages
4. `sdlc telegram status` outputs bot username and total stored message count
5. `SDLC_NO_NPM=1 cargo test --all` passes with new unit tests for the polling/storage logic
6. `cargo clippy --all -- -D warnings` produces no new warnings
