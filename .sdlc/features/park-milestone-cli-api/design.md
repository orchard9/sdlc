# Design: Park/Unpark CLI Commands and REST Endpoints

## Data Model Changes (sdlc-core/src/milestone.rs)

### MilestoneStatus Enum

```
pub enum MilestoneStatus {
    Active,
    Verifying,
    Released,
    Skipped,
    Parked,       // <-- new
}
```

Display impl adds `"parked"` mapping.

### Milestone Struct

```
pub struct Milestone {
    // ... existing fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parked_at: Option<DateTime<Utc>>,
}
```

### compute_status Priority

```
if skipped_at.is_some()  -> Skipped
if released_at.is_some() -> Released
if parked_at.is_some()   -> Parked      // <-- new, before Verifying
if all-features-released  -> Verifying
else                      -> Active
```

### Methods

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

## CLI Commands (sdlc-cli/src/cmd/milestone.rs)

### `sdlc milestone park <slug>`

Follows the exact pattern of `skip`:
1. Load milestone
2. Call `milestone.park()`
3. Save
4. Output: text `"Milestone '<slug>' parked."` or JSON `{"slug": "<slug>", "status": "parked"}`

### `sdlc milestone unpark <slug>`

1. Load milestone
2. Call `milestone.unpark()`
3. Save
4. Output: text `"Milestone '<slug>' unparked."` or JSON `{"slug": "<slug>", "status": "<computed>"}`

## REST Endpoints (sdlc-server/src/routes/milestones.rs)

### `POST /api/milestones/:slug/park`

Handler: `park_milestone`. Loads milestone, calls `park()`, saves, returns full milestone JSON with computed status.

### `POST /api/milestones/:slug/unpark`

Handler: `unpark_milestone`. Loads milestone, calls `unpark()`, saves, returns full milestone JSON with computed status.

Both follow the `spawn_blocking` pattern used by all milestone routes.

### Response Shape Changes

- `list_milestones` — already uses `compute_status`, no changes needed.
- `get_milestone` — add `parked_at` to the JSON output.

## Route Registration

In `crates/sdlc-server/src/routes/mod.rs` (or wherever milestone routes are mounted):
```
.route("/api/milestones/:slug/park", post(milestones::park_milestone))
.route("/api/milestones/:slug/unpark", post(milestones::unpark_milestone))
```

## Parallel Work (sdlc-core/src/parallel_work.rs)

Change the status filter:
```rust
MilestoneStatus::Released | MilestoneStatus::Skipped | MilestoneStatus::Parked => continue,
```

## Backward Compatibility

- `parked_at` uses `#[serde(default)]` — existing milestone YAML files without the field deserialize to `None` without error.
- No schema version bump needed; the field is additive.

## Error Handling

- Parking an already-parked milestone is idempotent (overwrites `parked_at`, no error).
- Unparking a non-parked milestone is idempotent (clears `parked_at` which is already `None`).
