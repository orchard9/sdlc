# Spec: Commit-specific file viewing and diff display

## Problem

The Git Commit History milestone adds a History tab showing commit logs. When a user clicks on a specific commit, they need to see what files changed and the actual diff content for that commit. Without this, the commit history is just a list of messages with no way to inspect what actually happened.

## Solution

Add a backend API endpoint and frontend UI component that, given a commit SHA, returns the list of files changed and the unified diff output for that commit. The frontend renders this as an expandable commit detail view within the History tab.

### API: `GET /api/git/show/:sha`

Returns the commit metadata and diff for a single commit.

**Response shape:**
```json
{
  "sha": "abc123...",
  "author": "Name <email>",
  "date": "2026-03-07T12:00:00Z",
  "message": "feat: add something",
  "files": [
    {
      "path": "src/main.rs",
      "status": "modified",
      "additions": 10,
      "deletions": 3
    }
  ],
  "diff": "diff --git a/src/main.rs b/src/main.rs\n..."
}
```

**Implementation:** Uses `git show --stat --format=... <sha>` and `git diff-tree -p <sha>` via `std::process::Command`, run inside `tokio::task::spawn_blocking`. Follows the same pattern as the existing `get_git_status` handler in `crates/sdlc-server/src/routes/git.rs`.

**Validation:** The SHA parameter is validated to contain only hex characters (0-9, a-f) and be between 4-40 characters. Invalid SHAs return 400 Bad Request.

### Frontend: CommitDetail component

A React component that:
1. Accepts a commit SHA as a prop
2. Fetches `GET /api/git/show/:sha` on mount
3. Renders a file list with addition/deletion counts
4. Renders the unified diff with syntax-highlighted hunks
5. Shows loading and error states

The component is used by the History tab (from `git-history-tab` feature) when a user clicks/expands a commit row.

## Dependencies

- `git-log-api` — provides the commit list that feeds SHAs into this feature
- `git-history-tab` — the UI shell that hosts the CommitDetail component

## Out of Scope

- Side-by-side diff view (future enhancement)
- Diff between arbitrary commits (only single-commit diffs)
- File content viewing at a specific commit (covered by `git-file-browser-ui`)
- Inline commenting on diffs

## Acceptance Criteria

1. `GET /api/git/show/:sha` returns commit metadata, file stats, and unified diff for a valid SHA
2. Invalid or nonexistent SHAs return appropriate HTTP error codes (400 for malformed, 404 for not found)
3. The frontend renders the file change list with addition/deletion indicators
4. Diff hunks are rendered with proper line coloring (green for additions, red for deletions)
5. The component handles loading and error states gracefully
6. Large diffs are truncated with a "diff too large" message (threshold: 100KB)
