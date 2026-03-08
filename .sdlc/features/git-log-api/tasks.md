# Tasks: git-log-api

## Task List

1. **Add CommitEntry struct and git log parser** — Define `CommitEntry`, `GitLogQuery` structs and `parse_git_log_output` function in `git.rs`. Parse the separator-delimited git log output into a `Vec<CommitEntry>`.

2. **Add collect_git_log function** — Implement `collect_git_log(root, page, per_page)` that runs `git rev-list --count HEAD` and `git log --format=... --skip=N -n M`, returning the parsed commits and total count.

3. **Add get_git_log handler and register route** — Create the `get_git_log` async handler with query parameter extraction, spawn_blocking, and JSON response. Register `/api/git/log` route in `build_router_from_state`.

4. **Add unit tests for parse_git_log_output** — Test parsing with clean output, empty repo, single commit, multi-line body, and special characters in commit messages.
