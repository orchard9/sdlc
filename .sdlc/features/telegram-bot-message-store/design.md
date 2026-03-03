# Design: telegram-bot-message-store

## Overview

This document covers the technical design for bot registration, long-polling, and SQLite message storage for the `telegram-digest-bot` milestone. The implementation follows the existing sdlc architecture: new data types and I/O in `sdlc-core`, new CLI subcommand in `sdlc-cli`.

---

## Architecture

### Module placement

```
crates/
  sdlc-core/src/
    telegram.rs          ← NEW: TelegramConfig, MessageStore, polling types, I/O
  sdlc-cli/src/cmd/
    telegram.rs          ← NEW: `sdlc telegram poll` and `sdlc telegram status`
    mod.rs               ← ADD: pub mod telegram
  sdlc-cli/src/main.rs   ← ADD: Telegram subcommand routing
```

No server-side changes are needed for this feature. The message store is a local SQLite file only.

### Dependency additions

| Crate | Added to | Purpose |
|---|---|---|
| `rusqlite = "0.31"` | `sdlc-core/Cargo.toml` | SQLite message persistence |
| `reqwest = { version = "0.12", features = ["json", "blocking"] }` | `sdlc-core/Cargo.toml` | Telegram HTTP API calls |

Note: `reqwest` with `blocking` feature for simplicity in the polling loop (avoids mixing async Tokio runtime complexity into a simple poll-sleep loop). The CLI is already tokio-based so both blocking and async are acceptable.

---

## Data Layer: `crates/sdlc-core/src/telegram.rs`

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub poll_timeout_secs: u64,     // default 30
    pub db_path: PathBuf,           // default .sdlc/telegram/messages.db
}
```

Loaded from (in priority order):
1. Environment variable `TELEGRAM_BOT_TOKEN` for the token
2. `.sdlc/config.yaml` under key `telegram:`
3. Fallback: error if token missing

### SQLite Schema

Managed via `rusqlite`. Schema is applied on first open (idempotent `CREATE TABLE IF NOT EXISTS`):

```sql
CREATE TABLE IF NOT EXISTS messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id  INTEGER NOT NULL UNIQUE,
    chat_id     INTEGER NOT NULL,
    user_id     INTEGER,
    username    TEXT,
    first_name  TEXT,
    text        TEXT,
    date        INTEGER NOT NULL,
    raw_json    TEXT NOT NULL,
    stored_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date);

CREATE TABLE IF NOT EXISTS poll_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

`poll_state` stores the last `update_id` offset under key `"last_update_id"`.

### Public API

```rust
pub struct MessageStore {
    conn: rusqlite::Connection,
}

impl MessageStore {
    /// Open (or create) the SQLite database, applying schema migrations.
    pub fn open(path: &Path) -> Result<Self, SdlcError>;

    /// Insert a message. Returns Ok(true) if inserted, Ok(false) if duplicate.
    pub fn insert_message(&self, msg: &TelegramMessage) -> Result<bool, SdlcError>;

    /// Return total stored message count.
    pub fn message_count(&self) -> Result<u64, SdlcError>;

    /// Return oldest and newest message timestamps (unix).
    pub fn time_range(&self) -> Result<Option<(i64, i64)>, SdlcError>;

    /// Get the last stored update_id offset, or None if never polled.
    pub fn get_offset(&self) -> Result<Option<i64>, SdlcError>;

    /// Persist the latest update_id offset.
    pub fn set_offset(&self, offset: i64) -> Result<(), SdlcError>;
}
```

### Telegram API types

Minimal deserialization of the Telegram Bot API `getUpdates` response:

```rust
#[derive(Debug, Deserialize)]
pub struct GetUpdatesResponse {
    pub ok: bool,
    pub result: Vec<Update>,
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub chat: Chat,
    pub from: Option<User>,
    pub text: Option<String>,
    pub date: i64,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
}
```

`TelegramMessage` is the flat struct stored in SQLite, derived from an `Update`.

### Polling function

```rust
pub fn poll_once(
    config: &TelegramConfig,
    store: &MessageStore,
) -> Result<usize, SdlcError>;
// Returns count of new messages stored.

pub fn poll_loop(
    config: &TelegramConfig,
    store: &MessageStore,
    shutdown: Arc<AtomicBool>,
) -> Result<(), SdlcError>;
// Loops until shutdown is set. Calls poll_once, handles errors with backoff.
```

