# QA Plan: Git Status Chip

## Test Scenarios

### QA-1: Green state renders correctly
- Navigate to any page in the UI with a clean repo (no uncommitted changes).
- Verify the sidebar shows a green dot with "{branch} - clean" text.
- Verify the commit button is NOT visible.

### QA-2: Yellow state renders correctly
- Make uncommitted changes in the repo (modify a file without staging).
- Verify the sidebar shows a yellow/amber dot with "{branch} - N modified" text.

### QA-3: Red state renders correctly
- Simulate a conflict state (API returns `severity: red`, `has_conflicts: true`).
- Verify the sidebar shows a red dot with "{branch} - N conflicts" text.

### QA-4: Commit button visibility
- Stage files in the repo so `staged_count > 0`.
- Verify the commit button appears next to the summary text.
- Unstage files so `staged_count === 0`.
- Verify the commit button disappears.

### QA-5: Collapsed sidebar
- Collapse the sidebar.
- Verify the chip shows only the severity dot (no text, no commit button).
- Verify hovering shows a tooltip with the full summary text.

### QA-6: Polling behavior
- Verify the chip re-fetches status on the configured interval (default 10s).
- Switch to another browser tab and back — verify status updates on focus.

### QA-7: Error/offline state
- Simulate API failure (stop the server or block the endpoint).
- Verify the chip shows a grey dot with "Git status unavailable" tooltip.
- Verify no error toasts or console errors.

### QA-8: Commit action
- With staged files, click the commit button.
- Verify the commit request is sent.
- Verify the status re-fetches after the commit.

## Pass Criteria

All 8 scenarios pass. No layout shift in the sidebar during state transitions. Component renders correctly in both light and dark themes.
