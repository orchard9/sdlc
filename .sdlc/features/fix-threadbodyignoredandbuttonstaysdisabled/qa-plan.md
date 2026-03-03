# QA Plan: Fix Thread Body Ignored and Submit Button Stays Disabled

## TC-1 — Body stored on creation (server unit test)

**Type:** Rust unit test (`feedback_thread.rs`)
**Method:** Call `create_thread(root, "general", "T", Some("My core element"))`, then `load_thread` and assert `body == Some("My core element")`.
**Pass:** Body round-trips through save/load.

## TC-2 — Body returned in create response (route test)

**Type:** Rust integration test (`threads.rs`)
**Method:** POST `CreateBody { context: None, title: Some("T"), body: Some("B") }`, assert response `body == "B"`.
**Pass:** Route returns stored body in JSON response.

## TC-3 — GET returns stored body (route test)

**Type:** Rust integration test (`threads.rs`)
**Method:** Create thread with body "B", then GET by id, assert `body == "B"`.
**Pass:** GET returns actual body, not hardcoded null.

## TC-4 — No body field is backward compatible (unit test)

**Type:** Rust unit test
**Method:** Call `create_thread(root, "general", "T", None)`, assert response `body == null`.
**Pass:** Threads without body continue to work.

## TC-5 — Existing tests pass (regression)

**Type:** `SDLC_NO_NPM=1 cargo test --all`
**Pass:** All existing tests green with no changes required to test assertions.

## TC-6 — Submit button re-enables after success (code inspection)

**Type:** Code review
**Method:** Verify `setSubmitting(false)` appears in the try block of `handleSubmit` after `await onSubmit(...)`.
**Pass:** Code change is present and correct.

## TC-7 — Submitting resets on modal reopen (code inspection)

**Type:** Code review
**Method:** Verify `setSubmitting(false)` is in the `if (open)` branch of the reopen `useEffect`.
**Pass:** Code change is present and correct.

## TC-8 — Clippy clean

**Type:** `cargo clippy --all -- -D warnings`
**Pass:** Zero warnings.
