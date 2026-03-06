# Code Review: Park/Unpark Core Data Model

## Summary

Added `Parked` milestone status as a reversible pause state. Changes span 2 files in `sdlc-core` plus a pre-existing build fix in `sdlc-server`.

## Files Changed

### `crates/sdlc-core/src/milestone.rs`
- Added `Parked` variant to `MilestoneStatus` enum with `Display` returning `"parked"`
- Added `parked_at: Option<DateTime<Utc>>` field to `Milestone` struct with `serde(default, skip_serializing_if = "Option::is_none")`
- Added `parked_at: None` to `Milestone::new()` constructor
- Updated `compute_status()` with priority: Skipped > Released > Parked > Verifying > Active
- Added `park()` method (sets `parked_at` timestamp)
- Added `unpark()` method (clears `parked_at` to `None`)
- Added 7 unit tests: park/unpark round-trip, priority ordering (parked overrides active, parked overrides verifying, skipped overrides parked, released overrides parked), idempotent behavior, backward compatibility, YAML round-trip

### `crates/sdlc-core/src/parallel_work.rs`
- Added `MilestoneStatus::Parked` to the skip arm in `select_parallel_work()` so parked milestones are excluded from parallel work selection

### `crates/sdlc-server/src/error.rs` (pre-existing fix)
- Added missing `SdlcError::PonderMergeError(_)` match arm to restore exhaustive pattern matching

## Review Findings

### F1: Correct priority ordering
The `compute_status()` check order is: skipped_at -> released_at -> parked_at -> feature-derived. This matches the spec (Skipped > Released > Parked > Verifying > Active). **Verified correct.**

### F2: Backward compatibility
The `parked_at` field uses `serde(default, skip_serializing_if = "Option::is_none")` — identical pattern to `skipped_at`, `released_at`, `prepared_at`. Existing YAML files without `parked_at` deserialize with `None`. The `parked_at_backward_compat` test confirms this. **Verified correct.**

### F3: No migration needed
Since the field is additive with `serde(default)`, no schema version bump or migration code is required. **Verified correct.**

### F4: Parallel work exclusion
Parked milestones are now skipped alongside Released and Skipped in `select_parallel_work()`. The match arm is exhaustive. **Verified correct.**

### F5: State CLI filter
`crates/sdlc-cli/src/cmd/state.rs` line 20 filters out `Released | Skipped` milestones from the active list. Parked milestones are NOT filtered here — they will appear in `sdlc state` output as "parked". This is acceptable for the core feature; the CLI feature (`park-milestone-cli-api`) can refine this if needed. **Accepted — deferred to CLI feature.**

### F6: No feature classifier impact
Features inside parked milestones continue to work normally if invoked directly. The classifier is not affected. This matches the design decision from the ponder session. **Verified correct.**

## Verdict

All findings verified or accepted. No blocking issues. Code is minimal, additive, and follows existing patterns exactly.
