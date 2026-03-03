# Design: Daily Cron Digest — Telegram → SMTP Email

## Overview

This document describes the technical design for the `sdlc telegram digest` command. The feature adds a new CLI subcommand that pulls messages from one or more Telegram chats via the Bot API and delivers a formatted daily digest email via SMTP.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  sdlc telegram digest [--dry-run] [--window N] [--chat ID]      │
│  crates/sdlc-cli/src/cmd/telegram.rs                            │
└────────────────────────┬────────────────────────────────────────┘
                         │
          ┌──────────────▼──────────────────┐
          │  DigestRunner (async)           │
          │  crates/sdlc-core/src/telegram/ │
          └──┬──────────────┬──────────────┘
             │              │
   ┌──────────▼──────┐  ┌──▼──────────────┐
   │ TelegramClient  │  │  SmtpMailer      │
   │ Bot API (HTTPS) │  │  lettre crate    │
   └──────────┬──────┘  └──────────────────┘
              │
   ┌──────────▼──────────────────────────────┐
   │ getUpdates / getChatHistory (Bot API)    │
   │  → Vec<TelegramMessage>                  │
   └──────────┬──────────────────────────────┘
              │
   ┌──────────▼──────────────────────────────┐
   │ DigestBuilder                            │
   │  → filter by time window                 │
   │  → group by chat                         │
   │  → format plain text + HTML              │
   └──────────┬──────────────────────────────┘
              │
   ┌──────────▼──────────────────────────────┐
   │ RunRecord  (.sdlc/.runs/<ts>-tg-dg.json) │
   │  summary, chat_count, message_count      │
   └─────────────────────────────────────────┘
```

## Module Layout

```
crates/sdlc-core/src/telegram/
    mod.rs          — re-exports public API
    client.rs       — TelegramClient (reqwest-based, async)
    types.rs        — TelegramMessage, TelegramChat, DigestConfig
    digest.rs       — DigestBuilder, ChatDigest, formatted output
    mailer.rs       — SmtpMailer (lettre-based)
    runner.rs       — DigestRunner orchestrating client → builder → mailer

crates/sdlc-cli/src/cmd/telegram.rs
    — Clap subcommand definition + main entry point
    — Loads config, constructs DigestRunner, runs async runtime
```

## Data Structures

### DigestConfig (loaded from SdlcConfig)

```rust
#[derive(Debug, Deserialize)]
pub struct DigestConfig {
    pub bot_token: String,         // TELEGRAM_BOT_TOKEN or config
    pub chat_ids: Vec<String>,
    pub smtp: SmtpConfig,
    pub window_hours: u32,         // default: 24
    pub subject_prefix: String,    // default: "[sdlc Digest]"
    pub max_messages_per_chat: u32, // default: 100
}

#[derive(Debug, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,                 // default: 587
    pub username: String,          // SMTP_USERNAME or config
    pub password: String,          // SMTP_PASSWORD or config
    pub from: String,
    pub to: Vec<String>,
}
```

### TelegramMessage

```rust
#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub date: i64,                 // Unix timestamp
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    pub title: Option<String>,
    pub username: Option<String>,
    pub type_: String,             // "group", "supergroup", "channel", "private"
}
```

### ChatDigest and DigestSummary

```rust
pub struct ChatDigest {
    pub chat_id: String,
    pub chat_name: String,
    pub messages: Vec<DigestMessage>,
}

pub struct DigestMessage {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub text: String,
}

pub struct DigestSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub chats: Vec<ChatDigest>,
    pub total_messages: usize,
}
```

## Telegram Bot API Integration

### Method: `getUpdates`

The Bot API `getUpdates` method returns all updates received by the bot since last call. The bot must be a member of the target chats.

```
GET https://api.telegram.org/bot<token>/getUpdates
  ?offset=<last_update_id+1>
  &limit=100
  &allowed_updates=["message"]
```

**Limitation:** `getUpdates` returns only messages after the bot joined. For groups/channels the bot has been in, messages arrive as updates. The digest command uses a stateless approach — it fetches the last N updates and filters by timestamp. For initial usage, the bot must already be added to the target chats.

**Offset management:** The `getUpdates` call is stateless between runs — no offset is persisted. Each run fetches the most recent `max_messages_per_chat` updates and filters to the configured window. This avoids the need to store state between cron runs, at the cost of potentially missing messages if volume exceeds `max_messages_per_chat` in the window.

### Response parsing

```
{"ok": true, "result": [{
  "update_id": 123456,
  "message": {
    "message_id": 1,
    "from": {"id": 42, "first_name": "Alice", "username": "alice"},
    "chat": {"id": -1001234567890, "title": "Project Alpha", "type": "supergroup"},
    "date": 1740000000,
    "text": "Deployed v1.4 to staging"
  }
}]}
```

## Email Generation

### Multipart MIME structure

```
multipart/alternative
  ├── text/plain  (human-readable, no HTML)
  └── text/html   (styled version with table layout)
