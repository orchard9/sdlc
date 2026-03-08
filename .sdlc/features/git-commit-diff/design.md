# Design: Commit-specific file viewing and diff display

## Architecture

### Backend

**New handler in `crates/sdlc-server/src/routes/git.rs`:**

```
GET /api/git/show/:sha
```

The handler:
1. Validates the SHA parameter (hex chars only, 4-40 length)
2. Runs `git show --format='%H%n%an <%ae>%n%aI%n%s' --stat --no-patch <sha>` for metadata + file stats
3. Runs `git diff-tree -p --no-commit-id <sha>` for the unified diff
4. Parses file stats from `--stat` output (path, insertions, deletions)
5. Truncates diff output at 100KB, appending a truncation marker
6. Returns JSON response

**Route registration in `lib.rs`:**
```rust
.route("/api/git/show/:sha", get(routes::git::get_commit_detail))
```

### Data Structures

```rust
#[derive(Serialize)]
pub struct CommitDetail {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub files: Vec<FileChange>,
    pub diff: String,
    pub truncated: bool,
}

#[derive(Serialize)]
pub struct FileChange {
    pub path: String,
    pub status: String,       // "added", "modified", "deleted", "renamed"
    pub additions: u32,
    pub deletions: u32,
}
```

### Frontend

**`frontend/src/components/CommitDetail.tsx`** — React component:

```
Props: { sha: string; onClose?: () => void }
```

- Fetches `/api/git/show/{sha}` on mount
- Renders file change summary (colored +/- counts)
- Renders diff with line-by-line coloring:
  - Lines starting with `+` (not `+++`): green background
  - Lines starting with `-` (not `---`): red background
  - `@@` hunk headers: blue/purple text
  - Everything else: default
- Uses `<pre>` for diff rendering (monospace, preserves whitespace)
- Shows loading spinner during fetch, error message on failure
- Shows truncation notice when `truncated: true`

### Integration Point

The `git-history-tab` feature will import `CommitDetail` and render it when a commit row is expanded/clicked. This feature only builds the component and API — the integration is owned by `git-history-tab`.

## Error Handling

| Scenario | HTTP Status | Response |
|---|---|---|
| Invalid SHA format | 400 | `{"error": "invalid_sha"}` |
| SHA not found | 404 | `{"error": "commit_not_found"}` |
| Not a git repo | 200 | `{"error": "not_a_git_repo"}` |
| Git command failure | 500 | `{"error": "git_error", "message": "..."}` |
