# Architecture: API Contract

## Endpoints

### `GET /api/git/status`
Returns composite git status for the workspace.

```json
{
  "branch": "main",
  "has_remote": true,
  "detached": false,
  "has_uncommitted_changes": true,
  "has_staged_changes": false,
  "modified_count": 3,
  "untracked_count": 1,
  "unpushed_commits": 2,
  "unpulled_commits": 0,
  "merge_in_progress": false,
  "rebase_in_progress": false,
  "severity": "yellow"
}
```

`severity` is computed server-side: green/yellow/red based on the composite state.
This endpoint is polled every 5s (local git ops only — cheap).

### `GET /api/git/files`
Returns all workspace files with git status.

```json
{
  "files": [
    { "path": "src/main.rs", "status": "M", "staged": false },
    { "path": "src/new.rs", "status": "??", "staged": false },
    { "path": "old.rs", "status": "D", "staged": true }
  ]
}
```

### `GET /api/git/diff?path=<relative_path>`
Returns unified diff for a single file.

```json
{
  "path": "src/main.rs",
  "old_path": null,
  "is_binary": false,
  "diff": "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,5 +1,6 @@...",
  "too_large": false
}
```

For untracked files: returns full content as all-addition diff.
For binary files: `is_binary: true`, `diff: null`.
For very large diffs: `too_large: true`, diff truncated at ~2000 lines.

## Implementation Notes

- All git commands run via `std::process::Command` (no libgit2)
- `git status --porcelain=v2` for structured status parsing
- `git rev-list --count HEAD..@{upstream}` / `@{upstream}..HEAD` for ahead/behind
- `git diff HEAD -- <path>` for file diffs (includes staged + unstaged)
- Remote fetch cached, refreshed every 60s in background

⚑ Decided: Three endpoints, clean separation of concerns
⚑ Decided: Server-side severity computation
⚑ Decided: Untracked files get synthetic all-addition diffs