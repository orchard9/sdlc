# Spec: GET /api/git/status endpoint with composite state model

## Overview

Add a `GET /api/git/status` REST endpoint to `sdlc-server` that shells out to `git` commands to collect repository health signals and returns a composite status model with an overall severity level (green/yellow/red).

## Motivation

The git-status-indicator milestone aims to give users glanceable repo health from every screen. This feature provides the backend data source — a single API call that aggregates branch name, dirty-file count, ahead/behind tracking, and conflict state into a unified JSON response with a computed severity.

## API Contract

### Request

```
GET /api/git/status
```

No query parameters required. The endpoint operates on the project root directory already known to `AppState`.

### Response

```json
{
  "branch": "main",
  "dirty_count": 3,
  "staged_count": 1,
  "untracked_count": 2,
  "ahead": 0,
  "behind": 2,
  "has_conflicts": false,
  "conflict_count": 0,
  "severity": "yellow",
  "summary": "2 behind upstream, 3 dirty files"
}
```

### Fields

| Field | Type | Description |
|---|---|---|
| `branch` | `string` | Current branch name (or detached HEAD sha) |
| `dirty_count` | `u32` | Number of modified (unstaged) files |
| `staged_count` | `u32` | Number of staged files |
| `untracked_count` | `u32` | Number of untracked files |
| `ahead` | `u32` | Commits ahead of upstream |
| `behind` | `u32` | Commits behind upstream |
| `has_conflicts` | `bool` | Whether merge conflicts exist |
| `conflict_count` | `u32` | Number of conflicted files |
| `severity` | `string` | One of `"green"`, `"yellow"`, `"red"` |
| `summary` | `string` | Human-readable one-line summary |

### Severity Rules

- **red**: `has_conflicts == true` OR `behind > 10`
- **yellow**: `dirty_count > 0` OR `behind > 0` OR `untracked_count > 5`
- **green**: everything else (clean working tree, up-to-date with upstream)

### Error Handling

- If the project root is not a git repository, return `200` with a JSON body: `{ "error": "not_a_git_repo" }`.
- If `git` is not found on PATH, return `500` with an appropriate error message.

## Implementation Approach

1. **New route module**: `crates/sdlc-server/src/routes/git.rs` with a `get_git_status` handler.
2. **Register in router**: Add `pub mod git;` to `routes/mod.rs` and `.route("/api/git/status", get(routes::git::get_git_status))` in `lib.rs`.
3. **Git data collection**: Use `tokio::process::Command` to run git commands (`git rev-parse --abbrev-ref HEAD`, `git status --porcelain=v2 --branch`) and parse the output. Use `spawn_blocking` if needed for synchronous parsing.
4. **Composite model**: A `GitStatus` struct with `serde::Serialize` that holds all fields, computes severity and summary.
5. **No sdlc-core dependency**: This is a server-only feature — the git status model does not belong in the core state machine. It lives entirely in `sdlc-server`.

## Out of Scope

- WebSocket / SSE streaming of git status changes (future feature).
- Git operations (commit, push, pull) — this is read-only.
- File-level diff content — that belongs to the git-diff-api feature.
- Caching or polling intervals — the endpoint is called on demand.

## Acceptance Criteria

1. `GET /api/git/status` returns a valid JSON response matching the contract above.
2. Severity is computed correctly according to the documented rules.
3. The endpoint handles non-git directories gracefully (returns `not_a_git_repo`).
4. The endpoint responds in under 500ms for typical repositories.
5. No `unwrap()` calls — all errors handled with `?` and `AppError`.
