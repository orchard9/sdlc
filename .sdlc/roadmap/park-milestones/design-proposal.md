# Design Proposal: Milestone Parking

## The Primitive

Add `Parked` to `MilestoneStatus`. Stored as `parked_at: Option<DateTime<Utc>>` on the Milestone struct — same pattern as `skipped_at` and `released_at`.

### Semantics
- **Parked**: 'not now, but not cancelled'. Work is paused. Features remain linked. Can be unparked.
- **Skipped**: 'intentionally cancelled'. Terminal. Not coming back.
- **Active**: 'in flight'. Being worked on.

### Priority in `compute_status()`
`Skipped > Released > Parked > Verifying > Active`

A parked milestone with `released_at` is Released (you finished it, then parked it — doesn't make sense, but Released wins). A parked milestone with `skipped_at` is Skipped (skip is permanent, overrides park).

### Unparking
`sdlc milestone unpark <slug>` — clears `parked_at`. Status reverts to computed (Active or Verifying depending on feature state).

## CLI Surface

```
sdlc milestone park <slug>      # set parked_at = now
sdlc milestone unpark <slug>    # clear parked_at
```

## UI Treatment

### MilestonesPage.tsx
Three sections:
1. **Active** — status is Active or Verifying (working milestones)
2. **Parked** — status is Parked (collapsed by default, like Archive)
3. **Archive** — status is Released or Skipped

### HorizonZone.tsx
Filter out parked milestones from the Horizon. They are explicitly not-now.

### Dashboard
Parked milestones do not appear in the main dashboard zone. A small count/link ('3 parked') in the Horizon section is sufficient.

## Data Migration
None needed. `parked_at` is `Option<DateTime<Utc>>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` — existing milestones without the field deserialize as `None` (not parked).

## REST API
- `PATCH /api/milestones/:slug/park` — sets parked_at
- `PATCH /api/milestones/:slug/unpark` — clears parked_at
- `GET /api/state` already returns milestone list — status field will include `parked`
