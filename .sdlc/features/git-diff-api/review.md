# Code Review: GET /api/git/diff endpoint

## Summary

The implementation adds a `GET /api/git/diff` endpoint to `routes/git.rs` that returns unified diff output for a single file. It follows the established patterns from the git-status-api feature.

## Findings

### 1. Path validation is solid
`validate_diff_path` uses `std::path::Component::ParentDir` matching, which correctly handles all traversal variants (`..`, `foo/../bar`, `../../../etc/passwd`). This is the right approach rather than string matching.

### 2. No unwrap() in production code
All error paths use `?` with `anyhow::Error` or explicit `map_err`. The only `unwrap_or` calls are in parsing where defaults are appropriate (e.g., `unwrap_or("")` for empty porcelain output).

### 3. Spawn blocking pattern is correct
Git commands run inside `tokio::task::spawn_blocking`, consistent with the existing `get_git_status` handler. This prevents blocking the async runtime.

### 4. Untracked file handling
Uses `git diff --no-index /dev/null <path>` which is the standard approach. The exit code 1 from git is correctly handled (not treated as an error).

### 5. Deleted file support
The handler checks `git ls-files` when a file doesn't exist on disk, allowing diffs for deleted-but-tracked files. This is a good edge case to handle.

### 6. Binary detection
Uses line-based check for `"Binary files"` prefix, which matches git's output format.

### 7. Error responses use JSON consistently
All error cases return 200 with JSON error objects (matching the git-status pattern), except for truly unexpected errors which propagate as 500 via `AppError`.

## Test Coverage

- Path validation: 2 tests (reject traversal, accept normal paths)
- File status parsing: 6 tests (modified, added, deleted, renamed, untracked, empty)
- Binary detection: 2 tests (binary and non-binary)
- Serialization: 1 test
- Total: 11 new tests, all passing

## Verdict

Approved. The implementation is clean, follows established patterns, handles edge cases well, and has comprehensive test coverage.
