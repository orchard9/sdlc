# Design: GET /api/git/status

## Architecture

This is a backend-only feature. A single Axum route handler collects git state via subprocess calls and returns a computed JSON model.

```
Browser / Agent
      |
      v
GET /api/git/status
      |
      v
routes::git::get_git_status(State(app))
      |
      v
spawn_blocking {
    run_git_commands(app.root)
      |
      +-- git rev-parse --abbrev-ref HEAD
      +-- git status --porcelain=v2 --branch
      |
      v
    parse output -> GitStatus struct
      |
      v
    compute severity + summary
}
      |
      v
Json(GitStatus)
```

## Module: `crates/sdlc-server/src/routes/git.rs`

### Structs

```rust
#[derive(Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub dirty_count: u32,
    pub staged_count: u32,
    pub untracked_count: u32,
    pub ahead: u32,
    pub behind: u32,
    pub has_conflicts: bool,
    pub conflict_count: u32,
    pub severity: String,      // "green" | "yellow" | "red"
    pub summary: String,
}
```

### Git Commands Used

1. **`git rev-parse --abbrev-ref HEAD`** — returns branch name or `HEAD` when detached.
2. **`git status --porcelain=v2 --branch`** — machine-readable status output including:
   - `# branch.head <name>` — branch name
   - `# branch.ab +N -M` — ahead/behind counts
   - `1 <XY> ...` — ordinary changed entries (X=staged, Y=unstaged)
   - `2 <XY> ...` — renamed/copied entries
   - `u <XY> ...` — unmerged (conflict) entries
   - `? ...` — untracked files

### Parsing Strategy

Parse `git status --porcelain=v2 --branch` line-by-line:
- Lines starting with `# branch.head` -> extract branch name
- Lines starting with `# branch.ab` -> extract ahead/behind from `+N -M`
- Lines starting with `1 ` or `2 ` -> check XY: if X != `.` increment `staged_count`; if Y != `.` increment `dirty_count`
- Lines starting with `u ` -> increment `conflict_count`, set `has_conflicts = true`
- Lines starting with `? ` -> increment `untracked_count`

### Severity Computation

```
fn compute_severity(status: &GitStatus) -> &'static str {
    if status.has_conflicts || status.behind > 10 {
        "red"
    } else if status.dirty_count > 0 || status.behind > 0 || status.untracked_count > 5 {
        "yellow"
    } else {
        "green"
    }
}
```

### Summary Generation

Build a human-readable summary from non-zero fields:
- `"{N} behind upstream"` if behind > 0
- `"{N} ahead"` if ahead > 0
- `"{N} dirty files"` if dirty_count > 0
- `"{N} staged"` if staged_count > 0
- `"{N} untracked"` if untracked_count > 0
- `"{N} conflicts"` if conflict_count > 0
- `"clean"` if nothing else applies

Join with `, `.

### Error Handling

- Non-git directory: `git rev-parse` exits non-zero. Return `200` with `{ "error": "not_a_git_repo" }`.
- Git not on PATH: `Command::new("git")` returns IO error. Return `500` via `AppError`.
- All internal errors use `?` propagation — no `unwrap()`.

### Handler

```rust
pub async fn get_git_status(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        collect_git_status(&root)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

    match result {
        Ok(status) => Ok(Json(serde_json::to_value(status)?)),
        Err(e) if is_not_git_repo(&e) => {
            Ok(Json(serde_json::json!({ "error": "not_a_git_repo" })))
        }
        Err(e) => Err(AppError(e)),
    }
}
```

### Registration

In `routes/mod.rs`:
```rust
pub mod git;
```

In `lib.rs` router:
```rust
.route("/api/git/status", get(routes::git::get_git_status))
```

## Testing

- Unit test: parse known `git status --porcelain=v2 --branch` output strings and verify field extraction.
- Unit test: severity computation for each tier (green, yellow, red).
- Integration test: call the endpoint against a temp git repo (init, add file, commit) and verify JSON shape.

## Design Decisions

| Decision | Rationale |
|---|---|
| Server-only, no sdlc-core types | Git status is UI plumbing, not state machine data |
| `spawn_blocking` for git subprocess | Keeps async runtime unblocked; git commands are fast but synchronous |
| `--porcelain=v2` format | Machine-stable, won't break across git versions, includes branch tracking info |
| Single endpoint, no caching | Called on demand by the UI; git commands are <50ms for typical repos |
