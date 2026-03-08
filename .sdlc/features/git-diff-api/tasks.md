# Tasks: GET /api/git/diff endpoint

## T1: Add DiffParams, DiffResult structs and path validation logic

Add the `DiffParams` query parameter struct and `DiffResult` response struct to `routes/git.rs`. Implement path validation (reject `..` traversal, verify file existence with deleted-file exception).

## T2: Implement collect_git_diff function with git command execution

Implement the core `collect_git_diff` function that:
- Checks if directory is a git repo (reuse existing pattern)
- Runs `git status --porcelain=v2 -- <path>` to determine file status
- Runs the appropriate `git diff` command based on status and staged flag
- Detects binary files
- Returns a `DiffResult`

## T3: Add get_git_diff handler and register route

Create the `get_git_diff` async handler that validates params, calls `spawn_blocking` with `collect_git_diff`, and handles errors. Register `.route("/api/git/diff", get(routes::git::get_git_diff))` in `lib.rs`.

## T4: Add unit tests for path validation, status parsing, and binary detection

Write tests covering:
- Path validation rejects `..` components
- Porcelain v2 status parsing for modified, added, deleted, renamed, untracked files
- Binary detection in diff output
- Empty diff for unchanged files
