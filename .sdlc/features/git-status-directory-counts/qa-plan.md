# QA Plan: git-status-directory-counts

## Scope

Verify that the `GET /api/git/status` endpoint returns correct `directory_counts` data and that existing fields are unchanged.

## Test Strategy

### Unit Tests (Rust, in `git.rs`)

1. **Clean repo**: `parse_porcelain_v2` with no changed files produces `directory_counts: []`.
2. **Single directory**: One dirty file in `src/` produces `[{ directory: "src", count: 1 }]`.
3. **Multiple directories**: Files in `frontend/src`, `crates/server`, and root produce three entries sorted descending by count.
4. **Root-level files**: A file path with no `/` separator is grouped under `"."`.
5. **Renamed files**: A porcelain v2 rename entry (`2 R. ...`) uses the destination path for directory grouping.
6. **Mixed change types**: A dirty file, a staged file, and an untracked file all in the same directory produce a single entry with `count: 3`.
7. **Unmerged files**: A conflict entry (`u ...`) contributes to the directory count.

### Build Verification

- `SDLC_NO_NPM=1 cargo test --all` passes.
- `cargo clippy --all -- -D warnings` passes.

### Regression

- All existing `parse_porcelain_v2` tests continue to pass unchanged.
- Existing `GitStatus` fields (`branch`, `dirty_count`, `staged_count`, etc.) are unaffected.

## Pass Criteria

All unit tests pass. Clippy clean. No existing test regressions.
