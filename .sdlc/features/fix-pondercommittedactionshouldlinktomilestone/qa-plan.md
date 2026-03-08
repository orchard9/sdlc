# QA Plan

## Test 1: Committed ponder shows View Milestone button
1. Navigate to a ponder page where `status === 'committed'` and `committed_to` has at least one milestone slug.
2. Verify the action button reads "View Milestone" (not "Prepare").
3. Click the button. Verify navigation to `/milestone/<slug>`.

## Test 2: Non-committed ponder statuses unaffected
1. Navigate to a ponder with status `exploring` — verify the "Commit" button still appears.
2. Navigate to a ponder with status `parked` — verify the "Resume" button still appears.

## Test 3: No prepare agent run triggered
1. On a committed ponder, verify no network request to `/api/milestone/<slug>/prepare` is made when clicking the action button.
