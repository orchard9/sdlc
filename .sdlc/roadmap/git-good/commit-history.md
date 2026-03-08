# Commit History API & UI

## New Endpoint

### `GET /api/git/log?limit=<n>&offset=<n>`
Returns recent commit history for the workspace.

```json
{
  "commits": [
    {
      "hash": "abc1234",
      "hash_short": "abc1234",
      "author": "Xist",
      "email": "xist@example.com",
      "date": "2026-03-07T19:30:00Z",
      "message": "feat: add tunnel preflight check",
      "files_changed": 5,
      "insertions": 140,
      "deletions": 7
    }
  ],
  "total": 128,
  "has_more": true
}
```

**Implementation**: `git log --format=<custom> --numstat` parsed into structs.
Default limit: 25. Max limit: 100. Offset for pagination.

## UI Placement

Commit history lives in the Git page as a third view mode alongside File Browser and Diff Viewer:

- **Tab bar** at top of Git page: `Files | History`
- History tab shows a compact commit list (hash, message, author, relative time)
- Clicking a commit shows the files changed in that commit in the file browser panel
- Clicking a file from a commit shows the diff for that file *in that commit* (not working tree)

## Commit-Specific Diffs

New endpoint variant needed:

### `GET /api/git/diff?path=<file>&commit=<hash>`
Returns diff for a specific file in a specific commit (against its parent).

**Implementation**: `git diff <hash>^..<hash> -- <path>`

## Milestone Impact

This adds scope to Milestone 2 (File Browser) and Milestone 3 (Diff Viewer):
- Milestone 2 gains the History tab and commit list
- Milestone 3 gains commit-specific diff viewing
- Consider splitting History into its own Milestone 4 if scope creep is a concern

⚑ Decided: Expose commit history via /api/git/log
⚑ Decided: History tab in Git page alongside Files
⚑ Decided: Commit-click shows files changed, file-click shows commit diff
? Open: Should History be Milestone 4 or folded into M2/M3?