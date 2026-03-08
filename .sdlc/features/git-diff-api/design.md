# Design: GET /api/git/diff endpoint

## Architecture

This feature adds a single read-only endpoint to the existing `routes/git.rs` module. It follows the same pattern as `get_git_status`: extract `AppState`, spawn a blocking task to run git commands, parse output, and return JSON.

## Handler Signature

```rust
pub async fn get_git_diff(
    State(app): State<AppState>,
    Query(params): Query<DiffParams>,
) -> Result<Json<serde_json::Value>, AppError>
```

### Query Parameters

```rust
#[derive(Deserialize)]
pub struct DiffParams {
    pub path: Option<String>,
    pub staged: Option<bool>,
}
```

## Data Flow

```
Client                    Handler                  spawn_blocking
  │                          │                          │
  ├─ GET /api/git/diff ─────►│                          │
  │   ?path=src/lib.rs       │                          │
  │                          ├── validate path ──┐      │
  │                          │                   │      │
  │                          │◄──────────────────┘      │
  │                          │                          │
  │                          ├── spawn_blocking ───────►│
  │                          │                          ├─ git rev-parse --git-dir
  │                          │                          ├─ git status --porcelain=v2 -- <path>
  │                          │                          ├─ git diff [--cached] -- <path>
  │                          │◄─────── DiffResult ──────┤
  │                          │                          │
  │◄─── 200 JSON ────────────┤                          │
```

## Internal Types

```rust
#[derive(Serialize)]
pub struct DiffResult {
    pub path: String,
    pub diff: String,
    pub status: String,
    pub is_new: bool,
    pub is_deleted: bool,
    pub is_binary: bool,
}
```

## Path Validation

Before any git operations, the handler:
1. Checks `path` parameter is present (400 if missing).
2. Rejects paths containing `..` components (400 `invalid_path`).
3. Verifies the file exists on disk relative to `app.root` (404 if not found). Exception: deleted files (tracked by git but removed from disk) are allowed.

## Git Command Strategy

### Determine file status
```bash
git status --porcelain=v2 -- <path>
```
Parse the first character of the output line:
- `1` (ordinary) → extract XY flags to determine modified/added/deleted
- `2` (renamed/copied) → status is "renamed"
- `?` (untracked) → status is "untracked"
- No output → file is unchanged (empty diff)

### Get diff content
- **Modified/staged files**: `git diff [--cached] -- <path>`
- **Untracked files**: `git diff --no-index /dev/null <path>` (exits with code 1, which is expected)
- **Deleted files**: `git diff -- <path>` (shows removal)

### Binary detection
Check diff output for the string `"Binary files"` at the start of a line. If found, set `is_binary: true` and include the binary marker line in `diff`.

## Route Registration

Add to `lib.rs` router, adjacent to the existing git/status route:

```rust
.route("/api/git/diff", get(routes::git::get_git_diff))
```

## Error Handling

All errors use `AppError` (wrapping `anyhow::Error`). No `unwrap()` calls. The `not_a_git_repo` check reuses the existing `is_not_git_repo` helper from `git.rs`.

## Testing Strategy

Unit tests for:
- Path validation (reject `..`, accept normal paths)
- Status parsing from porcelain v2 output
- Binary detection in diff output
- DiffResult serialization
