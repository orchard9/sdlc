# Tasks: telegram-bot-message-store

## T1: Add rusqlite and reqwest dependencies to sdlc-core

Add `rusqlite = "0.31"` and `reqwest = { version = "0.12", features = ["json", "blocking"] }` to `crates/sdlc-core/Cargo.toml`. Verify workspace compiles.

**Files:** `crates/sdlc-core/Cargo.toml`

---

## T2: Implement TelegramConfig and config loading

Add `TelegramConfigYaml` struct (optional fields: `bot_token`, `poll_timeout_secs`, `db_path`) to `crates/sdlc-core/src/config.rs`. Add `telegram: Option<TelegramConfigYaml>` field to `SdlcConfig`. Implement `TelegramConfig::from_env_and_config()` that merges env var `TELEGRAM_BOT_TOKEN` with config file, applies defaults (`poll_timeout_secs=30`, `db_path=.sdlc/telegram/messages.db`), and errors clearly if token is missing.

**Files:** `crates/sdlc-core/src/config.rs`, `crates/sdlc-core/src/telegram.rs`

---

## T3: Implement MessageStore (SQLite layer)

Create `crates/sdlc-core/src/telegram.rs` with:
- `MessageStore` struct wrapping `rusqlite::Connection`
- `MessageStore::open(path: &Path) -> Result<Self, SdlcError>` — creates parent dirs, opens/creates SQLite, applies schema (messages table + poll_state table + indexes)
- `insert_message(&self, msg: &TelegramMessage) -> Result<bool, SdlcError>` — INSERT OR IGNORE, returns true if inserted
- `message_count(&self) -> Result<u64, SdlcError>`
- `time_range(&self) -> Result<Option<(i64, i64)>, SdlcError>`
- `get_offset(&self) -> Result<Option<i64>, SdlcError>`
- `set_offset(&self, offset: i64) -> Result<(), SdlcError>`

**Files:** `crates/sdlc-core/src/telegram.rs`, `crates/sdlc-core/src/lib.rs`

---

## T4: Implement Telegram API types and polling functions

In `telegram.rs`, add:
- Deserialize types: `GetUpdatesResponse`, `Update`, `Message`, `Chat`, `User`
- `TelegramMessage` (flat store struct derived from Update)
- `BotInfo` struct + `get_me(config: &TelegramConfig) -> Result<BotInfo, SdlcError>`
- `poll_once(config, store) -> Result<usize, SdlcError>` — single getUpdates call, inserts results, updates offset
- `poll_loop(config, store, shutdown: Arc<AtomicBool>) -> Result<(), SdlcError>` — loops with exponential backoff on errors

**Files:** `crates/sdlc-core/src/telegram.rs`

---

## T5: Implement `sdlc telegram poll` CLI subcommand

Create `crates/sdlc-cli/src/cmd/telegram.rs` with `TelegramSubcommand::Poll`. Implementation:
1. Load config via `TelegramConfig::from_env_and_config()`
2. Open `MessageStore`
3. Call `get_me()`, print bot username
4. Register ctrlc handler setting `Arc<AtomicBool>` shutdown flag
5. Call `poll_loop()`
6. Print final message count on exit

Register the subcommand in `mod.rs` and `main.rs`.

**Files:** `crates/sdlc-cli/src/cmd/telegram.rs`, `crates/sdlc-cli/src/cmd/mod.rs`, `crates/sdlc-cli/src/main.rs`

---

## T6: Implement `sdlc telegram status` CLI subcommand

In `telegram.rs`, add `TelegramSubcommand::Status`. Implementation:
1. Load config
2. Call `get_me()`, print bot id + username
3. Open `MessageStore`
4. Print message count, oldest/newest timestamps (formatted as UTC), current offset

**Files:** `crates/sdlc-cli/src/cmd/telegram.rs`

---

## T7: Add unit tests for MessageStore

In `crates/sdlc-core/src/telegram.rs`, add `#[cfg(test)] mod tests` with:
- `test_open_creates_schema` — open DB in tempdir, verify tables exist
- `test_insert_deduplication` — insert same message_id twice, assert count=1 and returns false on second insert
- `test_offset_round_trip` — set_offset then get_offset
- `test_time_range_empty` — returns None on empty DB
- `test_time_range_with_messages` — insert two messages, verify time_range

**Files:** `crates/sdlc-core/src/telegram.rs`

---

## T8: Add CLI integration test for missing token

In `crates/sdlc-cli/tests/` (or inline), add an `assert_cmd` test verifying that `sdlc telegram status` exits with a non-zero code and a helpful error message when `TELEGRAM_BOT_TOKEN` is unset and no config file has a token.

**Files:** `crates/sdlc-cli/tests/telegram_test.rs` (or existing integration test file)

---

## T9: Add .sdlc/telegram/ to .gitignore

Add `.sdlc/telegram/` to `.gitignore` so the runtime SQLite database is not committed.

**Files:** `.gitignore`

---

## T10: Verify build and clippy clean

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`. Fix any warnings or failures.

**Files:** as needed
