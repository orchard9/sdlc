# Design: Park/Unpark Core Data Model

## Overview

Additive changes to `sdlc-core` milestone data model. No migration needed — new field uses `serde(default)`.

## Data Model Changes

### `MilestoneStatus` enum (`milestone.rs`)

```rust
pub enum MilestoneStatus {
    Active,
    Verifying,
    Parked,      // NEW
    Released,
    Skipped,
}
```

`Display` impl adds `"parked"` case.

### `Milestone` struct

New field after `prepared_at`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub parked_at: Option<DateTime<Utc>>,
```

Follows the identical pattern as `skipped_at` and `released_at`.

### `compute_status()` update

```
if skipped_at.is_some()  -> Skipped
if released_at.is_some() -> Released
if parked_at.is_some()   -> Parked      // NEW — inserted here
<feature-derived logic>  -> Verifying | Active
```

### New methods

```rust
pub fn park(&mut self) {
    self.parked_at = Some(Utc::now());
    self.updated_at = Utc::now();
}

pub fn unpark(&mut self) {
    self.parked_at = None;
    self.updated_at = Utc::now();
}
```

### `Milestone::new()` — no change needed

`parked_at` defaults to `None` via `serde(default)`. The `new()` constructor already initializes all `Option` fields to `None`.

## Parallel Work Integration

In `parallel_work.rs`, `select_parallel_work()` matches on milestone status. Add `Parked` to the skip arm:

```rust
MilestoneStatus::Released | MilestoneStatus::Skipped | MilestoneStatus::Parked => continue,
```

## Backward Compatibility

- Existing milestone YAML files have no `parked_at` field. `serde(default)` deserializes to `None`.
- No schema version bump needed — field is purely additive.
- `compute_status()` returns the same results for all existing milestones (since `parked_at` is `None`).

## Files Modified

| File | Change |
|------|--------|
| `crates/sdlc-core/src/milestone.rs` | `Parked` variant, `parked_at` field, `park()`/`unpark()` methods, `compute_status()` priority, `Display` impl, 4 tests |
| `crates/sdlc-core/src/parallel_work.rs` | Add `Parked` to skip arm in `select_parallel_work()` |
