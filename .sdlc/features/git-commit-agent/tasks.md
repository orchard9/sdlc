# Tasks: Git Commit Agent

## Task 1: Add GitCommitCompleted SSE variant
Add `GitCommitCompleted` to the `SseMessage` enum in `crates/sdlc-server/src/state.rs` with proper serialization.

## Task 2: Implement POST /api/git/commit endpoint
Add `start_git_commit` handler in `crates/sdlc-server/src/routes/git.rs` that uses `spawn_agent_run` with a prompt instructing the agent to read the diff, generate a commit message, and run `sdlc commit`.

## Task 3: Register the route in lib.rs
Add `.route("/api/git/commit", post(routes::git::start_git_commit))` to the router in `crates/sdlc-server/src/lib.rs`.

## Task 4: Update GitStatusChip for SSE refresh
Update `GitStatusChip` to listen for `git_commit_completed` SSE events and auto-refresh git status.

## Task 5: Build and test
Run `SDLC_NO_NPM=1 cargo build --all` and `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` to verify everything compiles and passes.
