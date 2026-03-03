# Tasks: Daily Cron Digest — Telegram → SMTP Email

## Task Breakdown

### T1 — Add dependencies to Cargo.toml files

Add `reqwest` (with `json` and `rustls-tls` features) and `lettre` (with `tokio1-native-tls` and `builder` features) to `crates/sdlc-core/Cargo.toml`. Verify `cargo build --all` succeeds with `SDLC_NO_NPM=1`.

**Acceptance:** `cargo build -p sdlc-core` succeeds with new deps resolved.

---

### T2 — Define core types in `crates/sdlc-core/src/telegram/types.rs`

Create the module directory `crates/sdlc-core/src/telegram/` with:
- `mod.rs` — re-exports public API (`DigestConfig`, `DigestSummary`, `DigestRunner`)
- `types.rs` — `DigestConfig`, `SmtpConfig`, `TelegramMessage`, `TelegramUser`, `TelegramChat`, `ChatDigest`, `DigestMessage`, `DigestSummary`

All types derive `Debug`, `Deserialize` (serde) where appropriate. `DigestConfig` and `SmtpConfig` must support `${ENV_VAR}` interpolation via a `resolve_env` method.

**Acceptance:** `cargo test -p sdlc-core` builds with new module; unit test for `resolve_env` with env vars set passes.

---

### T3 — Implement `TelegramClient` in `crates/sdlc-core/src/telegram/client.rs`

Implement an async `TelegramClient` struct wrapping `reqwest::Client`:

```rust
pub struct TelegramClient { base_url: String, token: String, client: reqwest::Client }

impl TelegramClient {
    pub fn new(token: &str) -> Self
    pub async fn get_updates(&self, limit: u32) -> Result<Vec<TelegramUpdate>, SdlcError>
}
```

- `get_updates` calls `GET https://api.telegram.org/bot{token}/getUpdates?limit={limit}&allowed_updates=["message"]`
- Parses response into `Vec<TelegramUpdate>` (wraps message + update_id)
- Maps HTTP errors, non-200 status, and API `ok: false` to `SdlcError`
- No retry logic; 429 returns `SdlcError::TelegramRateLimit`

**Acceptance:** Unit tests with `wiremock` mock server for success, 401, 429, and malformed JSON responses.

---

### T4 — Implement `DigestBuilder` in `crates/sdlc-core/src/telegram/digest.rs`

Implement `DigestBuilder`:

```rust
pub struct DigestBuilder { window_hours: u32, chat_ids: Vec<String>, max_messages: u32 }

impl DigestBuilder {
    pub fn new(config: &DigestConfig) -> Self
    pub fn build(&self, updates: Vec<TelegramUpdate>, now: DateTime<Utc>) -> DigestSummary
}
```

- Filters updates to the configured time window (`now - window_hours`)
- Filters to configured `chat_ids` (or all chats if empty)
- Groups messages by chat, sorts by timestamp ascending
- Truncates to `max_messages_per_chat` (most recent messages kept if over limit)
- Returns `DigestSummary` with all `ChatDigest` entries

**Acceptance:** Unit tests for: empty updates, time window filtering, multi-chat grouping, max_messages truncation, chat ID filtering.

---

### T5 — Implement email formatters in `crates/sdlc-core/src/telegram/digest.rs`

Add to `DigestSummary`:

```rust
impl DigestSummary {
    pub fn format_subject(&self, prefix: &str) -> String
    pub fn format_plain_text(&self) -> String
    pub fn format_html(&self) -> String
}
```

- `format_subject`: `"{prefix} {YYYY-MM-DD} — {N} messages across {M} chats"`
- `format_plain_text`: matches spec template (header, per-chat sections, footer)
- `format_html`: inline-CSS multipart HTML with per-chat tables; no external resources

**Acceptance:** Snapshot/golden-file tests for both plain text and HTML output using a deterministic `DigestSummary` fixture.

---

### T6 — Implement `SmtpMailer` in `crates/sdlc-core/src/telegram/mailer.rs`

```rust
pub struct SmtpMailer { config: SmtpConfig }

impl SmtpMailer {
    pub fn new(config: SmtpConfig) -> Self
    pub async fn send(&self, subject: &str, plain: &str, html: &str) -> Result<(), SdlcError>
}
```

- Uses `lettre::AsyncSmtpTransport::<Tokio1Executor>::starttls_relay` for port 587
- Uses `lettre::AsyncSmtpTransport::<Tokio1Executor>::relay` (TLS) for port 465
- Other ports use STARTTLS
- Maps `lettre::Error` variants to `SdlcError` (auth → `SmtpAuthError`, delivery → `SmtpDeliveryError`)
- `send` builds multipart MIME message and sends to all `config.to` recipients

