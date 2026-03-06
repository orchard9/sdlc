# Code Review: Park/Unpark CLI Commands and REST Endpoints

## Files Changed

### sdlc-core
- `crates/sdlc-core/src/milestone.rs` — Added `Parked` variant to `MilestoneStatus`, `parked_at` field to `Milestone`, `park()`/`unpark()` methods, updated `compute_status` priority, added 7 unit tests
- `crates/sdlc-core/src/parallel_work.rs` — `Parked` already in skip filter (pre-existing), added 2 unit tests for parked milestone exclusion

### sdlc-cli
- `crates/sdlc-cli/src/cmd/milestone.rs` — Added `Park`/`Unpark` subcommands, `park()`/`unpark()` handler functions, wired into `run()` match

### sdlc-server
- `crates/sdlc-server/src/routes/milestones.rs` — Added `park_milestone`/`unpark_milestone` handlers, added `parked_at` to `get_milestone` response
- `crates/sdlc-server/src/lib.rs` — Registered `/api/milestones/{slug}/park` and `/api/milestones/{slug}/unpark` routes

### Documentation
- `crates/sdlc-cli/src/cmd/init/templates.rs` — Added park/unpark to GUIDANCE_MD_CONTENT CLI reference table
- `.sdlc/guidance.md` — Added park/unpark to live CLI reference table

## Review Findings

### Correctness
- `compute_status` priority is correct: Skipped > Released > Parked > Verifying > Active
- `parked_at` uses `#[serde(default, skip_serializing_if = "Option::is_none")]` — backward compatible
- `park()` and `unpark()` are idempotent — no error on double-park or double-unpark
- `unpark()` handler computes status from features for accurate JSON response

### Test Coverage
- 7 new unit tests in milestone.rs covering park/unpark, priority ordering, idempotency, backward compat, round-trip
- 2 new unit tests in parallel_work.rs for parked exclusion and unpark restoration
- Full test suite passes: 0 failures

### Code Quality
- Follows established patterns (skip/cancel pattern for CLI, spawn_blocking for server routes)
- No `unwrap()` in library/server code
- JSON output consistent with existing milestone endpoints

### No Issues Found
All changes are clean, consistent with codebase conventions, and fully tested.
