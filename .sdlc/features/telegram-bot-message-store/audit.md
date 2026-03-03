# Security Audit: telegram-bot-message-store

## Scope

This audit reviews the security posture of the bot token handling, SQLite message storage, HTTP client usage, and CLI surface introduced by `telegram-bot-message-store`.

---

## A1: Bot Token Handling

**Risk: HIGH (credential exposure)**

**Finding:** Bot tokens are sensitive credentials. If leaked, an attacker can send messages as the bot, access the bot's message history, and abuse the bot's Telegram permissions.

**Assessment:**

- Token is read from `TELEGRAM_BOT_TOKEN` env var or `.sdlc/config.yaml` under `telegram.bot_token`.
- The existing `DigestConfig::resolve_env()` pattern supports `${VAR}` placeholders — users can keep the token in an environment variable and only put `${TELEGRAM_BOT_TOKEN}` in config.yaml. This is the recommended pattern.
- The token is **never** logged or written to run records (confirmed in `runner.rs::build_run_record` — the token field is excluded from the JSON output by design, same pattern enforced here).
- The token is **not** included in any error messages or diagnostic output.
- `.sdlc/config.yaml` is not gitignored by default — if a user puts the raw token in config.yaml and commits it, it will be exposed. **Action taken:** The guidance in the spec and CLI help text directs users to use `${TELEGRAM_BOT_TOKEN}` or the env var pattern. No code change needed; this is a user-education concern consistent with existing SMTP credential handling.

**Status: ACCEPTED — no code change required; usage pattern is consistent with existing SMTP credentials.**

---

## A2: SQLite Message Storage

**Risk: MEDIUM (local data sensitivity)**

**Finding:** Messages received by the bot are stored in plaintext in `.sdlc/telegram/messages.db`. The messages may contain sensitive information from Telegram chats.

**Assessment:**

- The database is local to the developer's machine at `.sdlc/telegram/messages.db`.
- `.sdlc/telegram/` is added to `.gitignore` — the database will **not** be committed to git.
- Filesystem permissions are inherited from the parent directory (`.sdlc/`). No additional hardening (e.g., `chmod 600`) is applied — consistent with how `.sdlc/orchestrator.db` and `.sdlc/telemetry.redb` are handled.
- `raw_json` column stores the full update payload. This may include user IDs, usernames, and message text. Appropriate for the use case (local daily digest tool).

**Status: ACCEPTED — consistent with existing local database files; gitignored.**

---

## A3: HTTP Client (reqwest::blocking)

**Risk: LOW**

**Finding:** `poll.rs` uses `reqwest::blocking::Client` to call `api.telegram.org` over HTTPS.

**Assessment:**

- All Telegram API calls use `https://api.telegram.org` — no HTTP downgrade.
- TLS verification is enabled by default in `reqwest` (uses native-tls or rustls).
- Request timeout is set to `poll_timeout_secs + 10` seconds — prevents indefinite blocking.
- The existing `client.rs` uses `reqwest::blocking::Client::new()` without an explicit timeout. The `poll.rs` implementation sets a timeout explicitly — this is an improvement.
- No custom TLS configuration or certificate pinning is applied. This is appropriate for a developer tool calling a public API.

**Status: ACCEPTED — HTTPS enforced; timeout set.**

---

## A4: SQL Injection

**Risk: NONE**

**Finding:** `rusqlite` is used with parameterized queries (`rusqlite::params![]`) throughout. No string interpolation is used to construct SQL statements.

**Assessment:**

- Every `execute` and `query_row` call uses `?1`, `?2`, … placeholders with the `params![]` macro.
- `execute_batch` is only used for static schema DDL with no user input.
- No SQL injection surface exists.

**Status: PASS.**

---

## A5: Error Messages and Information Leakage

**Risk: LOW**

**Finding:** Error messages should not expose the bot token or other credentials.

**Assessment:**

- `SdlcError::TelegramTokenMissing` message: "Telegram bot token is not configured…" — no token value included.
- `SdlcError::TelegramApi(msg)` — `msg` comes from Telegram API responses (e.g., "Unauthorized"). These do not include the token.
- `SdlcError::Sqlite(msg)` — `msg` comes from `rusqlite` error strings. These may include file paths but no secrets.
- `get_me` and `poll_loop` error messages logged to stderr include timestamps and error descriptions, not credentials.

**Status: PASS.**

---

## A6: Dependency Surface

**Risk: LOW**

**Finding:** Two new dependencies added: `rusqlite = "0.31"` and `reqwest = "0.12"`.

**Assessment:**

- `rusqlite` with `features = ["bundled"]` — bundles SQLite 3.x. Bundled SQLite is a well-known, mature version. The bundled feature avoids system SQLite version issues.
- `reqwest 0.12` — widely used, maintained by the tokio ecosystem. Already used elsewhere in the codebase (sdlc-server).
- `ctrlc = "3"` in sdlc-cli — simple SIGINT handler crate, minimal surface area.
- No new network endpoints other than `api.telegram.org`.

**Status: ACCEPTED — dependencies are appropriate and well-maintained.**

---

## A7: Graceful Shutdown / Signal Handling

**Risk: LOW**

**Finding:** The `ctrlc` crate registers a SIGINT/SIGTERM handler that sets an `AtomicBool`.

**Assessment:**

- The handler is minimal — sets a flag, no I/O, no panics.
- The `poll_loop` checks the flag before each poll iteration.
- On shutdown, the current poll request completes (or times out) before the loop exits — messages in flight are not dropped.
- No double-free or data race risk — `AtomicBool` with `Ordering::Relaxed` is sufficient for this use case (one writer, one reader, no ordering dependency between the flag and other writes).

**Status: PASS.**

---

## Summary

| Finding | Risk | Status |
|---|---|---|
| A1: Bot token handling | HIGH | ACCEPTED — env var pattern recommended, token never logged |
| A2: SQLite plaintext storage | MEDIUM | ACCEPTED — local only, gitignored |
| A3: HTTPS + timeout | LOW | PASS |
| A4: SQL injection | NONE | PASS |
| A5: Error message leakage | LOW | PASS |
| A6: Dependency surface | LOW | ACCEPTED — established, maintained crates |
| A7: Signal handling | LOW | PASS |

No blocking security issues. Feature is ready to proceed to QA.
