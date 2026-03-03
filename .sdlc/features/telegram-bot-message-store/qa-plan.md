# QA Plan: telegram-bot-message-store

## Scope

Verify that the Telegram bot polling and SQLite message storage feature works correctly, is resilient to errors, and integrates cleanly with the existing sdlc build and test infrastructure.

---

## TC-1: Cargo build and clippy

**Type:** Automated
**Command:**
```bash
SDLC_NO_NPM=1 cargo build --all
cargo clippy --all -- -D warnings
```
**Pass:** Build succeeds with zero errors; clippy produces no warnings in new or modified files.
**Fail:** Any compilation error or new clippy warning.

---

## TC-2: Unit tests — MessageStore open and schema

**Type:** Automated (unit test in `sdlc-core`)
**Command:**
```bash
SDLC_NO_NPM=1 cargo test -p sdlc-core telegram
```
**Test:** `test_open_creates_schema`
**Assertion:** After `MessageStore::open()`, the `messages` and `poll_state` tables exist in the DB; `message_count()` returns 0.
**Pass:** Test passes.

---

## TC-3: Unit tests — insert deduplication

**Type:** Automated (unit test in `sdlc-core`)
**Test:** `test_insert_deduplication`
**Steps:**
1. Create two `TelegramMessage` values with the same `message_id`
2. Call `insert_message()` for both
3. Assert first call returns `Ok(true)`, second returns `Ok(false)`
4. Assert `message_count()` returns 1
**Pass:** All assertions pass.

---

## TC-4: Unit tests — offset round trip

**Type:** Automated (unit test in `sdlc-core`)
**Test:** `test_offset_round_trip`
**Steps:**
1. Open MessageStore, assert `get_offset()` returns `Ok(None)`
2. Call `set_offset(42)`
3. Assert `get_offset()` returns `Ok(Some(42))`
**Pass:** Assertions pass.

---

## TC-5: Unit tests — time_range

**Type:** Automated (unit test in `sdlc-core`)
**Tests:** `test_time_range_empty` and `test_time_range_with_messages`
**Assertions:**
- Empty DB: `time_range()` returns `Ok(None)`
- Two messages with dates 1000 and 2000: `time_range()` returns `Ok(Some((1000, 2000)))`
**Pass:** Both assertions pass.

---

## TC-6: CLI test — missing token error

**Type:** Automated (integration test in `sdlc-cli`)
**Test:** `test_status_missing_token`
**Steps:**
```bash
# Ensure TELEGRAM_BOT_TOKEN is unset, no .sdlc/config.yaml has token
TELEGRAM_BOT_TOKEN="" cargo test -p sdlc-cli telegram
```
**Assertion:** `sdlc telegram status` exits with non-zero code and stderr contains a human-readable error mentioning `TELEGRAM_BOT_TOKEN`.
**Pass:** Test passes.

---

## TC-7: Full test suite passes

**Type:** Automated
**Command:**
```bash
SDLC_NO_NPM=1 cargo test --all
```
**Pass:** All existing tests continue to pass; new tests pass; no regressions.
**Fail:** Any previously-passing test fails.

---

## TC-8: Manual — `sdlc telegram status` with invalid token

**Type:** Manual (developer)
**Precondition:** Set `TELEGRAM_BOT_TOKEN` to a syntactically valid but invalid token string (e.g., `123456:invalid`)
**Steps:**
1. Run `sdlc telegram status`
**Expected:** Error message: "Invalid bot token" or similar; non-zero exit.
**Pass:** Clear error, no panic, no stack trace.

---

## TC-9: Manual — `sdlc telegram poll` (live, optional)

**Type:** Manual (developer, requires real bot token)
**Precondition:** Valid `TELEGRAM_BOT_TOKEN` set; bot added to a test group
**Steps:**
1. Run `sdlc telegram poll`
2. Send a message in the Telegram group
3. Wait for the poll cycle to complete (up to 30 seconds)
4. Press Ctrl-C
5. Run `sdlc telegram status`
**Expected:**
- Step 1: Prints bot username, "Polling for messages..."
- Step 3: Log line shows message stored
- Step 4: Prints "Stopped. Total messages stored: N"
- Step 5: Shows correct message count and timestamps
**Pass:** All steps produce expected output; DB contains the sent message.

---

## TC-10: .gitignore check

**Type:** Manual
**Steps:**
1. Run `sdlc telegram poll` briefly (or manually create `.sdlc/telegram/`)
2. Run `git status`
**Expected:** `.sdlc/telegram/` does not appear as an untracked file.
**Pass:** Directory is ignored by git.

---

## Exit Criteria

All automated tests (TC-1 through TC-7) must pass. TC-8 and TC-10 must pass manually. TC-9 is optional (requires a live bot token) but should pass if run.
