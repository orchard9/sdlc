# Tasks: Park/Unpark CLI Commands and REST Endpoints

## T1: Add Parked variant and parked_at field to milestone core
- Add `Parked` to `MilestoneStatus` enum with Display impl
- Add `parked_at: Option<DateTime<Utc>>` to `Milestone` struct with serde defaults
- Update `compute_status` priority: Skipped > Released > Parked > Verifying > Active
- Add `park()` and `unpark()` methods
- Add unit tests for compute_status with Parked priority

## T2: Add park/unpark CLI subcommands
- Add `Park { slug }` and `Unpark { slug }` variants to `MilestoneSubcommand`
- Implement `park()` and `unpark()` handler functions following the `skip()` pattern
- Wire into the `run()` match

## T3: Add park/unpark REST endpoints
- Add `park_milestone` and `unpark_milestone` handlers in milestones.rs
- Add `parked_at` to `get_milestone` JSON response
- Register routes in the router

## T4: Update parallel_work to skip Parked milestones
- Add `MilestoneStatus::Parked` to the skip filter in `select_parallel_work`
- Add unit test for parked milestone exclusion

## T5: Update guidance.md CLI reference table
- Add `sdlc milestone park <slug>` and `sdlc milestone unpark <slug>` to the command table in guidance.md section 6
