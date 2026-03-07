# QA Plan: ReleasedPanel Component

## Test Strategy

This is a frontend-only feature. Verification focuses on correct rendering, conditional display logic, and interaction with existing hooks.

## Test Cases

### TC-1: ReleasedPanel renders for released milestones
- Navigate to a milestone detail page where `milestone.status === 'released'`
- Verify the ReleasedPanel is visible with victory banner, stats, and actions
- Verify MilestonePreparePanel is NOT rendered

### TC-2: MilestonePreparePanel renders for non-released milestones
- Navigate to a milestone detail page where `milestone.status === 'active'`
- Verify MilestonePreparePanel is rendered
- Verify ReleasedPanel is NOT rendered

### TC-3: Stats display correct data
- On a released milestone with UAT history, verify:
  - Feature count matches milestone's feature list length
  - UAT run count matches actual runs
  - Latest verdict badge shows correct verdict
  - Date is formatted correctly

### TC-4: Re-run UAT button works
- Click "Re-run UAT" button
- Verify button changes to "Running" state with spinner
- Verify UAT run is triggered (API call made)

### TC-5: Submit manually link opens modal
- Click "Submit manually" link
- Verify HumanUatModal opens

### TC-6: Next milestone link
- When another active milestone exists, verify the link is shown with correct title and navigates correctly
- When no other active milestone exists, verify the link section is omitted

### TC-7: Build verification
- `cd frontend && npm run build` completes without errors
- No TypeScript compilation errors
- No unused imports or dead code warnings

## Pass Criteria

All test cases pass. The frontend build succeeds cleanly.
