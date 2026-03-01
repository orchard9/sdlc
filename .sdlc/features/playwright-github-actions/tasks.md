# Tasks: Playwright CI Gate on PRs

## Task List

- [ ] Create `.github/workflows/uat.yml` with the Playwright CI workflow
  - Trigger: `pull_request` targeting `main`
  - Job: `playwright` on `ubuntu-22.04`
  - Steps: checkout, Rust toolchain, Cargo cache, build sdlc binary, add to PATH, Node.js 20, npm ci, playwright install, playwright test, upload reports
  - Global env: `CI=true`
