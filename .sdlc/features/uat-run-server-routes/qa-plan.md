# QA Plan: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Scope

This QA plan covers the four implementation areas:
1. Rust server route handlers and registration
2. SSE enum variant and serialization
3. Frontend TypeScript types
4. Frontend API client methods

## Static Analysis

### Rust

- [ ] `SDLC_NO_NPM=1 cargo build --all` exits 0 with no warnings
- [ ] `cargo clippy --all -- -D warnings` exits 0
- [ ] No `unwrap()` calls in new code (use `?` and `AppError`)
- [ ] All new functions have doc comments

### TypeScript

- [ ] `cd frontend && npx tsc --noEmit` exits 0

## Unit / Integration Test Checklist

### Route: GET /api/milestones/{slug}/uat-runs

- [ ] Returns HTTP 200 with `[]` when no UAT runs exist for the milestone
- [ ] Returns HTTP 200 with a populated array when runs exist
- [ ] Array is sorted newest-first by `started_at`
- [ ] Each element is a valid serialized `UatRun` object (has `id`, `milestone_slug`, `verdict`, `tests_total`, `tests_passed`, `tests_failed`, `tasks_created`, `summary_path`)
- [ ] Returns HTTP 400 or 500 for an invalid slug (e.g. path traversal characters)

### Route: GET /api/milestones/{slug}/uat-runs/latest

- [ ] Returns HTTP 404 when no UAT runs exist for the milestone
- [ ] Returns HTTP 200 with the most recent `UatRun` when at least one run exists
- [ ] Returns the newest run (not oldest) when multiple runs exist

### SSE: MilestoneUatCompleted

- [ ] `SseMessage::MilestoneUatCompleted { slug }` serializes to `{ "type": "MilestoneUatCompleted", "slug": "<value>" }`
- [ ] Event is emitted when a UAT agent run completes via `start_milestone_uat`
- [ ] Event is NOT emitted when a UAT run is stopped (stop path uses a different code path)

### TypeScript types

- [ ] `UatVerdict` accepts exactly `'pass' | 'pass_with_tasks' | 'failed'`
- [ ] `UatRun` interface has all required fields with correct types
- [ ] `completed_at` is typed as `string | null`
- [ ] `playwright_report_path` is typed as `string | null`

### API client

- [ ] `api.listMilestoneUatRuns(slug)` calls `GET /api/milestones/{slug}/uat-runs` with correct URL encoding
- [ ] `api.getLatestMilestoneUatRun(slug)` calls `GET /api/milestones/{slug}/uat-runs/latest` with correct URL encoding
- [ ] Both are typed with the correct generic return types

## Regression Check

- [ ] All existing milestone routes still work: `list_milestones`, `get_milestone`, `review_milestone`, `create_milestone`, `add_feature_to_milestone`, `reorder_milestone_features`
- [ ] Existing SSE events (`PonderRunCompleted`, `ToolBuildCompleted`, etc.) still serialize correctly
- [ ] `start_milestone_uat` still spawns the agent run correctly (the only change is adding a completion event)

## Manual Smoke Test (optional, requires running server)

```bash
# Start server
cargo run --bin sdlc-server -- serve

# No runs: expect []
curl http://localhost:3000/api/milestones/my-milestone/uat-runs

# No runs: expect 404
curl -i http://localhost:3000/api/milestones/my-milestone/uat-runs/latest

# SSE stream — confirm MilestoneUatCompleted appears after UAT completes
curl -N http://localhost:3000/api/events &
curl -X POST http://localhost:3000/api/milestone/my-milestone/uat
# wait for agent to finish, confirm event appears in stream
```
