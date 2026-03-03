# Tasks: Fix Thread Body Ignored and Submit Button Stays Disabled

## T1 — Add `body` field to `FeedbackThread` and update `create_thread`

**File:** `crates/sdlc-core/src/feedback_thread.rs`

- Add `body: Option<String>` to `FeedbackThread` struct (with `#[serde(default)]` so existing YAML deserializes safely)
- Update `create_thread` to accept `body: Option<&str>` parameter
- Store trimmed, non-empty body in the struct before saving manifest
- Update existing unit tests that call `create_thread` to pass `None` as the new parameter
- Add a unit test asserting body is stored and round-trips through `create_thread` + `load_thread`

## T2 — Add `body` to `CreateBody` and wire through the route

**File:** `crates/sdlc-server/src/routes/threads.rs`

- Add `body: Option<String>` to `CreateBody` struct
- In `create_thread` route handler, pass `body` to `sdlc_core::feedback_thread::create_thread`
- Update `thread_to_json` helper to serialize `body` (null when `None`)
- In `get_thread` route, remove the hardcoded `value["body"] = serde_json::Value::Null` line (thread_to_json now handles it)
- Update existing route tests that construct `CreateBody` directly to include the new field (pass `body: None`)
- Add a route integration test: POST with body → GET returns same body

## T3 — Fix submit button stays disabled in `NewThreadModal`

**File:** `frontend/src/components/threads/NewThreadModal.tsx`

- In `handleSubmit` try block, add `setSubmitting(false)` after the successful `await onSubmit(...)` call
- In the reopen `useEffect` (runs when `open` is truthy), add `setSubmitting(false)` to the reset block
