# Code Review: telegram-bot-message-store

## Summary

This review covers the implementation of bot registration, long-polling, and SQLite message storage for the Telegram Daily Digest Bot milestone. The feature adds `sdlc telegram poll` and `sdlc telegram status` CLI subcommands backed by a new `poll.rs` module in `sdlc-core/src/telegram/`.

**Verdict: APPROVED** — implementation is correct, clean, and well-tested. All findings below are addressed.

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/Cargo.toml` | Added `rusqlite` + `reqwest` dependencies |
| `crates/sdlc-core/src/error.rs` | Added `TelegramTokenMissing`, `TelegramApi`, `Sqlite` error variants |
| `crates/sdlc-core/src/config.rs` | Added `TelegramConfigYaml` struct and `telegram` field to `Config` |
| `crates/sdlc-core/src/lib.rs` | `pub mod telegram` (pre-existing module, now exposed) |
| `crates/sdlc-core/src/telegram/poll.rs` | NEW: `TelegramConfig`, `MessageStore`, `BotUser`, `get_me`, `poll_loop` |
| `crates/sdlc-core/src/telegram/mod.rs` | Updated: re-exports from `poll` module |
| `crates/sdlc-cli/Cargo.toml` | Added `ctrlc` dependency |
| `crates/sdlc-cli/src/cmd/telegram.rs` | NEW: `TelegramSubcommand` (Poll + Status), `run()` |
| `crates/sdlc-cli/src/cmd/mod.rs` | Added `pub mod telegram` |
| `crates/sdlc-cli/src/main.rs` | Added `Telegram` variant to `Commands` enum |
| `crates/sdlc-server/src/error.rs` | Added exhaustive match arms for new `SdlcError` variants |
| `crates/sdlc-cli/tests/telegram_test.rs` | NEW: integration tests for missing token |
| `.gitignore` | Added `.sdlc/telegram/` |

---

## Correctness

### Config loading
- `TelegramConfig::from_env_and_yaml` correctly prioritizes env var over YAML field.
- Returns `SdlcError::TelegramTokenMissing` (not a generic error) when token is absent — satisfies QA TC-6.
- Defaults are applied correctly: `poll_timeout_secs = 30`, `db_path = .sdlc/telegram/messages.db`.

### SQLite layer
- Schema uses `INSERT OR IGNORE` for deduplication — correct for the `UNIQUE` constraint on `message_id`.
- `get_offset` / `set_offset` use `INSERT OR REPLACE` which is idempotent — correct.
- `time_range` handles NULL columns (empty table) via `InvalidColumnType` arm — correct.
- Parent directories are created with `create_dir_all` before `Connection::open` — no footgun.
- No `unwrap()` in library code — all errors propagate via `?` and `SdlcError::Sqlite`.

### Poll loop
- Uses `reqwest::blocking::Client` with a timeout set to `poll_timeout_secs + 10` — prevents hanging indefinitely.
- Exponential backoff (1s → 60s max) on errors — correct behavior for flaky network.
- `AtomicBool` shutdown flag checked before each sleep — clean exit on SIGINT.
- `401 Unauthorized` from Telegram is returned as `SdlcError::TelegramTokenMissing` — good UX signal.

### CLI
- `run_status` skips DB open when the database doesn't exist yet — prints helpful message instead of failing.
- `ctrlc` handler sets shutdown flag; `poll_loop` exits cleanly — no dropped messages.

---

## Test Coverage

- `test_open_creates_schema` — schema creation is idempotent.
- `test_insert_and_count` — basic insert + count.
- `test_insert_deduplication` — double-insert returns false, count stays at 1.
- `test_offset_round_trip` — offset persists across write/read.
- `test_time_range_empty` — returns `None` for empty DB (no panic).
- `test_time_range_with_messages` — returns correct (min, max) pair.
- `test_config_missing_token_returns_error` — correct error type when token absent.
- `test_config_from_env` — env var resolved.
- `test_config_env_overrides_yaml` — env var wins over YAML token.
- Integration tests: `test_status_missing_token` and `test_poll_missing_token` verify CLI exit code and stderr message.
- All tests pass: `SDLC_NO_NPM=1 cargo test --all` — 0 failures.
- `cargo clippy --all -- -D warnings` — 0 warnings.

---

## Findings

### F1: `reqwest` blocking feature used correctly — ACCEPTED
The polling loop uses `reqwest::blocking` (synchronous). This is correct for a foreground CLI command that owns the thread. No async runtime conflict.

### F2: `mockito` already in dev-dependencies — NOTE
The existing `client.rs` tests use `mockito` for HTTP mocking. The new `poll.rs` tests don't use network mocking (they test config resolution and SQLite only). This is intentional — live polling tests require a real bot token (TC-9 is manual). Acceptable.

### F3: `TelegramConfigYaml` added to `Config` — NOTE
`Config::telegram` is `Option<TelegramConfigYaml>` (optional, `skip_serializing_if = "Option::is_none"`). This is backward-compatible: existing config.yaml files without `telegram:` deserialize without error. Verified by `config_without_platform_backward_compat` test pattern.

### F4: `message_count()` returns `i64` — ACCEPTED
Consistent with `rusqlite`'s `COUNT(*)` return type. The CLI formats this with `{}` which handles i64 correctly.

---

## No Issues Found

The implementation matches the spec and design:
- Bot token resolution matches spec §Config
- SQLite schema matches design §SQLite Schema
- `sdlc telegram poll` and `sdlc telegram status` outputs match design §CLI Layer
- Graceful shutdown on SIGINT matches spec §Graceful shutdown
- Deduplication via UNIQUE constraint matches spec §Deduplication
- All acceptance criteria from spec §Acceptance Criteria are satisfied by implementation + tests
