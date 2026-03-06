# Tasks: Park/Unpark Core Data Model

## T1: Add Parked variant and parked_at field to milestone.rs

- Add `Parked` to `MilestoneStatus` enum
- Add `"parked"` case to `Display` impl
- Add `parked_at: Option<DateTime<Utc>>` field to `Milestone` struct with `serde(default, skip_serializing_if = "Option::is_none")`
- Initialize `parked_at: None` in `Milestone::new()`
- Add `park()` and `unpark()` methods
- Update `compute_status()` priority: Skipped > Released > Parked > Verifying > Active

## T2: Update parallel_work.rs to skip parked milestones

- Add `MilestoneStatus::Parked` to the `Released | Skipped => continue` match arm in `select_parallel_work()`

## T3: Add unit tests for park/unpark

- `milestone_park_unpark` — park sets timestamp, unpark clears it
- `compute_status_parked_overrides_active` — parked returns Parked with active features
- `compute_status_parked_overrides_verifying` — parked returns Parked with all features released
- `compute_status_skipped_overrides_parked` — skipped+parked returns Skipped

## T4: Verify build and clippy

- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` clean
