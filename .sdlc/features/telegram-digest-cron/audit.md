# Security Audit: Daily Cron Digest — Telegram → SMTP Email

## Scope

Security audit of `telegram-digest-cron`: the `sdlc telegram digest` command, its Rust implementation in `sdlc-core/telegram/`, and the CLI binding in `sdlc-cli/cmd/telegram.rs`.

The audit focuses on: credential handling, injection risks, data leakage, dependency surface, input validation, and error handling security.

---

## SA-1 — Credential Storage and Exposure

**Area:** Bot token and SMTP credentials

**Finding:** Credentials (bot_token, smtp.username, smtp.password) are loaded from environment variables or `.sdlc/config.yaml`. They are never written to:
- Run records (`.sdlc/.runs/`) — verified: `build_run_record` receives `&DigestConfig` but only emits `smtp.host` and `smtp.port`.
- Log output — `redact_credentials()` in `mailer.rs` masks username and password from curl error output before surfacing in error messages.
- Stdout — no credential field is included in JSON or human-readable output.

**Test coverage:** `run_record_has_no_credentials` (runner.rs), `redact_credentials_masks_username_and_password` (mailer.rs).

**Verdict:** PASS. Credentials are handled correctly.

**Recommendation:** Document in the project's operations runbook that `TELEGRAM_BOT_TOKEN`, `SMTP_USERNAME`, and `SMTP_PASSWORD` should be stored in a secrets manager (e.g. `sdlc secrets`) and injected as environment variables, not committed to `.sdlc/config.yaml`.

---

## SA-2 — TLS Enforcement for SMTP

**Area:** SMTP transport (`mailer.rs`)

**Finding:** The `curl` invocation uses `--ssl-reqd` which makes TLS mandatory. If the SMTP server does not support TLS, the connection fails with an error rather than falling back to plaintext. The SMTP URL uses `smtps://` for port 465 (direct TLS) and `smtp://` with `--ssl-reqd` for all other ports (STARTTLS required).

**Verdict:** PASS. No plaintext SMTP is possible. This matches the spec requirement: "TLS required for SMTP (STARTTLS on port 587 or SMTPS on port 465); plaintext SMTP rejected."

---

## SA-3 — Telegram Bot API Authentication

**Area:** `client.rs` — getUpdates

**Finding:** The bot token is embedded in the API URL path (`/bot{token}/getUpdates`). This is the standard Telegram Bot API authentication mechanism. The token is transmitted over HTTPS (enforced by the reqwest client's default behavior). The URL is not logged anywhere.

**Concern:** The bot token appears in the URL. If request logging is enabled in reqwest (e.g., via tracing), the token could be emitted. Currently, no tracing is enabled for the Telegram client.

**Action:** Mark as accepted risk — this is inherent to the Telegram Bot API design. If URL-level logging is ever added to the codebase, a URL redaction filter must be applied to `api.telegram.org` paths.

**Verdict:** PASS for current implementation.

---

## SA-4 — Message Content in Digest

**Area:** User-generated content from Telegram chats

**Finding:** Message text from Telegram is included verbatim in the email digest. The HTML email formatter runs `html_escape()` on all user content before inserting into HTML. Plain text has no HTML injection risk.

**Test coverage:** `html_escape_handles_special_chars` — verifies `<`, `>`, `&`, `"` are all escaped.

**Verdict:** PASS. XSS via HTML injection is prevented.

---

## SA-5 — Command Injection via SMTP Credentials

**Area:** `mailer.rs` — curl subprocess

**Finding:** SMTP username and password are passed to curl via `--user <username>:<password>`. This is passed as a single `--user` argument value, not via shell interpolation — `Command::new("curl")` is used directly without `/bin/sh`, so no shell expansion occurs. Special characters in credentials (including `$`, `` ` ``, `;`) are safe.

**Verdict:** PASS. No shell injection risk.

---

## SA-6 — Config File ENV Interpolation

**Area:** `types.rs` — `resolve_placeholder`

**Finding:** `${ENV_VAR}` placeholders are resolved via `std::env::var`. The function only substitutes values when the entire string matches `${...}`. Partial placeholders or nested placeholders are not expanded — the raw string is returned as-is. This means a config value like `prefix-${VAR}-suffix` would NOT be interpolated; it would be used literally. This is intentional conservative behavior.

**No injection risk:** The resolved value is used directly in data structures, not eval'd or shell-expanded.

**Verdict:** PASS.

---

## SA-7 — Input Validation: Chat IDs

**Area:** `digest.rs` — chat ID filtering

**Finding:** Chat IDs are string-compared against the Telegram message's `chat.id` (converted to string). No validation is performed on the format of configured chat IDs. An invalid chat ID (e.g. "not-a-number") simply never matches any real chat — it produces an empty digest rather than an error.

**Security impact:** None — this is a configuration filtering issue, not a security risk.

**Verdict:** PASS. Minor UX note: consider warning when configured chat IDs never match any received updates.

---

## SA-8 — Run Record File Path

**Area:** `runner.rs` — run record write

**Finding:** The run record filename is `<timestamp>-tgd.json`, constructed from `started_at.format("%Y%m%d-%H%M%S-tgd")`. The timestamp is generated from `chrono::Utc::now()`, not from user input. No path traversal is possible.

**Verdict:** PASS.

---

## SA-9 — Dependency Surface

**Area:** `reqwest` (blocking mode, existing dep), `mockito` (dev-only)

**Finding:**
- `reqwest` is already a dependency of `sdlc-core` for other functionality. No new production dependency was added.
- `mockito` is added as a dev-dependency only — it does not appear in production builds.
- The `curl` binary used for SMTP is a system dependency, not a Cargo crate. Its TLS implementation (libcurl + the system TLS library) is maintained by the OS vendor.

**Verdict:** PASS. Minimal new dependency surface.

---

## SA-10 — Error Message Information Leakage

**Area:** All error paths

**Finding:** Error messages are surfaced to the user via `anyhow::Context` chains. Reviewed all error paths:
- Config errors include field names and helpful hints — no credential values.
- Telegram API errors include HTTP status and API description — no token.
- SMTP errors go through `redact_credentials` — no username or password.
- `SdlcError::TelegramTokenMissing` message says "check your bot token" — no token value.

**Verdict:** PASS.

---

## Summary

| Finding | Severity | Verdict | Action |
|---|---|---|---|
| SA-1: Credential storage | High | PASS | Document ops guidance |
| SA-2: TLS enforcement | High | PASS | None |
| SA-3: Token in URL | Medium | PASS (accepted) | Note re: future logging |
| SA-4: HTML injection from message content | Medium | PASS | None |
| SA-5: Command injection via SMTP creds | Medium | PASS | None |
| SA-6: Config ENV interpolation | Low | PASS | None |
| SA-7: Chat ID validation | Low | PASS | Minor UX note |
| SA-8: Run record path | Low | PASS | None |
| SA-9: Dependency surface | Low | PASS | None |
| SA-10: Error message leakage | Medium | PASS | None |

**Overall verdict: PASS.** No security findings require blocking action. The implementation handles credential isolation, TLS enforcement, HTML escaping, and command injection prevention correctly. One ops recommendation (SA-1) to document secrets management practice.
