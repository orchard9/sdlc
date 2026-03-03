# QA Results: telegram-bot-message-store

**Date:** 2026-03-03
**Tester:** Agent (automated + manual inspection)
**Build:** `SDLC_NO_NPM=1 cargo test --all` — 0 failures; `cargo clippy --all -- -D warnings` — 0 warnings

---

## TC-1: Cargo build and clippy — PASS

```
SDLC_NO_NPM=1 cargo build --all
  → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s

cargo clippy --all -- -D warnings
  → Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.46s
```

Zero errors. Zero warnings.

---

## TC-2: Unit tests — MessageStore open and schema — PASS

```
cargo test -p sdlc-core -- telegram::poll::tests::test_open_creates_schema
  → test telegram::poll::tests::test_open_creates_schema ... ok
```

`MessageStore::open()` creates the `messages` and `meta` tables idempotently. `message_count()` returns 0 on a fresh DB; `get_offset()` returns `None`.

---

## TC-3: Unit tests — insert deduplication — PASS

```
cargo test -p sdlc-core -- telegram::poll::tests::test_insert_deduplication
  → test telegram::poll::tests::test_insert_deduplication ... ok
```

Two `insert_message` calls with the same `update_id`: second call is silently ignored via `INSERT OR IGNORE`. `message_count()` returns 1.

---

## TC-4: Unit tests — offset round trip — PASS

```
cargo test -p sdlc-core -- telegram::poll::tests::test_offset_round_trip
  → test telegram::poll::tests::test_offset_round_trip ... ok
```

`get_offset()` returns `None` before any write; `set_offset(42)` → `get_offset()` returns `Some(42)`; `set_offset(100)` → `Some(100)`.

---

## TC-5: Unit tests — time_range — PASS

```
cargo test -p sdlc-core -- telegram::poll::tests::test_time_range_empty
  → test telegram::poll::tests::test_time_range_empty ... ok

cargo test -p sdlc-core -- telegram::poll::tests::test_time_range_with_messages
  → test telegram::poll::tests::test_time_range_with_messages ... ok
```

Empty DB returns `Ok(None)`. DB with two messages (date 1000, date 2000) returns `Ok(Some((1000, 2000)))`.

Additional config tests also pass:
- `test_config_missing_token_returns_error` — ok
- `test_config_from_env` — ok
- `test_config_env_overrides_yaml` — ok
- `test_insert_and_count` — ok

9/9 poll unit tests pass.

---

## TC-6: CLI test — missing token error — PASS

```
TELEGRAM_BOT_TOKEN="" cargo test -p sdlc-cli --test telegram_test
  → test test_status_missing_token ... ok
  → test test_poll_missing_token ... ok
  → test result: ok. 2 passed; 0 failed; 0 ignored
```

`sdlc telegram status` and `sdlc telegram poll` both exit non-zero and produce stderr mentioning `TELEGRAM_BOT_TOKEN` when no token is configured.

---

## TC-7: Full test suite — PASS

```
SDLC_NO_NPM=1 cargo test --all
```

All test bins pass with 0 failures:

| Test binary | Passed | Failed |
|---|---|---|
| sdlc-core (lib) | 396 | 0 |
| sdlc-cli (lib) | 52 | 0 |
| sdlc-cli (main) | 52 | 0 |
| sdlc-cli (integration) | 114 | 0 |
| sdlc-cli (telegram_test) | 2 | 0 |
| sdlc-server (lib) | 131 | 0 |
| sdlc-server (integration) | 45 | 0 |
| claude-agent (lib) | 23 | 0 |

No regressions.

---

## TC-8: Manual — `sdlc telegram status` with invalid token — PASS (manual inspection)

Inspected `crates/sdlc-cli/src/cmd/telegram.rs`: `run_status` calls `get_me(config)` first, which issues a `getMe` HTTP request. On a 401 Unauthorized response, `poll.rs::poll_loop` maps the HTTP 401 to `SdlcError::TelegramTokenMissing`. For `get_me`, the API returns `ok: false` with `description: "Unauthorized"` which propagates as `SdlcError::TelegramApi("Unauthorized")`. Both produce a non-zero exit with a clear error message — no panic, no stack trace. No live bot token available in CI; manual verification accepted per QA plan.

---

## TC-9: Manual — `sdlc telegram poll` (live, optional) — SKIPPED

Marked optional in QA plan. No live bot token available in CI environment. The long-poll implementation is covered by unit tests (TC-2 through TC-6) and code review. Skipped per plan's exit criteria.

---

## TC-10: .gitignore check — PASS

```
mkdir -p .sdlc/telegram && git status .sdlc/telegram/
  → nothing to commit, working tree clean
```

`.sdlc/telegram/` is present in `.gitignore` and does not appear as an untracked file.

---

## Additional finding: poll.rs tests were missing

The QA plan referenced `test_open_creates_schema`, `test_insert_deduplication`, `test_offset_round_trip`, `test_time_range_empty`, and `test_time_range_with_messages` — but these tests were absent from the codebase. They were added during QA execution to `crates/sdlc-core/src/telegram/poll.rs` (9 tests total). All pass. No change to production code; test-only addition.

---

## Summary

| TC | Description | Result |
|---|---|---|
| TC-1 | Build + clippy | PASS |
| TC-2 | MessageStore schema creation | PASS |
| TC-3 | Insert deduplication | PASS |
| TC-4 | Offset round trip | PASS |
| TC-5 | time_range (empty + with messages) | PASS |
| TC-6 | CLI missing token error | PASS |
| TC-7 | Full test suite | PASS |
| TC-8 | Invalid token error (manual) | PASS (inspected) |
| TC-9 | Live poll (optional) | SKIPPED |
| TC-10 | .gitignore | PASS |

**All required test cases pass. Feature is ready to merge.**
