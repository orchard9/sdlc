# Spec: History Tab UI with Compact Commit List

## Summary

Add a "History" tab to the existing Git page shell that displays the repository's commit log in a compact, scrollable list. The tab fetches data from `GET /api/git/log` (provided by the sibling `git-log-api` feature) and renders each commit as a compact row showing hash, author, relative timestamp, and commit message.

## Problem

The Ponder UI currently has no way to view commit history. Users must switch to a terminal to run `git log`. This breaks flow and removes context that would help developers understand recent changes, track milestone progress, and review what was committed.

## Solution

### UI Component: `GitHistoryTab`

A React component rendered as a tab within the Git page area. It displays a compact list of commits fetched from the backend API.

**Each commit row displays:**
- Short commit hash (7 chars, monospace, muted color)
- Commit message (first line only, truncated with ellipsis if too long)
- Author name
- Relative timestamp (e.g., "2 hours ago", "3 days ago")

**Layout:**
- Vertical scrollable list
- Compact rows (single-line per commit, no unnecessary padding)
- Consistent with existing Ponder UI design system (card backgrounds, muted text, border styling)

### Data Flow

1. Component mounts and calls `GET /api/git/log?limit=50`
2. Response is an array of commit objects: `{ hash, short_hash, message, author, timestamp }`
3. Commits render in reverse chronological order (newest first)
4. "Load more" button at the bottom fetches the next page using `?limit=50&offset=<n>` or cursor-based pagination

### Integration

- The tab is added to the Git page shell (which may be created by a sibling feature or already exist)
- If no Git page shell exists yet, this feature creates a minimal `/git` route with a tab bar containing "History" as the first tab
- The sidebar gains a "Git" nav entry under the "integrate" group, using the `GitBranch` icon

### Error States

- **Not a git repo:** Show a centered message "Not a git repository"
- **API error:** Show "Failed to load commit history" with a retry button
- **Empty repo:** Show "No commits yet"
- **Loading:** Skeleton/pulse animation matching existing patterns

## Non-Goals

- Commit-specific diff viewing (handled by `git-commit-diff` feature)
- Branch switching or management
- File browsing within commits
- Search/filter within commit history (future enhancement)

## Dependencies

- `git-log-api` feature provides `GET /api/git/log` endpoint. If the API is not yet available, the UI should gracefully handle 404 responses and show "Commit history not available yet."

## Acceptance Criteria

1. A "History" tab exists on the Git page showing the 50 most recent commits
2. Each commit row shows short hash, message, author, and relative time
3. Pagination loads additional commits on demand
4. Error states are handled gracefully (not a repo, empty, API errors)
5. The page is accessible via `/git` route with a sidebar link
6. The component uses the existing design system (no custom colors or fonts)
