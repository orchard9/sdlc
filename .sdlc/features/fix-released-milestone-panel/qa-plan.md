# QA Plan: Fix released milestone showing verifying UI

## Test 1: Released milestone shows released indicator
- Navigate to a milestone detail page where `status === 'released'`
- Verify the panel shows "Released" text with a green check icon
- Verify the "Run UAT" button is NOT present
- Verify the "Submit manually" link is NOT present

## Test 2: Verifying milestone still shows verifying UI
- Navigate to a milestone detail page where `status === 'verifying'` and all features are released
- Verify the "All features released" text is shown
- Verify the "Run UAT" button is present
- Verify the "Submit manually" link is present

## Test 3: Active milestone with waves shows wave plan
- Navigate to a milestone detail page where `status === 'active'` and waves exist
- Verify the wave plan UI renders (progress bar, wave details)
- Verify no released or verifying indicator is shown

## Test 4: TypeScript compilation
- Run `npm run build` in the frontend directory
- Verify no type errors related to the `milestoneStatus` prop

## Test 5: No visual regression on feature list
- On any milestone detail page, verify the features list still renders with reorder buttons
- Verify the UAT History section still appears below
