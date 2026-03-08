# Design: Git Commit Agent

## Architecture

```
GitStatusChip (frontend)
    |
    | POST /api/git/commit
    v
start_git_commit() handler (git.rs)
    |
    | spawn_agent_run("git-commit", prompt, opts, ...)
    v
Agent Task (async)
    |
    |-- git diff HEAD --stat / git diff HEAD
    |-- Generate conventional commit message
    |-- sdlc commit --message "<msg>"
    |
    v
SSE: GitCommitCompleted
    |
    v
GitStatusChip refetches /api/git/status
```

## Backend Changes

### File: `crates/sdlc-server/src/routes/git.rs`

Add to existing file:

```rust
use super::runs::{sdlc_query_options, spawn_agent_run};
use crate::state::SseMessage;

pub async fn start_git_commit(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let prompt = /* agent prompt: read diff, generate message, run sdlc commit */;
    let opts = sdlc_query_options(app.root.clone(), 10, None);
    spawn_agent_run(
        "git-commit".to_string(),
        prompt,
        opts,
        &app,
        "git_commit",
        "Git commit",
        Some(SseMessage::GitCommitCompleted),
    ).await
}
```

### File: `crates/sdlc-server/src/state.rs`

Add `GitCommitCompleted` variant to `SseMessage` enum.

### File: `crates/sdlc-server/src/lib.rs`

Register route:
```rust
.route("/api/git/commit", post(routes::git::start_git_commit))
```

## Agent Prompt Design

The agent prompt instructs:
1. Run `git diff HEAD --stat` to get an overview of changes.
2. Run `git diff HEAD` to read the full diff.
3. Generate a conventional-commit message (feat:/fix:/refactor:/docs:/chore:/test:) that describes what changed and why, 120 chars max.
4. Execute `sdlc commit --message "<message>"`.
5. Report the result.

The agent gets `max_turns: 10` — enough for diff inspection + commit execution.

## SSE Event

A new `GitCommitCompleted` variant in `SseMessage` signals the frontend to refetch git status. This follows the same pattern as `AdvisoryRunCompleted`, `PonderRunCompleted`, etc.

## Frontend Changes

The `GitStatusChip` already calls the endpoint. Minimal changes needed:
- After `POST /api/git/commit` returns the run_id, the chip can show a spinner.
- The SSE `git_commit_completed` event triggers `refetch()` on the git status hook.

## Error Handling

- If no changes to commit: agent runs `git status --short`, sees clean tree, reports "nothing to commit" — run completes without error.
- If agent already running: `spawn_agent_run` returns 409 Conflict (built-in dedup).
- If commit fails (e.g., not on main): agent reports the error in its output.
