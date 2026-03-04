# QA Plan: human-uat-backend

## Automated Tests (cargo test)

Run: `SDLC_NO_NPM=1 cargo test --all`

### Core unit test (sdlc-core)

- [ ] `uat_run_mode_backward_compat` — YAML without `mode` field → `UatRunMode::Agent`

### Integration tests (sdlc-server/tests/integration.rs)

- [ ] `human_uat_submit_pass` — POST `/api/milestone/{slug}/uat/human` with `verdict: pass`, valid notes → 200 response; `run.yaml` written with `mode: human`; milestone `released_at` is set
- [ ] `human_uat_submit_pass_with_tasks_empty_notes` — POST with `verdict: pass_with_tasks`, `notes: ""` → 422
- [ ] `human_uat_submit_failed_empty_notes` — POST with `verdict: failed`, `notes: ""` → 422
- [ ] `human_qa_submit_drafts_artifact` — POST `/api/features/{slug}/human-qa` with valid body → 200; feature `qa_results` artifact is in `draft` state

## Static Analysis

- [ ] `cargo clippy --all -- -D warnings` — zero warnings
- [ ] No `unwrap()` calls in non-test code (grep check: `grep -rn "\.unwrap()" crates/ --include="*.rs" | grep -v "#\[test\]" | grep -v "\.unwrap_or"` should produce no matches in new code)

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` — clean build, no errors

## Backward Compatibility Check

- [ ] Existing `run.yaml` files in `.sdlc/milestones/*/uat-runs/*/run.yaml` still load without error (no `mode` field required)
- [ ] `SDLC_NO_NPM=1 cargo test -p sdlc-core uat_run` — all existing UatRun tests still pass

## Acceptance Criteria Verification

| Criterion | Test |
|---|---|
| Existing run.yaml without `mode` → Agent | `uat_run_mode_backward_compat` |
| POST `/uat/human` valid → 200, run.yaml with mode=human | `human_uat_submit_pass` |
| POST `/uat/human` verdict=pass → milestone released | `human_uat_submit_pass` (assert released_at) |
| POST `/uat/human` failed + empty notes → 422 | `human_uat_submit_failed_empty_notes` |
| POST `/human-qa` valid → 200, qa_results=draft | `human_qa_submit_drafts_artifact` |
| All tests green | `SDLC_NO_NPM=1 cargo test --all` |
| Zero clippy warnings | `cargo clippy --all -- -D warnings` |
