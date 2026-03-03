# Spec: Fix Thread Body Ignored and Submit Button Stays Disabled

## Problem

Two independent bugs in the feedback threads feature:

### Bug 1 — Thread body is silently discarded on creation

When a user creates a thread with a core element (body) via POST `/api/threads`, the body is silently dropped. The server's `CreateBody` struct only deserializes `context` and `title`; the `body` field is unknown to serde and ignored. Additionally, `FeedbackThread` and `create_thread(...)` in `feedback_thread.rs` have no concept of a body/core element, so the data is never stored. The `get_thread` endpoint returns `"body": null` always.

### Bug 2 — Submit button stays disabled after successful thread creation

After a thread is created successfully, the submit button in `NewThreadModal` remains permanently disabled. `setSubmitting(false)` is only called in the `catch` block. On success, the parent (`ThreadsPage`) calls `setCreateOpen(false)`, which fires the modal's `useEffect`—but that effect only resets `title`, `body`, and `error`, not `submitting`. When the modal is reopened, `submitting` is still `true`, so the button is disabled even with a valid title.

## Requirements

### R1 — Body stored and returned
- `FeedbackThread` struct gains an optional `body: Option<String>` field (the "core element").
- `create_thread(root, context, title, body)` accepts and persists the body.
- `CreateBody` in `threads.rs` gains `body: Option<String>`.
- The `create_thread` route passes `body` to the core function.
- `thread_to_json` serializes `body` (null when absent).
- `get_thread` returns the stored `body` value instead of hardcoded `null`.

### R2 — Submit button re-enables after success
- `handleSubmit` in `NewThreadModal` calls `setSubmitting(false)` in the success path (after `await onSubmit(...)` returns).
- As a safety net, the reopen `useEffect` (runs when `open` becomes truthy) also resets `submitting` to `false`.

## Out of Scope

- Editing an existing thread's body (mutation endpoint).
- Frontend display of the body in the thread list pane.
- Any body field in the comment/post model (`ThreadPost.content` is separate).

## Acceptance Criteria

1. `POST /api/threads` with `{"title":"T","body":"B"}` stores `B` and returns `"body":"B"`.
2. `GET /api/threads/:id` returns the stored body value (not null) for a thread created with a body.
3. `POST /api/threads` with no body field continues to work (body is optional).
4. Existing tests continue to pass.
5. After a successful thread creation, the modal closes, and when reopened the submit button is enabled (not stuck in "Creating…").
6. On a creation error the button re-enables immediately (existing behavior preserved).