Polling sequence:
1. Read `offset` from store (last `update_id + 1`, or `0` if none)
2. Call `GET https://api.telegram.org/bot<token>/getUpdates?offset=<n>&timeout=<t>`
3. For each update: extract message, call `store.insert_message()`
4. Update offset to `last_update_id + 1`
5. On HTTP error or deserialization failure: log error, sleep with backoff (1s, 2s, 4s, ... max 60s), reset backoff on success

### Bot info function

```rust
pub struct BotInfo {
    pub id: i64,
    pub username: String,
    pub first_name: String,
}

pub fn get_me(config: &TelegramConfig) -> Result<BotInfo, SdlcError>;
```

---

## CLI Layer: `crates/sdlc-cli/src/cmd/telegram.rs`

### Subcommand enum

```rust
#[derive(Subcommand)]
pub enum TelegramSubcommand {
    /// Start polling the Telegram Bot API and storing messages (runs until Ctrl-C)
    Poll,

    /// Show bot info and stored message statistics
    Status,
}
```

### `sdlc telegram poll`

1. Load config (env + `.sdlc/config.yaml`)
2. Create/open `MessageStore` at configured `db_path`
3. Call `get_me()` to verify token is valid — print bot username
4. Set up SIGINT/SIGTERM handler setting an `AtomicBool` shutdown flag
5. Call `poll_loop(config, store, shutdown)` — blocks until signal
6. Print total message count on exit

Output example:
```
Bot: @my_digest_bot (id: 123456789)
Polling for messages... (Ctrl-C to stop)
[2026-03-02 12:01:05] Stored 3 new messages from chat -1001234567890
[2026-03-02 12:01:35] No new messages
^C
Stopped. Total messages stored: 47
```

### `sdlc telegram status`

1. Load config
2. Call `get_me()` — print bot username + id
3. Open `MessageStore` (read-only)
4. Print: message count, oldest message date, newest message date, current offset

Output example:
```
Bot:      @my_digest_bot (id: 123456789)
Messages: 47 stored
Oldest:   2026-03-01 09:15:22 UTC
Newest:   2026-03-02 11:58:04 UTC
Offset:   142 (last update_id + 1)
```

---

## Error Handling

- Missing `TELEGRAM_BOT_TOKEN` and no config → clear error: `"TELEGRAM_BOT_TOKEN is not set. Set the environment variable or add telegram.bot_token to .sdlc/config.yaml"`
- Telegram API 401 Unauthorized → `"Invalid bot token. Check TELEGRAM_BOT_TOKEN."`
- SQLite open failure (permissions, corrupt file) → surface the `rusqlite` error with context
- All errors use `SdlcError` and propagate via `?`; no `unwrap()` in library code

---

## Config integration

The existing `crates/sdlc-core/src/config.rs` `SdlcConfig` struct gets a new optional field:

```rust
#[serde(default)]
pub telegram: Option<TelegramConfigYaml>,
```

Where `TelegramConfigYaml` holds `bot_token`, `poll_timeout_secs`, `db_path` (all optional, with defaults applied at load time).

---

## Testing strategy

- **Unit tests in `sdlc-core`**: `MessageStore` open, insert, deduplication (insert same message_id twice), offset read/write, `time_range` with empty DB
- **Integration tests in `sdlc-cli`**: CLI invocation with `assert_cmd`; verify `sdlc telegram status` fails cleanly when token is missing
- No live Telegram API calls in tests — polling functions take the HTTP client as a parameter (dependency injection) for testability, or use a mock via `wiremock`/`httpmock`

Tests run with `SDLC_NO_NPM=1 cargo test --all`.

---

## Sequence: message ingestion

```
sdlc telegram poll
       │
       ▼
  get_me() → verify token, print bot name
       │
       ┌──────────────────────────────┐
       │  poll_loop                   │
       │                              │
       │  offset = store.get_offset() │
       │  → GET getUpdates?offset=N   │
       │  ← [{update_id, message}...] │
       │  for each update:            │
       │    store.insert_message()    │
       │  store.set_offset(max+1)     │
       │  (repeat until shutdown)     │
       └──────────────────────────────┘
       │
       ▼
  shutdown signal → exit cleanly
```

---

## File layout after implementation

```
crates/sdlc-core/src/telegram.rs          ← all Telegram data types + store + polling
crates/sdlc-core/src/lib.rs               ← pub mod telegram added
crates/sdlc-cli/src/cmd/telegram.rs       ← CLI poll + status subcommands
crates/sdlc-cli/src/cmd/mod.rs            ← pub mod telegram added
crates/sdlc-cli/src/main.rs               ← Telegram arm in match
.sdlc/telegram/messages.db                ← runtime artifact (gitignored)
```

`.sdlc/telegram/` should be added to `.gitignore`.
