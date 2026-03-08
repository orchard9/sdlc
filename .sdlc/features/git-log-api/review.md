# Code Review: git-log-api

## Summary

Added `GET /api/git/log` endpoint with pagination support to the sdlc-server. The implementation follows the established patterns in `git.rs` for running git commands via `spawn_blocking` and parsing structured output.

## Files Changed

- `crates/sdlc-server/src/routes/git.rs` — Added `CommitEntry`, `GitLogQuery`, `GitLogResponse` structs, `get_git_log` handler, `collect_git_log` function, `parse_git_log_output` parser, and 6 unit tests. Also added `get_commit_detail` stub for the git-commit-diff feature.
- `crates/sdlc-server/src/lib.rs` — Registered `/api/git/log` route in `build_router_from_state`.

## Findings

### 1. Separator strategy is sound
Using `\x1e` (record separator) between fields and `\x1d` (group separator) between commits avoids conflicts with commit message content. This is the correct approach for structured git output parsing.

### 2. Pagination implementation is correct
- `page` defaults to 1, clamped to minimum 1
- `per_page` defaults to 25, clamped to 1..=100
- `skip = (page - 1) * per_page` correctly computes offset
- `git rev-list --count HEAD` efficiently gets total count

### 3. Error handling follows existing patterns
- Not-a-git-repo returns `{ "error": "not_a_git_repo" }` (consistent with `get_git_status`)
- Empty repo (no commits) gracefully returns empty array
- Git command failures propagate as `AppError`

### 4. Parser is resilient
- Handles empty output, whitespace-only records, missing body field
- Uses `splitn(7, ...)` to prevent body content from being split

### 5. Test coverage
6 unit tests covering: multi-commit parsing, empty output, single commit, multi-line body, special characters, whitespace-only records. All pass.

## Verdict

No blocking issues found. Code is clean, follows existing patterns, and has solid test coverage.
