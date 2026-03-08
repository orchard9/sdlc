# QA Plan: Git Commit Agent

## Test Strategy

### 1. Compilation Verification
- `SDLC_NO_NPM=1 cargo build --all` completes without errors.
- `cargo clippy --all -- -D warnings` passes cleanly.
- `SDLC_NO_NPM=1 cargo test --all` passes.

### 2. Route Registration
- Verify `/api/git/commit` is registered as a POST route in `lib.rs`.
- Verify the handler function `start_git_commit` exists and compiles.

### 3. SSE Variant
- Verify `GitCommitCompleted` variant exists in `SseMessage` enum.
- Verify it serializes correctly (snake_case: `git_commit_completed`).

### 4. Agent Run Pattern
- Verify `start_git_commit` calls `spawn_agent_run` with appropriate key, prompt, and options.
- Verify the agent prompt instructs reading the diff and running `sdlc commit`.
- Verify `max_turns` is reasonable (10 turns).

### 5. Frontend SSE Integration
- Verify `GitStatusChip` handles the `git_commit_completed` SSE event.
- Verify git status is refetched after commit completion.

### 6. Duplicate Run Protection
- `spawn_agent_run` returns 409 if "git-commit" key is already running (built-in behavior).

## Pass Criteria

All compilation, clippy, and test checks pass. The endpoint follows the established `spawn_agent_run` pattern used by advisory, ponder, and other agent endpoints.
