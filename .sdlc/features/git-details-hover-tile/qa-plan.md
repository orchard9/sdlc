# QA Plan: Git Details Hover Tile

## Test Strategy

Manual verification via the running UI and automated checks via build compilation.

## Test Cases

### TC1: Popover appears on hover (desktop)
1. Open the UI in a browser with the sidebar expanded
2. Hover over the GitStatusChip at the bottom of the sidebar
3. Verify a popover appears showing branch name, status counts, severity, and guidance
4. Move mouse away from the chip
5. Verify the popover dismisses after a short delay

### TC2: Popover appears on click
1. Click the GitStatusChip
2. Verify the popover toggles open
3. Click again to verify it toggles closed

### TC3: Click outside dismisses popover
1. Click the GitStatusChip to open the popover
2. Click elsewhere in the page content area
3. Verify the popover closes

### TC4: Zero counts are omitted
1. With a clean working tree (no modified/staged/untracked/conflict files), hover the chip
2. Verify only the branch line, severity ("Green"), and guidance ("All clear") appear
3. No status count rows should be visible

### TC5: All non-zero counts appear
1. With modified, staged, and untracked files present, hover the chip
2. Verify each non-zero category appears as a labeled row with count

### TC6: Guidance matches severity
1. Green state: verify guidance says "All clear" or "Push to share"
2. Yellow state: verify guidance mentions uncommitted changes
3. Red state: verify guidance mentions conflicts or pulling

### TC7: Collapsed sidebar popover positioning
1. Collapse the sidebar using the toggle
2. Hover or click the severity dot
3. Verify the popover appears (positioned to the right or above) and is fully visible

### TC8: TypeScript interface updated
1. Verify `GitStatus` interface in `useGitStatus.ts` includes `untracked_count`, `conflict_count`, and `summary`
2. Verify the app compiles without TypeScript errors

### TC9: No new API calls
1. Open browser network tab
2. Hover the chip to show popover
3. Verify no additional `/api/git/status` calls are triggered by the popover -- it reuses existing data

## Pass Criteria

All test cases pass. The popover renders correctly in both expanded and collapsed sidebar states and dismisses properly.
