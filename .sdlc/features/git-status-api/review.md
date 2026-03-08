# Code Review: git-status-api

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/git.rs` | New — GitStatus struct, porcelain v2 parser, severity/summary computation, handler, 19 unit tests |
| `crates/sdlc-server/src/routes/mod.rs` | Added `pub mod git;` |
| `crates/sdlc-server/src/lib.rs` | Added `.route("/api/git/status", get(routes::git::get_git_status))` |

## Review Checklist

### Correctness

- [x] `git status --porcelain=v2 --branch` parsing handles all entry types: ordinary (`1`), renamed (`2`), unmerged (`u`), untracked (`?`), and branch headers.
- [x] Ahead/behind extracted correctly from `# branch.ab +N -M` format.
- [x] XY flag parsing correctly distinguishes staged (X != `.`) from dirty (Y != `.`).
- [x] Detached HEAD case handled (`# branch.head (detached)`).
- [x] Severity thresholds match spec: red (conflicts or behind > 10), yellow (dirty > 0, behind > 0, untracked > 5), green (clean).

### Error Handling

- [x] No `unwrap()` in production code — parsing uses `unwrap_or(0)` for numeric conversion which is safe (defaults to 0 on unexpected format).
- [x] Non-git directory detected via `git rev-parse --git-dir` and returns `{ "error": "not_a_git_repo" }` with 200 status.
- [x] Git subprocess failures propagate via `?` and `AppError`.
- [x] `spawn_blocking` used to avoid blocking the async runtime.

### Architecture

- [x] Server-only — no sdlc-core types involved. Correct per architecture principle (git status is UI plumbing).
- [x] Follows existing route patterns: `State(app)` extraction, `spawn_blocking`, `Json` response, `AppError` error type.
- [x] Route registered alongside other read-only data endpoints (state, changelog).

### Tests

- [x] 19 unit tests covering: clean repo, dirty files, untracked, conflicts, ahead/behind, detached HEAD, severity tiers, summary generation, XY extraction.
- [x] All tests pass. Clippy clean with `-D warnings`.

## Findings

1. **The `unwrap_or("0")` in ahead/behind parsing** — This is intentional defensive parsing. If git changes the format, we default to 0 rather than crashing. Acceptable.

2. **No integration test against a real temp git repo** — The unit tests thoroughly cover the parsing logic with known inputs. An integration test would require setting up a temp git repo with specific states, which adds complexity for marginal value since the handler is a thin wrapper around the parser. Accepted as-is; can add later if needed.

3. **No caching** — The endpoint runs git commands on every call. Per the spec, this is by design (called on demand, git commands are fast). No action needed.

## Verdict

**Approved.** Clean implementation following established patterns, comprehensive test coverage, proper error handling.
