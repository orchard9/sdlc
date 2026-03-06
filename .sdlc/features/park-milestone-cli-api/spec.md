# Spec: Park/Unpark CLI Commands and REST Endpoints

## Problem

Milestones can only be in Active, Verifying, Released, or Skipped states. There is no way to temporarily pause a milestone without permanently skipping it. When priorities shift or a milestone is waiting on external dependencies, users must either leave it Active (cluttering the work queue) or Skip it (losing progress context and requiring re-creation to resume).

## Solution

Add a reversible "Parked" status to the milestone lifecycle. Parked milestones are excluded from parallel work selection and the active horizon, but retain all their features and progress. Unparking restores the milestone to Active status.

## Scope

### Core Data Model (sdlc-core)

1. **`MilestoneStatus::Parked` variant** — new enum variant in `MilestoneStatus`, serialized as `"parked"`.
2. **`parked_at: Option<DateTime<Utc>>` field** on `Milestone` — set when parked, cleared (`None`) when unparked.
3. **`compute_status` priority update** — `Parked` takes priority below `Skipped` and `Released` but above `Verifying` and `Active`. Priority order: `Skipped > Released > Parked > Verifying > Active`.
4. **`park()` method** — sets `parked_at` to now, updates `updated_at`.
5. **`unpark()` method** — clears `parked_at` to `None`, updates `updated_at`.
6. **Serde compatibility** — `parked_at` uses `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing milestones without the field deserialize cleanly.

### CLI (sdlc-cli)

7. **`sdlc milestone park <slug>`** — loads milestone, calls `park()`, saves. Outputs confirmation (text or JSON).
8. **`sdlc milestone unpark <slug>`** — loads milestone, calls `unpark()`, saves. Outputs confirmation (text or JSON).

### REST API (sdlc-server)

9. **`POST /api/milestones/:slug/park`** — parks a milestone. Returns updated milestone JSON with `"status": "parked"`.
10. **`POST /api/milestones/:slug/unpark`** — unparks a milestone. Returns updated milestone JSON with `"status": "active"` (or computed).

### Parallel Work Integration

11. **`select_parallel_work` in `parallel_work.rs`** — the existing filter `MilestoneStatus::Released | MilestoneStatus::Skipped => continue` must also skip `MilestoneStatus::Parked`.

### Milestone List / Info

12. **`milestone list`** — the status column already uses `compute_status`, so parked milestones will show `"parked"` automatically.
13. **`milestone info`** — include `parked_at` in the JSON output (alongside `skipped_at` and `released_at`).
14. **`GET /api/milestones` and `GET /api/milestones/:slug`** — include `parked_at` in the response JSON.

## Out of Scope

- UI changes (covered by `park-milestone-ui`)
- Preventing park while UAT is running (future safeguard)
- Bulk park/unpark operations

## Acceptance Criteria

- `sdlc milestone park v1` sets `parked_at`, status becomes `parked`
- `sdlc milestone unpark v1` clears `parked_at`, status returns to computed (active/verifying)
- Parked milestones are excluded from `sdlc parallel-work` output
- REST endpoints return correct status after park/unpark
- Existing milestones without `parked_at` field load without error
- Unit tests cover `compute_status` priority: Skipped > Released > Parked > Verifying > Active
