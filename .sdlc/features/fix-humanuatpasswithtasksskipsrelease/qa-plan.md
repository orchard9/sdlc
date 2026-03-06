# QA Plan: Fix Human UAT PassWithTasks Skips Release

## Test Cases

1. **PassWithTasks releases milestone** — POST `/api/milestone/{slug}/uat/human` with `verdict: "pass_with_tasks"` and notes → milestone `released_at` is set
2. **Pass still releases milestone** — Existing test: POST with `verdict: "pass"` → milestone released (regression guard)
3. **Failed does NOT release** — POST with `verdict: "failed"` and notes → milestone `released_at` remains None
4. **PassWithTasks requires notes** — POST with `verdict: "pass_with_tasks"` and empty notes → 422 error

## Verification

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no warnings
