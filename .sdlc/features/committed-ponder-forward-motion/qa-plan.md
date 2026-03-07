# QA Plan: Committed Ponder Forward Motion

## Test Scenarios

### 1. Milestone links visible for committed ponder
- Navigate to a ponder with status `committed` and non-empty `committed_to`
- **Expect:** A banner below the header shows each milestone slug as a clickable link
- **Expect:** Links navigate to `/milestones/{slug}`

### 2. Milestone links hidden for non-committed ponders
- Navigate to a ponder with status `exploring` or `converging`
- **Expect:** No milestone links banner is shown

### 3. Prepare button visible and functional
- Navigate to a committed ponder with at least one milestone in `committed_to`
- **Expect:** A "Prepare" button with Play icon appears in the header
- Click the button
- **Expect:** Button transitions to loading state ("Preparing…" with spinner)
- **Expect:** POST request sent to `/api/milestone/{slug}/prepare`

### 4. Prepare button hidden for non-committed ponders
- Navigate to an exploring/converging ponder
- **Expect:** No Prepare button in the header

### 5. Empty state shows milestone links for committed ponder
- Navigate to a committed ponder with no sessions
- **Expect:** The empty state shows milestone links instead of "Start from title & brief" button

### 6. Type-check passes
- Run `npx tsc --noEmit` in frontend/
- **Expect:** No type errors

## Verification Method

Manual browser testing against local dev server + TypeScript type-check.
