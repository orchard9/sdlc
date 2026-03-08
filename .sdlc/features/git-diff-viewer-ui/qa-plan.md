# QA Plan: git-diff-viewer-ui

## Test Strategy

Manual visual inspection and automated unit/integration tests. The component is a UI building block not yet wired to a route, so end-to-end browser tests are deferred to the Git page milestone.

## Test Cases

### TC-1: Renders unified diff correctly
- **Input**: Mock API response with a unified diff containing additions and deletions
- **Expected**: Lines render with correct line numbers, additions have blue styling, deletions have amber styling
- **Type**: Visual inspection + unit test

### TC-2: Split view toggle
- **Input**: Click "Split" toggle button
- **Expected**: View switches to side-by-side layout with old on left, new on right
- **Type**: Unit test

### TC-3: Loading state
- **Input**: Component mounts before API response arrives
- **Expected**: Skeleton loader displays with animated bars
- **Type**: Unit test

### TC-4: Error state
- **Input**: API returns 500 or network failure
- **Expected**: Error card displays with "Failed to load diff" message and retry button
- **Type**: Unit test

### TC-5: Empty diff
- **Input**: API returns empty diff (no changes)
- **Expected**: "No changes" message with check icon
- **Type**: Unit test

### TC-6: Binary file
- **Input**: API indicates file is binary
- **Expected**: "Binary file — diff not available" message
- **Type**: Unit test

### TC-7: Responsive collapse
- **Input**: Viewport width < 1024px while in split view
- **Expected**: View auto-collapses to unified mode
- **Type**: Manual verification

### TC-8: Horizontal scroll
- **Input**: Diff contains lines exceeding viewport width
- **Expected**: Horizontal scrollbar appears, no line wrapping
- **Type**: Visual inspection

## Build Verification

- `npm run build` completes without errors after adding @git-diff-view/react
- `npm run test` passes (if unit tests are added)
- No TypeScript errors in DiffViewer.tsx
