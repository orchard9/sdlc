# QA Plan: Playwright CI Gate on PRs

## Verification Approach

This feature is a GitHub Actions workflow file. Verification is structural (YAML linting, schema correctness) and functional (open a PR and confirm the workflow runs to completion).

## Checks

### 1. YAML Validity
- The workflow file parses without errors as valid YAML.
- All `uses:` action references include a pinned version tag (`@v4`).

### 2. Workflow Trigger
- Confirm `on.pull_request.branches` targets `main`.
- Confirm there is no `push` trigger (the UAT workflow is PR-only).

### 3. Job Steps Completeness
Verify the job contains all required steps in order:
- `actions/checkout@v4`
- `dtolnay/rust-toolchain@stable`
- `actions/cache@v4` for Cargo paths
- `cargo build --bin sdlc --release`
- `$GITHUB_WORKSPACE/target/release` appended to `$GITHUB_PATH`
- `actions/setup-node@v4` with `node-version: "20"`
- `npm ci` in `frontend/`
- `npx playwright install --with-deps chromium`
- `npx playwright test` (working-directory: `frontend`)
- `actions/upload-artifact@v4` for `playwright-report/` with `if: always()`
- `actions/upload-artifact@v4` for `test-results/` with `if: failure()`

### 4. Environment Variables
- `CI: "true"` is set at the workflow (or job) `env` block.

### 5. Artifact Retention
- `playwright-report/` retention: 30 days.
- `test-results/` retention: 7 days.

### 6. Functional Test (Manual / CI)
- Open a draft PR against `main` — confirm the `UAT` workflow appears and passes on the GitHub Actions tab.
- Temporarily introduce a failing Playwright test — confirm the workflow fails and the `playwright-report` artifact is still uploaded.
