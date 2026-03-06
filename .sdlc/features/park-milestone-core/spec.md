# Park/Unpark Core Data Model

## Problem

The project has 17+ non-released milestones, but only 2-3 are actively being worked. The rest clutter the dashboard, Horizon zone, and agent parallel-work selection. There is no way to distinguish "active and in-flight" from "active but not right now" — `Skipped` is permanent cancellation, not a reversible pause.

## Solution

Add a `Parked` status to milestones — a reversible "not now" state that removes milestones from active consideration without cancelling them.

## Scope (core data model only)

This feature covers the `sdlc-core` library changes. CLI commands, REST endpoints, and UI are separate features in the v32-park-milestones milestone.

### Changes to `crates/sdlc-core/src/milestone.rs`

1. **`MilestoneStatus::Parked` variant** — new enum variant with `Display` impl returning `"parked"`.

2. **`parked_at: Option<DateTime<Utc>>` field on `Milestone`** — same serde treatment as `skipped_at`: `#[serde(default, skip_serializing_if = "Option::is_none")]`. Additive field, no migration needed — existing YAML files deserialize with `None`.

3. **`compute_status()` priority update** — insert `Parked` between `Released` and `Verifying`:
   ```
   Skipped > Released > Parked > Verifying > Active
   ```
   When `parked_at.is_some()`, return `MilestoneStatus::Parked` before checking feature phases.

4. **`park()` method** — sets `parked_at = Some(Utc::now())`, updates `updated_at`.

5. **`unpark()` method** — sets `parked_at = None`, updates `updated_at`. Status reverts to whatever `compute_status()` would derive (Active or Verifying).

### Changes to `crates/sdlc-core/src/parallel_work.rs`

6. **Skip parked milestones** — add `MilestoneStatus::Parked` to the `Released | Skipped => continue` match arm so parked milestones are never selected for parallel work.

### Tests

7. **Unit tests** (4, mirroring existing skip/release patterns):
   - `milestone_park_unpark` — park sets `parked_at`, unpark clears it
   - `compute_status_parked_overrides_active` — parked milestone returns `Parked` even with active features
   - `compute_status_parked_overrides_verifying` — parked milestone returns `Parked` even when all features released
   - `compute_status_skipped_overrides_parked` — skipped+parked returns `Skipped` (skipped wins)

## What we are NOT building

- Park reason field
- Park/unpark history tracking
- Feature-level parking
- Bulk park operations
- Any change to feature classifier behavior

## Acceptance Criteria

- `Parked` variant exists in `MilestoneStatus` and serializes as `"parked"`
- `parked_at` field round-trips through YAML serialization
- `compute_status()` returns `Parked` when `parked_at` is set, with correct priority ordering
- `park()` sets the timestamp, `unpark()` clears it
- Parked milestones are excluded from `select_parallel_work()`
- All existing tests continue to pass (backward-compatible, additive change)
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` clean
