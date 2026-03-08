# Spec: Per-directory file counts in git status API

## Overview

Extend the existing `GET /api/git/status` response to include a `directory_counts` field that groups changed files (dirty, staged, untracked, conflicted) by their parent directory. This gives the frontend the data needed to show which directories have the most churn at a glance.

## Motivation

The current git status API returns flat aggregate counts (`dirty_count`, `staged_count`, `untracked_count`). Users see "12 dirty files" but have no idea where those changes are concentrated. Per-directory counts let the UI surface hotspots — e.g., "frontend/src: 8 files, crates/sdlc-server: 4 files" — enabling faster navigation to the areas that need attention.

## API Contract

### Existing endpoint (extended)

```
GET /api/git/status
```

### New field in response

The response gains one additional field, `directory_counts`:

```json
{
  "branch": "main",
  "dirty_count": 5,
  "staged_count": 2,
  "untracked_count": 3,
  "ahead": 0,
  "behind": 0,
  "has_conflicts": false,
  "conflict_count": 0,
  "severity": "yellow",
  "summary": "5 dirty files, 2 staged, 3 untracked",
  "directory_counts": [
    { "directory": "frontend/src", "count": 4 },
    { "directory": "crates/sdlc-server/src", "count": 3 },
    { "directory": ".", "count": 3 }
  ]
}
```

### `directory_counts` schema

| Field | Type | Description |
|---|---|---|
| `directory` | `string` | Relative path of the parent directory from the repo root. Files at the root use `"."`. |
| `count` | `u32` | Total number of changed files (dirty + staged + untracked + conflicted) in that directory. |

The array is sorted descending by `count`. Only directories with at least one changed file are included. Each file contributes to exactly one directory entry (its immediate parent).

### Edge cases

- **Root-level files**: Grouped under `"."`.
- **Deeply nested files**: Only the immediate parent directory is used (e.g., `a/b/c/file.rs` counts toward `"a/b/c"`).
- **Renamed files**: The destination path is used for directory grouping.
- **Clean repo**: `directory_counts` is an empty array `[]`.
- **Not a git repo**: The existing `{ "error": "not_a_git_repo" }` response is unchanged (no `directory_counts` field).

## Implementation Approach

1. **Extract file paths during parsing**: The `parse_porcelain_v2` function already iterates over porcelain v2 lines. For ordinary entries (`1 XY ...`), the file path is the last space-separated field. For renamed entries (`2 XY ...`), the destination path follows the tab separator. For untracked entries (`? path`), the path follows the `? ` prefix.
2. **Aggregate by directory**: Use a `HashMap<String, u32>` to count files per parent directory during the same parsing pass. Extract the parent directory via simple string operations (find last `/`, take everything before it, or `"."` if no slash).
3. **Add `directory_counts` to `GitStatus`**: Add a `Vec<DirectoryCount>` field where `DirectoryCount` is a small struct with `directory: String` and `count: u32`. Sort descending by count before returning.
4. **Backward compatible**: The new field is additive. Existing consumers that do not read `directory_counts` are unaffected.

## Out of Scope

- Per-directory breakdown by change type (dirty vs staged vs untracked) — this is a flat count per directory.
- Recursive rollup (parent directories aggregating child counts) — each file counts only in its immediate parent.
- Frontend consumption of `directory_counts` — that belongs to a separate UI feature.

## Acceptance Criteria

1. `GET /api/git/status` includes a `directory_counts` array in the JSON response.
2. Each entry has `directory` (string) and `count` (u32) fields.
3. The array is sorted descending by `count`.
4. Root-level files are grouped under `"."`.
5. A clean repo returns `directory_counts: []`.
6. The existing fields and their values are unchanged.
7. Unit tests cover: clean repo, files in multiple directories, root-level files, renamed files.
