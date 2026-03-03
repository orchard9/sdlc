# Review: Wire up Thread Detail Actions

## Summary

All three thread detail actions are now wired up. The implementation is clean, minimal, and correctly scoped to the bug fix.

## Changes Reviewed

### `crates/sdlc-core/src/feedback_thread.rs`
- Added `status: String` (serde default `"open"`) and `promoted_to: Option<String>` to `FeedbackThread`
- Added `patch_thread(root, id, status, promoted_to)` public function
- `create_thread` initializes both new fields correctly
- Existing threads deserialize correctly via `serde(default)` — backward compatible

### `crates/sdlc-server/src/routes/threads.rs`
- `thread_to_json` now emits `status` and `promoted_to` from the struct instead of hardcoded values
- New `patch_thread` handler: validates, calls core, returns updated JSON
- New `promote_thread` handler: derives ponder slug (collision-safe), creates ponder entry, patches thread status to `"promoted"`, returns both slugs
- Slug derivation is clean: lowercase, collapse dashes, trim, truncate to 50 chars, collision suffix

### `crates/sdlc-server/src/lib.rs`
- `PATCH /api/threads/{id}` and `POST /api/threads/{id}/promote` routes registered correctly alongside existing thread routes

### `frontend/src/api/client.ts`
- `deleteThread`, `patchThread`, `promoteThreadToPonder` added with correct types

### `frontend/src/components/threads/ThreadDetailPane.tsx`
- New props: `onDelete`, `onStatusChange`, `onPromoted`
- Delete: trash icon button → inline confirm state → delete + navigate
- Synthesize: enabled for `open` threads only, patches status, calls `onStatusChange`
- Promote: enabled when not `promoted`, calls promote API, calls `onPromoted`
- All three show spinner during async, inline error on failure
- `StatusBadge` already handled all three statuses — no change needed

### `frontend/src/pages/ThreadsPage.tsx`
- `handleDelete`: removes thread from list, navigates to `/threads`
- `handleStatusChange`: updates both detail and list state
- `handlePromoted`: navigates to `/ponder/<slug>`

## Findings

**All findings accepted as-is (no blockers):**

1. `promote_thread` creates the ponder entry but does not capture the thread body as a ponder artifact. The thread body is currently stored in the ponder title only. Acceptable for this fix — can be enhanced in a follow-up if needed.

2. `handleStatusChange` in `ThreadsPage` trusts the server response to update list status. If the server PATCH fails, the UI shows an error and doesn't update. Correct behavior.

3. The slug derivation in `promote_thread` uses a character-by-character loop to collapse dashes — slightly verbose but correct and matches the pattern in `feedback_thread.rs`.

## Verdict

**APPROVED** — Implementation is complete, tests pass, clippy clean, no TS errors. Spec acceptance criteria all met.
