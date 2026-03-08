# Tasks: Commit-specific file viewing and diff display

## T1: Add GET /api/git/show/:sha backend endpoint
Add `CommitDetail` and `FileChange` structs to `git.rs`. Implement `get_commit_detail` handler that validates the SHA, runs `git show` and `git diff-tree` commands via `spawn_blocking`, parses the output, and returns JSON. Register the route in `lib.rs`. Include SHA validation (hex only, 4-40 chars) and diff truncation at 100KB.

## T2: Add unit tests for commit detail parsing
Add tests in `git.rs` for: parsing commit metadata from `git show` output, parsing file stats from `--numstat` output, SHA validation (valid, invalid chars, too short, too long), and diff truncation logic.

## T3: Create CommitDetail React component
Build `frontend/src/components/CommitDetail.tsx` that fetches `/api/git/show/{sha}`, renders file change list with colored addition/deletion counts, renders unified diff with line-level coloring (green additions, red deletions, blue hunk headers), and handles loading/error/truncation states.
