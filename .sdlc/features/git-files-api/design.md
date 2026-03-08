# Design: GET /api/git/files endpoint

## Architecture

This is a pure backend feature. The endpoint lives in `crates/sdlc-server/src/routes/git.rs` alongside the existing `get_git_status` handler, reusing the same pattern of `spawn_blocking` + `Command` for git operations.

## Data Model

```rust
#[derive(Debug, Serialize)]
pub struct GitFileEntry {
    pub path: String,
    pub status: String,       // "modified", "added", "deleted", "renamed", "copied",
                               // "untracked", "conflicted", "staged_modified",
                               // "staged_added", "staged_deleted", "staged_renamed", "clean"
    pub staged: bool,          // has index changes
    pub unstaged: bool,        // has worktree changes
}
```

## Endpoint Design

```
GET /api/git/files?include_clean=false
```

### Response (200)

```json
{
  "files": [
    { "path": "src/main.rs", "status": "modified", "staged": false, "unstaged": true },
    { "path": "src/lib.rs", "status": "staged_modified", "staged": true, "unstaged": false },
    { "path": "new-file.txt", "status": "untracked", "staged": false, "unstaged": true }
  ]
}
```

### Response (not a git repo)

```json
{ "error": "not_a_git_repo" }
```

## Implementation Flow

```
GET /api/git/files
       |
       v
  spawn_blocking
       |
       v
  git rev-parse --git-dir   (verify git repo)
       |
       v
  git status --porcelain=v2  (get changed files)
       |
       v
  parse_file_entries()       (new function, parses porcelain v2 into Vec<GitFileEntry>)
       |
       v
  if include_clean:
    git ls-files             (get all tracked files)
    merge with status entries (tracked files not in status = clean)
       |
       v
  Return JSON { files: [...] }
```

## Porcelain v2 Parsing for File Entries

Reuse the line-type detection from `parse_porcelain_v2` but extract file paths and map XY codes to status strings:

| Line prefix | Meaning | X status | Y status |
|---|---|---|---|
| `1 XY ...` | Ordinary changed entry | Index change | Worktree change |
| `2 XY ...` | Renamed/copied entry | Index change | Worktree change |
| `u ...` | Unmerged (conflict) | — | — |
| `? path` | Untracked | — | — |

**XY mapping**:
- X = `.` and Y = `M` → `modified` (unstaged only)
- X = `M` and Y = `.` → `staged_modified` (staged only)
- X = `M` and Y = `M` → both staged and unstaged modified
- X = `A` → `staged_added`
- X = `D` or Y = `D` → `deleted` / `staged_deleted`
- X = `R` → `staged_renamed`
- Prefix `u` → `conflicted`
- Prefix `?` → `untracked`

For entries with both X and Y changes, use the worktree (Y) status as the primary `status` field, and set both `staged` and `unstaged` to true.

## File Path Extraction

- Ordinary entries (prefix `1`): path is the last space-separated field (field index 8+)
- Renamed entries (prefix `2`): has two paths separated by tab — use the new path (after tab)
- Untracked entries (prefix `?`): path follows `? ` prefix
- Conflict entries (prefix `u`): path is the last field

## Router Registration

Add to `build_router_from_state` in `lib.rs`:

```rust
.route("/api/git/files", get(routes::git::get_git_files))
```

Place it adjacent to the existing `/api/git/status` route.

## Query Parameter Extraction

Use `axum::extract::Query` with a simple struct:

```rust
#[derive(Debug, Deserialize)]
pub struct GitFilesQuery {
    #[serde(default)]
    pub include_clean: bool,
}
```
