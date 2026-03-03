# QA Plan: Daily Cron Digest — Telegram → SMTP Email

## Scope

This QA plan covers verification of the `sdlc telegram digest` command, including Telegram API integration, digest formatting, SMTP email delivery, configuration loading, error handling, and run record persistence.

## Test Environment

- Rust toolchain (stable, as per `rust-toolchain.toml`)
- `SDLC_NO_NPM=1 cargo test --all` — no npm/Node required
- Mock Telegram API server via `wiremock` crate
- `lettre` dry-run transport for SMTP tests (no real SMTP server)
- Isolated `.sdlc/` directory via `tempfile::TempDir`

---

## TC-1: Dependency Resolution

**Objective:** Confirm new crate dependencies build cleanly.

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo build --all`

**Expected:** Zero compile errors. `reqwest` and `lettre` resolved.

---

## TC-2: Config Loading — Happy Path

**Objective:** Config parsed correctly with valid `telegram` block.

**Steps:**
1. Create a temp `.sdlc/config.yaml` with a complete `telegram` block (no env vars).
2. Load config via `SdlcConfig::load`.

**Expected:** `DigestConfig` populated with all fields. No error.

---

## TC-3: Config Loading — Env Var Override

**Objective:** `${ENV_VAR}` in config is resolved from environment.

**Steps:**
1. Set `TELEGRAM_BOT_TOKEN=test-token` and `SMTP_PASSWORD=secret` in test env.
2. Config has `bot_token: "${TELEGRAM_BOT_TOKEN}"` and `password: "${SMTP_PASSWORD}"`.
3. Load config.

**Expected:** `bot_token == "test-token"`, `password == "secret"`.

---

## TC-4: Config Loading — Missing Required Field

**Objective:** Missing `bot_token` produces a clear error.

**Steps:**
1. Create config with `telegram.bot_token` omitted and env var unset.
2. Load config.

**Expected:** `SdlcError::ConfigError` with message referencing `bot_token`. Exit code 1.

---

## TC-5: Telegram Client — Successful `getUpdates`

**Objective:** Client fetches and parses updates correctly.

**Steps:**
1. Start `wiremock` server returning a valid `getUpdates` JSON response (2 messages in 2 chats).
2. Call `TelegramClient::get_updates(100)`.

**Expected:** Returns `Vec<TelegramUpdate>` with 2 entries. Message IDs, chat IDs, and text match fixture.

---

## TC-6: Telegram Client — 401 Unauthorized

**Objective:** Invalid token returns a mapped error.

**Steps:**
1. `wiremock` returns HTTP 401.
2. Call `get_updates`.

**Expected:** `SdlcError::TelegramApi { status: 401, .. }`. Exit code 2.

---

## TC-7: Telegram Client — 429 Rate Limit

**Objective:** Rate-limited response returns correct error variant.

**Steps:**
1. `wiremock` returns HTTP 429.
2. Call `get_updates`.

**Expected:** `SdlcError::TelegramRateLimit`. Exit code 2. No retry.

---

## TC-8: Telegram Client — Malformed JSON Response

**Objective:** Malformed API response returns a deserialization error.

**Steps:**
1. `wiremock` returns `{"ok": true, "result": "not-an-array"}`.
2. Call `get_updates`.

**Expected:** `SdlcError` variant wrapping a serde parse error.

---

## TC-9: DigestBuilder — Time Window Filtering

**Objective:** Messages outside the configured window are excluded.

**Steps:**
1. Create 5 updates: 2 within 24h window, 3 older than 24h.
2. `DigestBuilder::build` with `window_hours: 24`, `now = fixed timestamp`.

**Expected:** `DigestSummary.total_messages == 2`.

---

## TC-10: DigestBuilder — Chat ID Filtering

**Objective:** Messages from unspecified chats are excluded.

**Steps:**
1. Create updates from 3 different chat IDs.
2. Configure `chat_ids` with only 2 of the 3 IDs.
3. Call `DigestBuilder::build`.

**Expected:** Only messages from the 2 configured chats appear.

---

## TC-11: DigestBuilder — max_messages_per_chat Truncation

**Objective:** Excess messages are truncated to the most recent.

**Steps:**
1. Create 150 messages in one chat within the window.
2. Configure `max_messages_per_chat: 100`.
3. Call `DigestBuilder::build`.

**Expected:** `ChatDigest.messages.len() == 100`. The 100 most recent (by timestamp) are kept.

---

## TC-12: DigestBuilder — Empty Updates

**Objective:** No messages returns a valid empty summary.

**Steps:**
1. Call `DigestBuilder::build` with empty `Vec<TelegramUpdate>`.

**Expected:** `DigestSummary.total_messages == 0`. `chats` is empty.

---

## TC-13: Email Formatter — Subject Line

**Objective:** Subject matches spec template.

**Steps:**
1. Create a `DigestSummary` with `total_messages: 42`, `chats.len(): 2`, `period_end: 2026-03-03T18:00:00Z`.
2. Call `format_subject("[sdlc Digest]")`.

**Expected:** `"[sdlc Digest] 2026-03-03 — 42 messages across 2 chats"`.

---

## TC-14: Email Formatter — Plain Text Golden File

**Objective:** Plain text output matches expected format.

**Steps:**
1. Create a deterministic `DigestSummary` fixture (fixed timestamps, authors, text).
2. Call `format_plain_text()`.
3. Compare with a stored golden-file snapshot.

**Expected:** Output matches snapshot exactly.

---

## TC-15: Email Formatter — HTML Golden File

**Objective:** HTML output is valid and matches expected format.

**Steps:**
1. Same fixture as TC-14.
2. Call `format_html()`.
3. Compare with stored golden-file snapshot.

**Expected:** HTML contains per-chat tables. No external resource references. Matches snapshot.

---

## TC-16: SmtpMailer — Message Construction (Dry Run)

**Objective:** Email message is built correctly without sending.

**Steps:**
1. Create `SmtpMailer` with a valid config (fake credentials).
2. Use `lettre` dry-run transport.
3. Call `send("Test Subject", "plain text", "<html>body</html>")`.

**Expected:** No error. Message has correct `From`, `To`, `Subject`, and multipart body.

---

## TC-17: SmtpMailer — Auth Failure

**Objective:** SMTP auth failure returns `SdlcError::SmtpAuthError`.

**Steps:**
1. Configure `lettre` transport to reject with an auth error.
2. Call `SmtpMailer::send`.

**Expected:** `SdlcError::SmtpAuthError`. Exit code 3.

---

## TC-18: DigestRunner — Dry Run End-to-End

**Objective:** `--dry-run` produces a digest summary without sending email.

**Steps:**
1. `wiremock` returns 3 messages within the window.
2. `DigestRunner::run(dry_run: true)`.

**Expected:** `DigestRunResult.dry_run == true`. `sent_to` is empty. No SMTP call made.

---

## TC-19: DigestRunner — Full Run End-to-End

**Objective:** Full run fetches, formats, and sends email.

**Steps:**
1. `wiremock` returns 3 messages.
2. `lettre` dry-run transport.
3. `DigestRunner::run(dry_run: false)`.

**Expected:** `DigestRunResult.sent_to == ["team@example.com"]`. Summary has `total_messages: 3`.

---

## TC-20: Run Record Persistence

**Objective:** Run record is written to `.sdlc/.runs/`.

**Steps:**
1. Use `TempDir` for isolated `.sdlc/` directory.
2. Run `DigestRunner::run(dry_run: false)`.
3. List `.sdlc/.runs/` for `*telegram-digest*.json`.

**Expected:** File exists. Parse as JSON. Fields: `kind == "telegram-digest"`, `status == "completed"`, `metadata.message_count == 3`. No credentials in file.

---

## TC-21: Run Record — Failure Case

**Objective:** Failed run is recorded with `status: "failed"`.

**Steps:**
1. `wiremock` returns 401 (Telegram auth failure).
2. Run `DigestRunner::run`.
3. Check run record.

**Expected:** `status == "failed"`. `error` field present. No credentials in record.

---

## TC-22: CLI — `--dry-run` Prints Digest to Stdout

**Objective:** CLI dry-run prints to stdout and exits 0.

**Steps:**
1. Create temp `.sdlc/config.yaml` with valid `telegram` block and mock bot token.
2. `wiremock` serves 5 messages.
3. Run `sdlc telegram digest --dry-run`.

**Expected:** Stdout contains digest text. Exit code 0.

---

## TC-23: CLI — Missing Config Exits 1

**Objective:** Missing config exits 1 with actionable message.

**Steps:**
1. Run `sdlc telegram digest` with no `.sdlc/config.yaml` and no env vars.

**Expected:** Stderr contains error about missing `bot_token`. Exit code 1.

---

## TC-24: CLI — `--json` Output Format

**Objective:** `--json` flag emits valid JSON summary.

**Steps:**
1. Valid config. `wiremock` returns 3 messages. `--dry-run --json`.
2. Run `sdlc telegram digest --dry-run --json`.

**Expected:** Stdout is valid JSON with `total_messages`, `chat_count`, `period_start`, `period_end`.

---

## TC-25: Clippy — Zero Warnings

**Objective:** Codebase is clean.

**Steps:**
1. Run `cargo clippy --all -- -D warnings`.

**Expected:** Zero warnings, zero errors.

---

## Test Execution

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All 25 test cases must pass before QA approval.

## Pass Criteria

- All unit and integration tests pass (`cargo test --all`)
- Zero clippy warnings
- TC-20 confirms no credential leakage in run records
- TC-18 confirms dry-run does not send email
- TC-22 confirms CLI exit code 0 on success
- TC-23 confirms CLI exit code 1 on config error
