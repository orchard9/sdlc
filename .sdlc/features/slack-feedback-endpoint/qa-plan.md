# QA Plan: slack-feedback-endpoint

## Test Strategy

All tests are Rust unit tests in `crates/sdlc-server/src/routes/feedback.rs`, using the existing `AppState::new()` + `tempfile::TempDir` pattern.

## Test Cases

### TC-1: Valid payload creates thread — 201
- POST a valid `SlackFeedbackPayload` with all fields
- Assert: 201 status, response contains `id`, `context` starts with `"slack:"`, `title` contains trigger text, `comment_count` >= 1

### TC-2: Missing `text` — 400
- POST payload with `text: ""`
- Assert: 400 status, error message mentions "text"

### TC-3: Missing `user_name` — 400
- POST payload with `user_name: ""`
- Assert: 400 status, error message mentions "user_name"

### TC-4: Wrong `source` — 400
- POST payload with `source: "telegram"`
- Assert: 400 status, error message mentions "source"

### TC-5: Duplicate `message_ts` — 409
- POST valid payload with `message_ts: "123.456"`
- POST same payload again
- Assert: first returns 201, second returns 409 with `existing_thread_id`

### TC-6: No `context_messages` — 201
- POST payload without `context_messages` field
- Assert: 201, thread body contains only the trigger message, no "Conversation Context" heading

### TC-7: With `context_messages` — body renders markdown
- POST payload with 3 context messages
- Assert: 201, thread body contains "## Conversation Context", all context message authors appear, trigger message appears after separator

### TC-8: No `message_ts` skips dedup
- POST two payloads with `message_ts: null`
- Assert: both return 201, different thread IDs

## Build Verification

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```