```

### Plain text template

```
sdlc Daily Digest — {weekday}, {month} {day} {year}
Period: {start_utc} → {end_utc}

── {chat_name} ({n} messages) ──────────────────────
 {HH:MM}  {author:<12}  {text}
 ...

────────────────────────────────────────────────────
{total} total messages  |  Generated by sdlc telegram digest
```

### HTML template

A simple, inline-CSS HTML email with:
- Header: date and period
- One `<table>` per chat with rows for timestamp, author, message text
- Footer with total count and generation timestamp
- No external resources (images, fonts, CDN assets) — email-safe

## SMTP Integration

Using the `lettre` crate with `AsyncSmtpTransport<Tokio1Executor>` (STARTTLS):

```rust
let creds = Credentials::new(config.username.clone(), config.password.clone());
let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
    .credentials(creds)
    .port(config.port)
    .build();
mailer.send(email).await?
```

For port 465 (SMTPS), uses `AsyncSmtpTransport::relay` instead of `starttls_relay`.

## Run Record

After each execution, a run record is appended to `.sdlc/.runs/`:

```json
{
  "id": "20260303-180000-abc",
  "kind": "telegram-digest",
  "status": "completed",
  "started_at": "2026-03-03T18:00:00Z",
  "completed_at": "2026-03-03T18:00:03Z",
  "summary": "Sent digest: 42 messages across 2 chats",
  "metadata": {
    "period_start": "2026-03-02T18:00:00Z",
    "period_end": "2026-03-03T18:00:00Z",
    "chat_count": 2,
    "message_count": 42,
    "recipients": ["team@example.com"],
    "dry_run": false
  }
}
```

Error runs set `"status": "failed"` and include `"error": "<message>"` (no credentials in error text).

## Configuration Loading and Env Interpolation

The existing `SdlcConfig` struct is extended with an optional `telegram: Option<DigestConfig>` field. At load time, `${ENV_VAR}` placeholders are substituted using `std::env::var`. If a required field has no value (neither config nor env), `ConfigError::MissingField` is returned with a clear message.

Precedence (highest first):
1. CLI flags (`--chat`, `--window`)
2. Environment variables
3. `.sdlc/config.yaml` values

## Error Handling

```rust
pub enum TelegramDigestError {
    Config(ConfigError),
    TelegramApi { status: u16, description: String },
    TelegramNetwork(reqwest::Error),
    SmtpAuth(lettre::transport::smtp::Error),
    SmtpDelivery(lettre::transport::smtp::Error),
    NoMessagesFound,   // not an error — exits 0 with info message
}
```

Each variant maps to exit code as defined in the spec (0/1/2/3).

## Cron Usage Example

```cron
# Send daily digest at 08:00 UTC
0 8 * * * /usr/local/bin/sdlc telegram digest >> /var/log/sdlc-digest.log 2>&1
```

Or using macOS launchd or systemd timer equivalents.

## Dependency Changes

`crates/sdlc-core/Cargo.toml`:
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
lettre = { version = "0.11", features = ["tokio1-native-tls", "builder"] }
```

`crates/sdlc-cli/Cargo.toml`:
No new dependencies — uses `sdlc-core` for all heavy lifting, `tokio` already present.

## Testing Strategy

- Unit tests for `DigestBuilder` — filtering, grouping, formatting (no network)
- Unit tests for config loading + env var interpolation
- Integration test with a mock Telegram API server (`wiremock` or `httpmock`)
- Integration test with `lettre`'s `SmtpTransport::dry_run()` to verify email composition without SMTP server
- All tests in `sdlc-core` — CLI layer is thin glue, not independently tested

## Security Design

- Credentials are never stored in `RunRecord` or logs
- TLS is enforced at the transport level — no runtime option to disable
- `bot_token` in config supports `${TELEGRAM_BOT_TOKEN}` substitution to keep secrets out of committed YAML
- Rate limiting: no retry loops — if Telegram API returns 429, exit with error code 2; the cron scheduler handles retries on next tick

## Alternatives Considered

| Option | Rejected reason |
|---|---|
| MTProto (Telethon-style) | Requires user authentication and phone number; Bot API is sufficient for group/channel monitoring |
| Webhook-based delivery | Requires a persistent HTTP server; cron pull is simpler and sufficient |
| Persisting offset between runs | Adds statefulness complexity; stateless windowing is simpler and more resilient |
| SQLite for message storage | Unnecessary — pull-and-discard is the right model for a digest |
