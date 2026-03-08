# Tasks: Git Status Chip

## Task 1: Create useGitStatus hook
Implement the `useGitStatus` custom React hook that polls `GET /api/git/status` on a configurable interval. Handle visibility-based pause/resume and window focus re-fetch. Return `{ status, loading, error }`.

## Task 2: Create GitStatusChip component
Build the `GitStatusChip` React component that renders the severity dot, branch name, summary text, and conditional commit button. Support both expanded and collapsed sidebar modes.

## Task 3: Integrate GitStatusChip into Sidebar
Add the `GitStatusChip` to the Sidebar's bottom utility section, passing the `collapsed` prop. Position it as the first item above the existing utility buttons.

## Task 4: Add commit button action
Wire the commit button to `POST /api/git/commit` (or a placeholder that shows a toast if the endpoint doesn't exist yet). Re-fetch git status after a successful commit.
