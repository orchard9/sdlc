# Tasks: GET /api/git/files endpoint

## Task List

1. **Implement parse_file_entries function** — Add `parse_file_entries()` to `git.rs` that parses `git status --porcelain=v2` output into `Vec<GitFileEntry>`, handling ordinary, renamed, untracked, and conflict entries with correct XY mapping.

2. **Implement get_git_files handler** — Add the `get_git_files` async handler with `GitFilesQuery` extraction, `spawn_blocking` for git commands, `include_clean` support via `git ls-files`, and not-a-git-repo error handling.

3. **Register route in router** — Add `.route("/api/git/files", get(routes::git::get_git_files))` to `build_router_from_state` in `lib.rs`.

4. **Add unit tests for parse_file_entries** — Cover ordinary modified, staged, renamed, untracked, conflicted, and mixed XY entries. Test clean file merging logic.
