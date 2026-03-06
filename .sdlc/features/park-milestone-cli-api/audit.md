# Security Audit: Park/Unpark CLI Commands and REST Endpoints

## Attack Surface Analysis

### New REST Endpoints
- `POST /api/milestones/:slug/park` — mutates milestone state
- `POST /api/milestones/:slug/unpark` — mutates milestone state

### Findings

#### F1: Authentication — ACCEPTABLE
Both endpoints inherit the existing auth middleware (tunnel auth with token/cookie gate, local bypass). No new auth bypass paths introduced. Same protection level as existing milestone mutation endpoints (create, complete, cancel, skip).

#### F2: Authorization — NOT APPLICABLE
The system has no role-based access control. All authenticated users can perform all operations. This is consistent with the existing design.

#### F3: Input Validation — PASS
The `slug` parameter is validated by `Milestone::load()` via `paths::validate_slug()`, which rejects path traversal characters and invalid slugs. No additional user input is accepted (no request body).

#### F4: State Manipulation — LOW RISK
Park/unpark only set/clear a timestamp field. They do not delete data, modify features, or change artifact state. The operations are fully reversible. Idempotent calls do not produce errors.

#### F5: Data Integrity — PASS
- `parked_at` is written via `milestone.save()` which uses `atomic_write` (write-tmp-then-rename pattern)
- No concurrent write risk beyond what exists for all milestone operations
- Backward-compatible serde defaults ensure no data corruption on load

#### F6: Denial of Service — NOT APPLICABLE
No expensive computations, no external calls, no unbounded loops. Same cost as existing milestone status endpoints.

## Verdict

No security issues found. The feature adds two simple state-mutation endpoints that follow established patterns and inherit existing auth protection.
