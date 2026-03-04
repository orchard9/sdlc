# QA Plan: ThreadsPage Mobile Layout Fix

## Scope

Verify that the two-pane ThreadsPage layout correctly collapses to single-pane on mobile while preserving all desktop behavior.

## Test Cases

### TC-1: Mobile list view shows only thread list

**Viewport:** 375×812 (iPhone SE)
**URL:** `/threads`

Steps:
1. Navigate to `/threads` in a 375px-wide browser.

Expected:
- Thread list pane is visible.
- Thread detail pane (or empty state) is NOT visible.
- Only one column fills the screen.

### TC-2: Mobile detail view shows only thread detail

**Viewport:** 375×812
**URL:** `/threads/:slug` (select any thread from the list)

Steps:
1. From `/threads`, tap any thread row.
2. URL changes to `/threads/<slug>`.

Expected:
- Thread detail pane is visible (title, comments, compose area).
- Thread list pane is NOT visible.
- `AppShell` header shows back chevron (`<`).

### TC-3: Mobile back navigation returns to list

**Viewport:** 375×812

Steps:
1. Navigate to `/threads/:slug` (any thread).
2. Tap the `<` back chevron in the mobile header.

Expected:
- URL changes back to `/threads`.
- Thread list pane is visible.
- Thread detail pane is hidden.

### TC-4: Desktop shows both panes simultaneously

**Viewport:** 1280×800

Steps:
1. Navigate to `/threads`.

Expected:
- Thread list pane (280px wide) and detail/empty-state pane are both visible side by side.
- Layout is identical to pre-fix behavior.

### TC-5: Desktop thread selection preserves two-pane layout

**Viewport:** 1280×800

Steps:
1. On `/threads`, click any thread.

Expected:
- Both list pane and detail pane visible.
- Detail pane shows selected thread content.

### TC-6: Desktop functional regression — create thread

**Viewport:** 1280×800

Steps:
1. Click "New thread" button in list pane.
2. Fill in title and submit.

Expected:
- New thread appears in list.
- Detail pane navigates to new thread.
- No errors.

### TC-7: Desktop functional regression — add comment

**Viewport:** 1280×800

Steps:
1. Open any thread.
2. Type a comment and submit (⌘↵ or Send button).

Expected:
- Comment appears without page reload.
- No errors.

## Pass Criteria

All 7 test cases pass with no visual regressions or functional breakage.
