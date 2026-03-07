# QA Plan: Parked Ponder Resume Button

## Test 1: Resume button visible only when parked

1. Navigate to a ponder with status `parked`.
2. Verify the Resume button (Play icon + "Resume" text) is visible in the header.
3. Verify the Commit button is NOT visible.
4. Navigate to a ponder with status `exploring`.
5. Verify the Resume button is NOT visible.
6. Verify the Commit button IS visible.

## Test 2: Resume button changes status to exploring

1. Navigate to a parked ponder.
2. Click the Resume button.
3. Verify the API call `PUT /api/roadmap/:slug` fires with `{ "status": "exploring" }`.
4. Verify the status badge updates to "exploring".
5. Verify the Resume button disappears and the Commit button appears.
6. Verify chat input and "Start from title & brief" buttons become visible.

## Test 3: Empty state shows Resume for parked ponder

1. Navigate to a parked ponder with no dialogue sessions.
2. Verify the empty state shows a "Resume exploring" button.
3. Verify "Start from title & brief" is NOT shown.
4. Click "Resume exploring".
5. Verify status changes to exploring and the empty state updates to show "Start from title & brief".

## Test 4: Status modal still works for parked ponders

1. Navigate to a parked ponder.
2. Click the gear/sliders icon to open the status modal.
3. Verify all four statuses are selectable (except current).
4. Select "exploring" and apply.
5. Verify ponder resumes correctly — same behavior as Resume button.
