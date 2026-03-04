# Feature Specification: human-uat-backend

## Overview

Add a `mode` field to the `UatRun` struct (discriminating `agent` vs `human` runs) and expose two new REST endpoints that allow a human tester to submit UAT results directly from the dashboard — bypassing the AI Playwright agent.

## Problem Statement

Currently, `UatRun` records are only ever written by the AI agent (`start_milestone_uat`). When a human wants to manually verify a milestone (e.g. on a device the agent can't reach, or during a live demo), there is no structured way to record the result. The only path is to manually edit YAML files.

The companion feature `human-uat-frontend` will add a "Submit manually" button and a result form. This backend feature provides the data model change and the REST endpoints that form submits to.

## Solution

### 1. Add `mode: UatRunMode` to `UatRun`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UatRunMode {
    #[default]
    Agent,
    Human,
}
```

Add to `UatRun`:
```rust
#[serde(default)]
pub mode: UatRunMode,
```

`serde(default)` makes the field backward-compatible: existing `run.yaml` files without a `mode` field deserialize as `Agent`.

### 2. New REST Endpoint: `POST /api/milestone/{slug}/uat/human`

Allows a human to submit milestone UAT results. Creates a `UatRun` with `mode: human`, writes `run.yaml` and `summary.md` to the standard path, updates `uat_results.md`, and emits `MilestoneUatCompleted` SSE.

**Request body:**
```json
{
  "verdict": "pass" | "pass_with_tasks" | "failed",
  "tests_total": 5,
  "tests_passed": 5,
  "tests_failed": 0,
  "notes": "All steps passed on mobile Safari."
}
```

**Response (200):**
```json
{
  "run_id": "20260303-142500-abc",
  "slug": "v28-human-run-uat",
  "status": "submitted"
}
```

**Errors:**
- `404` if milestone does not exist
- `422` if `verdict` is missing or invalid, or if `notes` is empty when verdict is `failed` or `pass_with_tasks`

**Side effects:**
1. Generates a `run_id` in format `YYYYMMDD-HHMMSS-<3 random lowercase letters>` (UTC).
2. Writes `summary.md` to `.sdlc/milestones/{slug}/uat-runs/{run_id}/summary.md`.
3. Calls `save_uat_run` to write `run.yaml`.
4. If `verdict == pass`, calls `sdlc_core::milestone::Milestone::release()` on the milestone to set `released_at`.
5. Emits `SseMessage::MilestoneUatCompleted { slug }`.

**Summary.md format:**
```markdown
# UAT Results — {milestone_slug}

Run ID: {run_id}
Mode: Human (manual)
Verdict: {verdict}
Tests: {tests_passed}/{tests_total} passed

## Notes
{notes}

Submitted: {timestamp UTC ISO-8601}
```

### 3. New REST Endpoint: `POST /api/features/{slug}/human-qa`

Allows a human to submit QA results for a single feature's `run_qa` action. Writes a `qa-results.md` Draft artifact on the feature, then submits it as a draft, ready for the agent to approve via the normal state machine flow.

**Request body:**
```json
{
  "verdict": "pass" | "pass_with_tasks" | "failed",
  "notes": "Tested all 3 scenarios. One edge case needs a task."
}
```

**Response (200):**
```json
{
  "slug": "my-feature",
  "artifact": "qa_results",
  "status": "draft"
}
```

**Errors:**
- `404` if feature does not exist
- `422` if `verdict` is missing or `notes` is empty for non-pass verdicts

**Side effects:**
1. Writes `qa-results.md` to `.sdlc/features/{slug}/qa-results.md` using a fixed template.
2. Calls `sdlc artifact draft {slug} qa_results` logic via `sdlc_core::feature::Feature::draft_artifact`.
3. Emits `SseMessage::FeatureStateChanged { slug }` so the UI refreshes automatically.

**qa-results.md format:**
```markdown
## Verdict
{verdict_display}

## Notes
{notes}

Runner: human (manual)
Completed: {timestamp UTC ISO-8601}
```

Where `verdict_display` is `Pass`, `Pass with Tasks`, or `Fail`.

### 4. SSE Event

`MilestoneUatCompleted` already exists in `state.rs`. No new SSE variant is needed.

## File Locations

| Change | File |
|---|---|
| `UatRunMode` enum + `mode` field on `UatRun` | `crates/sdlc-core/src/milestone.rs` |
| `POST /api/milestone/{slug}/uat/human` handler | `crates/sdlc-server/src/routes/runs.rs` |
| `POST /api/features/{slug}/human-qa` handler | `crates/sdlc-server/src/routes/features.rs` (new function) |
| Route registrations | `crates/sdlc-server/src/lib.rs` |
| `UatRunMode` re-export | `crates/sdlc-core/src/lib.rs` |

## Integration Tests

In `crates/sdlc-server/tests/integration.rs`:

1. **human_uat_submit_pass**: POST to `/api/milestone/{slug}/uat/human` with `verdict: pass` returns 200, creates `run.yaml` with `mode: human`, and milestone `released_at` is set.
2. **human_uat_submit_failed_no_notes**: POST with `verdict: failed` and empty `notes` returns 422.
3. **human_qa_submit_drafts_artifact**: POST to `/api/features/{slug}/human-qa` returns 200, feature `qa_results` artifact is `draft`.
4. **uat_run_mode_backward_compat**: Deserializing existing `run.yaml` without a `mode` field yields `UatRunMode::Agent`.

## Constraints

- No `unwrap()` in library or server code — use `?` and `AppError`/`SdlcError`
- All file writes via `crate::io::atomic_write`
- `run_id` generation uses rand (already in dependencies) or `uuid`
- `serde(default)` on `mode` field ensures backward compatibility with existing `run.yaml` files
- Route handler must use `spawn_blocking` for all `sdlc-core` sync calls

## Acceptance Criteria

1. Existing `UatRun` YAML files without a `mode` field deserialize successfully as `Agent` mode.
2. `POST /api/milestone/{slug}/uat/human` with valid body returns 200, writes `run.yaml` and `summary.md`.
3. `POST /api/milestone/{slug}/uat/human` with `verdict: pass` sets `milestone.released_at`.
4. `POST /api/milestone/{slug}/uat/human` with `verdict: failed` and empty `notes` returns 422.
5. `POST /api/features/{slug}/human-qa` with valid body returns 200, feature qa_results artifact is `draft`.
6. `SDLC_NO_NPM=1 cargo test --all` passes with all new tests green.
7. `cargo clippy --all -- -D warnings` produces zero warnings.
