# QA Plan: Park/Unpark Core Data Model

## Test Strategy

All testing via Rust unit tests in `crates/sdlc-core/src/milestone.rs` and integration tests via `cargo test`.

## Test Cases

### TC1: Park/Unpark round-trip
- Create milestone, call `park()`, verify `parked_at.is_some()`
- Call `unpark()`, verify `parked_at.is_none()`
- Save and reload, verify `parked_at` persists through YAML round-trip

### TC2: compute_status priority — Parked overrides Active
- Create milestone with features in non-released phases
- Call `park()`, verify `compute_status()` returns `Parked` (not `Active`)

### TC3: compute_status priority — Parked overrides Verifying
- Create milestone with all features released (would normally be `Verifying`)
- Call `park()`, verify `compute_status()` returns `Parked` (not `Verifying`)

### TC4: compute_status priority — Skipped overrides Parked
- Create milestone, call both `park()` and `skip()`
- Verify `compute_status()` returns `Skipped` (skipped wins)

### TC5: compute_status priority — Released overrides Parked
- Create milestone, call both `park()` and `release()`
- Verify `compute_status()` returns `Released` (released wins)

### TC6: Backward compatibility — existing YAML without parked_at
- Deserialize milestone YAML that has no `parked_at` field
- Verify it deserializes successfully with `parked_at = None`
- Verify `compute_status()` returns same result as before

### TC7: Parallel work — parked milestones excluded
- Verify `select_parallel_work()` skips milestones with `Parked` status

### TC8: Display trait
- Verify `MilestoneStatus::Parked.to_string()` returns `"parked"`

## Pass Criteria

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no warnings
- No existing tests broken by the additive change