**Acceptance:** Unit test uses `lettre::SmtpTransport::dry_run()` to verify message construction without a real SMTP server. Test for missing TLS rejection.

---

### T7 — Implement `DigestRunner` in `crates/sdlc-core/src/telegram/runner.rs`

```rust
pub struct DigestRunner { config: DigestConfig }

impl DigestRunner {
    pub fn new(config: DigestConfig) -> Self
    pub async fn run(&self, dry_run: bool) -> Result<DigestRunResult, SdlcError>
}

pub struct DigestRunResult {
    pub summary: DigestSummary,
    pub dry_run: bool,
    pub sent_to: Vec<String>,   // empty if dry_run
}
```

Orchestrates: `TelegramClient::get_updates` → `DigestBuilder::build` → (if not dry_run) `SmtpMailer::send` → return `DigestRunResult`.

**Acceptance:** Integration test with `wiremock` mock for Telegram API + `lettre` dry_run transport.

---

### T8 — Write run record to `.sdlc/.runs/`

In `DigestRunner::run`, after successful completion (or failure), write a JSON run record to `.sdlc/.runs/<timestamp>-telegram-digest.json` using the existing file I/O patterns from `crates/sdlc-core/src/io.rs`.

Run record schema matches spec design (id, kind, status, started_at, completed_at, summary, metadata). Credentials must not appear in the record.

**Acceptance:** Integration test verifies run record is written with correct fields; no credential leakage test.

---

### T9 — Extend `SdlcConfig` with optional `telegram` field

In `crates/sdlc-core/src/config.rs` (or wherever `SdlcConfig` is defined), add:

```rust
pub struct SdlcConfig {
    // ... existing fields ...
    #[serde(default)]
    pub telegram: Option<DigestConfig>,
}
```

Implement `${ENV_VAR}` interpolation for `bot_token`, `smtp.username`, `smtp.password` at config load time.

**Acceptance:** Unit test for config parsing with and without `telegram` block; env var override test.

---

### T10 — Implement `sdlc telegram digest` CLI subcommand

In `crates/sdlc-cli/src/cmd/telegram.rs`:

```
sdlc telegram digest [--dry-run] [--window <hours>] [--chat <id>]... [--json] [-v]
```

- Loads `SdlcConfig` from `.sdlc/config.yaml` (or `--config` override)
- Applies CLI overrides (`--window`, `--chat`) on top of loaded config
- Validates required fields; exits 1 with error message if missing
- Calls `DigestRunner::run(dry_run)` inside `tokio::main` or existing async runtime
- On success: prints summary to stdout (human or `--json`)
- On error: prints error to stderr, exits with appropriate code (1/2/3)

Register `telegram` as a subcommand in `crates/sdlc-cli/src/main.rs` (or `cmd/mod.rs`).

**Acceptance:** CLI integration test with mock config file; dry-run prints digest to stdout and exits 0.

---

### T11 — Add unit and integration tests

Ensure test coverage per design:
- `client.rs` — mock server tests (T3 above)
- `digest.rs` — pure unit tests (T4/T5 above)
- `mailer.rs` — dry-run lettre tests (T6 above)
- `runner.rs` — end-to-end with mocks (T7 above)
- Config loading — T9 tests
- CLI — integration tests (T10 above)

Run `SDLC_NO_NPM=1 cargo test --all` and confirm all pass. Run `cargo clippy --all -- -D warnings` and fix all warnings.

**Acceptance:** All tests pass. Zero clippy warnings.

---

### T12 — Update documentation

- Add `telegram` section to `.sdlc/config.yaml` with commented-out example block
- Add `sdlc telegram digest` to `AGENTS.md` CLI reference table (under the `§6 Using sdlc` command table)
- Add cron example to `docs/` or inline in `AGENTS.md`

**Acceptance:** `AGENTS.md` and config example file updated; no broken references.

## Dependency Order

```
T1 (deps) → T2 (types) → T3 (client) ─┐
                        → T4 (builder) ─┤→ T7 (runner) → T8 (run record)
                        → T5 (format)  ─┤
                        → T6 (mailer) ──┘
T9 (config) → T10 (CLI) → T11 (tests) → T12 (docs)
```

T1, T2, T9 can start in parallel. T3–T6 depend on T2. T7 depends on T3–T6. T8 depends on T7. T10 depends on T7 and T9. T11 requires T3–T10. T12 last.
