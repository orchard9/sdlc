# QA Plan: History Tab UI with Compact Commit List

## Test Strategy

### 1. Unit Tests

**relativeTime utility** (`frontend/src/lib/relativeTime.test.ts`)
- Returns "just now" for timestamps within 60 seconds
- Returns "X min ago" for timestamps within the hour
- Returns "X hours ago" for timestamps within the day
- Returns "X days ago" for timestamps beyond 24 hours
- Handles invalid/null input gracefully

**useGitLog hook** (tested via component tests)
- Fetches initial commits on mount
- Sets loading state while fetching
- Sets error state on API failure
- Handles 404 gracefully (API not available)
- loadMore appends commits to existing list
- Sets hasMore to false when no more commits

### 2. Component Tests

**GitHistoryTab** (`frontend/src/components/git/GitHistoryTab.test.tsx`)
- Renders commit rows with hash, message, author, time
- Shows skeleton rows while loading
- Shows empty state when no commits
- Shows error state with retry button on API failure
- "Load more" button triggers additional fetch
- "Load more" button hidden when hasMore is false

**GitPage** (`frontend/src/pages/GitPage.test.tsx`)
- Renders tab bar with "History" tab active
- History tab content is visible by default

### 3. Integration Tests

- Navigate to `/git` from sidebar and verify page renders
- Verify commit data matches API response
- Verify pagination loads additional commits

### 4. Manual Verification

- Visual inspection of commit row layout and truncation
- Verify relative time formatting accuracy
- Test responsive behavior on narrow viewports
- Verify skeleton loading animation smoothness

## Pass Criteria

- All unit and component tests pass
- No TypeScript errors (`npm run build` succeeds)
- Commit list renders correctly with real git data
- Error states display appropriately
- Page is accessible via sidebar navigation
