# Design: GET /api/git/log endpoint with pagination

## Architecture

This is a backend-only feature. The endpoint lives in the existing `crates/sdlc-server/src/routes/git.rs` module alongside `get_git_status`.

## Data Flow

```
Browser/Frontend
      |
      | GET /api/git/log?page=1&per_page=25
      v
  axum Router
      |
      v
  get_git_log() handler
      |
      | tokio::task::spawn_blocking
      v
  collect_git_log(root, page, per_page)
      |
      | std::process::Command
      v
  git rev-list --count HEAD         --> total_commits
  git log --format=<sep> --skip=N -n=M  --> commit entries
      |
      v
  parse_git_log_output(stdout)  --> Vec<CommitEntry>
      |
      v
  JSON response { commits, page, per_page, total_commits }
```

## Structs

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitEntry {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct GitLogQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
```

## Git Command Strategy

Use a record separator (`\x1e`) between fields and a group separator (`\x1d`) between commits to avoid ambiguity with commit message content:

```bash
git log --format="%H%x1e%h%x1e%an%x1e%ae%x1e%aI%x1e%s%x1e%b%x1d" --skip=0 -n 25
```

For total count:
```bash
git rev-list --count HEAD
```

## Pagination Logic

- `skip = (page - 1) * per_page`
- `per_page` clamped to `1..=100`, default `25`
- `page` defaults to `1`, minimum `1`

## Error Cases

| Condition | Response |
|-----------|----------|
| Not a git repo | `{ "error": "not_a_git_repo" }` |
| Empty repo (no commits) | `{ "commits": [], "page": 1, "per_page": 25, "total_commits": 0 }` |
| Git failure | `AppError` with message |

## Route Registration

Add to `build_router_from_state` in `lib.rs`:
```rust
.route("/api/git/log", get(routes::git::get_git_log))
```

Place adjacent to the existing `/api/git/status` route.

## Testing Strategy

- Unit tests for `parse_git_log_output` with synthetic input (same pattern as `parse_porcelain_v2` tests)
- Test edge cases: empty output, single commit, multi-line body, special characters in messages
