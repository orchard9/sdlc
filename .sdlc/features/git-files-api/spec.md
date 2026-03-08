# Spec: GET /api/git/files endpoint returning workspace files with status

## Overview

Add a `GET /api/git/files` endpoint to `sdlc-server` that returns a list of all files in the workspace directory along with their git status (modified, staged, untracked, conflicted, etc.). This endpoint powers the Git File Browser UI milestone feature, giving the frontend a structured JSON response it can render as a file tree or flat list.

## Problem

The existing `GET /api/git/status` endpoint provides aggregate repository health (dirty count, staged count, severity) but does not enumerate individual files. The Git File Browser UI needs per-file data: file paths, their git status category, and enough metadata to render a tree view with status indicators.

## Requirements

### Functional

1. **Endpoint**: `GET /api/git/files` registered in the server router alongside `GET /api/git/status`.
2. **Response shape**: JSON array of file entries, each containing:
   - `path` (string) — relative path from workspace root
   - `status` (string) — one of: `modified`, `added`, `deleted`, `renamed`, `copied`, `untracked`, `conflicted`, `staged_modified`, `staged_added`, `staged_deleted`, `staged_renamed`, `clean`
   - `staged` (bool) — whether the file has staged (index) changes
   - `unstaged` (bool) — whether the file has unstaged (worktree) changes
3. **Query parameters**:
   - `include_clean` (bool, default `false`) — when true, include tracked files with no changes (uses `git ls-files` to enumerate all tracked files not already covered by status output)
4. **Data source**: Parse `git status --porcelain=v2` output to extract per-file entries, reusing the existing `spawn_blocking` + `Command` pattern from `get_git_status`.
5. **Not-a-git-repo handling**: Return `{"error": "not_a_git_repo"}` (200) when the workspace is not a git repository, matching the existing `get_git_status` behavior.
6. **Error handling**: Return appropriate `AppError` for git command failures.

### Non-Functional

1. **Performance**: The endpoint runs git commands in a blocking task (`spawn_blocking`) to avoid blocking the async runtime. For large repositories, `include_clean=false` (default) keeps the response size proportional to changed files only.
2. **Consistency**: Reuse the same porcelain v2 parsing infrastructure from `git.rs` where possible.
3. **No new dependencies**: Uses only `std::process::Command`, `serde`, and existing crate dependencies.

## Out of Scope

- Directory-level aggregation (e.g., "3 modified files in src/") — that is a frontend concern or a separate feature.
- File content retrieval — a separate `git-diff-api` feature handles diffs.
- Pagination — workspace file lists are bounded by repository size; pagination is unnecessary for typical projects.

## Acceptance Criteria

1. `GET /api/git/files` returns 200 with a JSON array of file entries reflecting the current git status.
2. Each entry has `path`, `status`, `staged`, and `unstaged` fields.
3. With `include_clean=true`, tracked clean files are included in the response.
4. With `include_clean=false` (default), only files with changes or untracked files appear.
5. Non-git-repo workspaces return `{"error": "not_a_git_repo"}`.
6. Unit tests cover parsing of porcelain v2 output into file entries.
7. The endpoint is registered in the router and accessible.
