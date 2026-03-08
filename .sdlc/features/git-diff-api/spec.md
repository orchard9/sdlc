# Spec: GET /api/git/diff endpoint for single file diffs

## Overview

Add a `GET /api/git/diff` REST endpoint to `sdlc-server` that returns the unified diff output for a single file in the working tree. This provides the backend data source for the git-diff-viewer UI, enabling users to see exactly what changed in any modified file.

## Motivation

The git-diff-viewer milestone needs a backend API that can serve file-level diff content. The existing `GET /api/git/status` endpoint tells users *which* files changed and the overall repo health, but not *what* changed. This endpoint fills that gap — given a file path, it returns the unified diff so the frontend can render a side-by-side or inline diff view.

## API Contract

### Request

```
GET /api/git/diff?path=<file_path>&staged=<bool>
```

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `path` | `string` | Yes | — | Relative path of the file to diff (from project root) |
| `staged` | `bool` | No | `false` | If `true`, show the staged (index) diff instead of working tree diff |

### Response (200 OK)

```json
{
  "path": "src/main.rs",
  "diff": "diff --git a/src/main.rs b/src/main.rs\nindex abc123..def456 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }",
  "status": "modified",
  "is_new": false,
  "is_deleted": false,
  "is_binary": false
}
```

### Fields

| Field | Type | Description |
|---|---|---|
| `path` | `string` | The file path that was diffed (echoed back) |
| `diff` | `string` | Raw unified diff output from git. Empty string if no changes. |
| `status` | `string` | One of `"modified"`, `"added"`, `"deleted"`, `"renamed"`, `"untracked"` |
| `is_new` | `bool` | `true` if the file is newly added (no previous version) |
| `is_deleted` | `bool` | `true` if the file was deleted |
| `is_binary` | `bool` | `true` if git reports the file as binary |

### Untracked Files

For untracked files (not yet added to git), the endpoint returns the full file content as an "add all lines" diff using `git diff --no-index /dev/null <path>`. The `status` field will be `"untracked"` and `is_new` will be `true`.

### Error Cases

| Condition | Response |
|---|---|
| Missing `path` parameter | `400` with `{ "error": "missing_path_parameter" }` |
| File does not exist | `404` with `{ "error": "file_not_found", "path": "..." }` |
| Not a git repository | `200` with `{ "error": "not_a_git_repo" }` (consistent with git/status) |
| Path traversal attempt (`../`) | `400` with `{ "error": "invalid_path" }` |

## Implementation Approach

1. **Extend existing git route module**: Add `get_git_diff` handler to `crates/sdlc-server/src/routes/git.rs`.
2. **Register in router**: Add `.route("/api/git/diff", get(routes::git::get_git_diff))` in `lib.rs`.
3. **Git diff collection**: Use `tokio::task::spawn_blocking` wrapping `std::process::Command` to run `git diff [--cached] -- <path>` and capture the output. For untracked files, use `git diff --no-index /dev/null <path>`.
4. **Path validation**: Reject paths containing `..` to prevent directory traversal. Resolve the path relative to `AppState.root`.
5. **File status detection**: Use `git status --porcelain=v2 -- <path>` to determine the file's status (modified, added, deleted, renamed, untracked).
6. **Binary detection**: Check the diff output for the `Binary files` marker that git emits for binary content.
7. **No sdlc-core dependency**: This is a server-only feature, consistent with the git-status-api pattern.

## Out of Scope

- Diffing between arbitrary commits or branches (future enhancement).
- Syntax highlighting or parsed hunk structures — the frontend handles rendering.
- Batch diffing of multiple files in a single request.
- Blame / annotation data.

## Acceptance Criteria

1. `GET /api/git/diff?path=src/main.rs` returns a valid JSON response matching the contract above for a modified file.
2. `GET /api/git/diff?path=src/main.rs&staged=true` returns the staged diff.
3. Untracked files return a full-content diff with `status: "untracked"`.
4. Path traversal attempts (`../../../etc/passwd`) are rejected with 400.
5. Missing `path` parameter returns 400.
6. Non-existent files return 404.
7. Non-git directories return `not_a_git_repo` (consistent with git/status endpoint).
8. No `unwrap()` calls — all errors handled with `?` and `AppError`.
9. The endpoint responds in under 500ms for typical files.
