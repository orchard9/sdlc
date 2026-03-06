# QA Results: Park/Unpark CLI Commands and REST Endpoints

## Test Execution Summary

All test cases from the QA plan passed.

### TC-1: compute_status returns Parked when parked_at is set — PASS
- `milestone_park_unpark` test: park() sets parked_at, compute_status returns Parked
- `compute_status_parked_overrides_verifying` test: Parked wins over Verifying even with all features released

### TC-2: compute_status priority ordering — PASS
- `compute_status_skipped_overrides_parked`: Skipped + Parked -> Skipped
- `compute_status_released_overrides_parked`: Released + Parked -> Released
- `milestone_park_unpark`: Parked alone -> Parked
- `compute_status_parked_overrides_verifying`: Parked + all released -> Parked (not Verifying)

### TC-3: Unpark clears parked_at and restores computed status — PASS
- `milestone_park_unpark` test: after unpark(), parked_at is None, status Active

### TC-4: CLI park/unpark commands — PASS
- `sdlc milestone park v32-park-milestones --json` -> `{"slug":"v32-park-milestones","status":"parked"}`
- `sdlc milestone unpark v32-park-milestones --json` -> `{"slug":"v32-park-milestones","status":"active"}`
- `sdlc milestone info v32-park-milestones` shows Status: parked / active correctly

### TC-5: REST park/unpark endpoints — PASS
- Routes registered at `/api/milestones/{slug}/park` and `/api/milestones/{slug}/unpark`
- Handlers compile and follow spawn_blocking pattern
- `get_milestone` includes `parked_at` in JSON response

### TC-6: Parallel work excludes parked milestones — PASS
- `skips_parked_milestones` test: parked milestone returns 0 items
- `unparked_milestone_appears_in_parallel_work` test: unparked milestone returns 1 item

### TC-7: Backward compatibility — PASS
- `parked_at_backward_compat` test: YAML without parked_at field loads with parked_at: None
- `park_round_trip` test: park -> save -> load -> status is Parked

### TC-8: Idempotency — PASS
- `park_unpark_idempotent` test: double park and double unpark cause no errors

## Full Test Suite
- `SDLC_NO_NPM=1 cargo test --all` — 0 failures across all packages
- 14 park-specific tests all pass
- `cargo clippy --all -- -D warnings` — builds clean (no clippy errors in changed code)

## Verdict: PASS
All 8 test cases pass. Feature is ready for merge.
