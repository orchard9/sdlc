# Spec: GET /api/git/log endpoint with pagination

## Overview

Add a `GET /api/git/log` endpoint to the sdlc-server that returns paginated git commit history for the project repository. This endpoint complements the existing `GET /api/git/status` endpoint and provides the data layer for the Git Commit History UI tab.

## Requirements

### Endpoint

- **Route:** `GET /api/git/log`
- **Handler:** `get_git_log` in `crates/sdlc-server/src/routes/git.rs`

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | u32 | 1 | Page number (1-indexed) |
| `per_page` | u32 | 25 | Commits per page (max 100) |

### Response Shape

```json
{
  "commits": [
    {
      "hash": "abc123def456...",
      "short_hash": "abc123d",
      "author_name": "Jordan Smith",
      "author_email": "jordan@example.com",
      "date": "2026-03-07T15:30:00Z",
      "message": "feat: add git log endpoint",
      "subject": "feat: add git log endpoint",
      "body": ""
    }
  ],
  "page": 1,
  "per_page": 25,
  "total_commits": 142
}
```

### Field Definitions

- `hash` — full 40-character SHA-1 commit hash
- `short_hash` — abbreviated 7-character hash
- `author_name` / `author_email` — commit author identity
- `date` — ISO 8601 author date
- `message` — full commit message (subject + body)
- `subject` — first line of the commit message
- `body` — remaining lines after subject (empty string if none)
- `page` / `per_page` — echo back the pagination parameters used
- `total_commits` — total number of commits in the repository (for UI pagination controls)

### Implementation Approach

1. Use `git log` with `--format` to extract structured fields, run via `std::process::Command` (same pattern as `get_git_status`)
2. Use `--skip` and `-n` flags for pagination
3. Use `git rev-list --count HEAD` for total commit count
4. Wrap the blocking git commands in `tokio::task::spawn_blocking` (same pattern as existing git status handler)
5. Clamp `per_page` to 1..=100 to prevent abuse

### Error Handling

- Not a git repo: return `{ "error": "not_a_git_repo" }` (same pattern as git status)
- Empty repo (no commits): return `{ "commits": [], "page": 1, "per_page": 25, "total_commits": 0 }`
- Git command failure: return appropriate `AppError`

### Non-Goals

- Filtering by branch, path, or author (future enhancement)
- Commit diff content (handled by separate `git-commit-diff` feature)
- File change lists per commit (handled by separate `git-commit-diff` feature)

## Acceptance Criteria

1. `GET /api/git/log` returns paginated commit history with correct fields
2. Pagination parameters (`page`, `per_page`) work correctly with defaults
3. `per_page` is clamped to max 100
4. `total_commits` reflects actual repository commit count
5. Empty repository returns empty commits array with total_commits=0
6. Non-git directory returns `not_a_git_repo` error
7. Unit tests cover response parsing logic
8. Route is registered in `build_router_from_state`
