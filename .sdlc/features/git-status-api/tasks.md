# Tasks: git-status-api

## Task Breakdown

1. **Create git route module with GitStatus struct and git command parsing** — Add `crates/sdlc-server/src/routes/git.rs` with the `GitStatus` struct, `collect_git_status()` function that runs `git status --porcelain=v2 --branch`, parses the output into the struct fields, and computes severity and summary.

2. **Create get_git_status handler and register route** — Add the `get_git_status` async handler that uses `spawn_blocking` to call `collect_git_status`, handles the not-a-git-repo case, and register `pub mod git` in `routes/mod.rs` plus the `.route("/api/git/status", ...)` in `lib.rs`.

3. **Add tests for parsing and severity logic** — Unit tests for porcelain v2 output parsing (branch name, ahead/behind, staged/dirty/untracked/conflict counts) and severity computation (green/yellow/red thresholds). Integration test calling the endpoint against a temp git repo.
