# Code Review: Fix resolve_root to Walk Directory Tree

## Changes

**File:** `crates/sdlc-cli/src/root.rs`

- Added `find_sdlc_root(start: &Path) -> Option<PathBuf>` — walks ancestors looking for `.sdlc/`.
- Updated `resolve_root` to call `find_sdlc_root` before falling back to CWD.
- Replaced `falls_back_to_cwd` test with three targeted tests covering the new behavior.

## Findings

**No issues found.**

- Logic is correct and minimal — two functions, clear separation of concerns.
- No `unwrap()` in library code; `find_sdlc_root` returns `Option`, `resolve_root` handles the fallback.
- All four unit tests pass: explicit root, `.sdlc/` in CWD, `.sdlc/` in grandparent, no `.sdlc/` found.
- `cargo clippy --package sdlc-cli -- -D warnings` passes clean.
- Integration test failures (110) are pre-existing on `main` and unrelated to this change (binary-not-found errors in the test harness).

## Verdict

APPROVED — ready to advance to audit.
