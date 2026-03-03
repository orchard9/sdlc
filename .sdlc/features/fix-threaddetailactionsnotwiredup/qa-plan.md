# QA Plan: Wire up Thread Detail Actions

## TC-1 — Delete thread

**Setup:** Navigate to a thread detail page (e.g. `/threads/20260303-general-2`).

**Steps:**
1. Confirm Delete button (trash icon) is visible in thread header
2. Click Delete button — confirm it shows inline confirm state ("Delete? / Cancel")
3. Click Cancel — confirm button reverts to normal, thread is unchanged
4. Click Delete again, then confirm — confirm spinner appears during request
5. After delete: confirm navigation to `/threads` and thread is gone from list

**Pass criteria:** Thread removed, user lands on thread list, no JS errors.

## TC-2 — Delete non-existent thread (error handling)

**Steps:**
1. Simulate delete API failure (e.g. thread already deleted in another tab)
2. Confirm inline error message appears without crash
3. Confirm Delete button is re-enabled after error

**Pass criteria:** Error message shown, no crash, button recoverable.

## TC-3 — Synthesize thread

**Setup:** Navigate to an open thread.

**Steps:**
1. Confirm "Synthesize" button is enabled (not disabled/cursor-not-allowed)
2. Click Synthesize — confirm spinner appears
3. After response: confirm status badge changes to "synthesized"
4. Confirm Synthesize button is now disabled (already synthesized)
5. Reload page — confirm status persists as "synthesized"

**Pass criteria:** Status badge updated, button disabled after, survives reload.

## TC-4 — Synthesize a thread that is already synthesized

**Steps:**
1. Navigate to an already-synthesized thread
2. Confirm Synthesize button is disabled

**Pass criteria:** Button disabled, no double-synthesis possible.

## TC-5 — Promote to Ponder

**Setup:** Navigate to an open thread with a title and body.

**Steps:**
1. Confirm "Promote to Ponder" button is enabled
2. Click Promote to Ponder — confirm spinner appears
3. After response: confirm navigation to `/ponder/<slug>`
4. On ponder page: confirm entry exists with thread title
5. Navigate back to thread URL — confirm status badge shows "→ ponder"

**Pass criteria:** Ponder entry created, navigation to ponder page, thread status updated.

## TC-6 — Promote already-promoted thread

**Steps:**
1. Navigate to a promoted thread
2. Confirm "Promote to Ponder" button is disabled

**Pass criteria:** Button disabled.

## TC-7 — Backend PATCH and promote routes

**Steps (cargo test):**
1. Run `SDLC_NO_NPM=1 cargo test --all` — all tests pass
2. Run `cargo clippy --all -- -D warnings` — no warnings
3. Verify new routes registered in `mod.rs`: `PATCH /api/threads/:id`, `POST /api/threads/:id/promote`

**Pass criteria:** Test suite green, clippy clean.

## TC-8 — Frontend type-check

**Steps:**
1. Run `cd frontend && npm run build` (or `npx tsc --noEmit`)
2. No TypeScript errors

**Pass criteria:** Zero TS errors.

## TC-9 — Existing thread functionality unaffected

**Steps:**
1. Create a new thread
2. Add a comment
3. Verify comment appears in thread detail
4. Verify thread list shows updated comment count

**Pass criteria:** Existing create/comment flow unchanged.
