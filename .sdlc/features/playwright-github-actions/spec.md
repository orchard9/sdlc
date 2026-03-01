# Spec: Playwright CI Gate on PRs

## Overview

Add a GitHub Actions workflow (`.github/workflows/uat.yml`) that runs the full Playwright e2e test suite on every pull request targeting `main`. This provides an automated quality gate ensuring UI regressions are caught before merge.

## Goals

- Every PR touching any part of the codebase (frontend or Rust) must pass the Playwright suite before merge is allowed.
- Failures surface a downloadable `playwright-report/` artifact (HTML report + JSON results) so developers can diagnose without re-running locally.
- Traces from failed retries are preserved as a separate artifact.
- Build times are minimized via Cargo registry and `target/` directory caching.

## Trigger

- `pull_request` targeting `main` — runs on every PR open, reopen, and push to a PR branch.

## What the CI Gate Does

1. **Checkout** the repository.
2. **Install Rust toolchain** — reads `rust-toolchain.toml` (stable channel) via `dtolnay/rust-toolchain@stable`.
3. **Cache Cargo** — caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` keyed on the OS and `Cargo.lock` hash.
4. **Build the `sdlc` binary** — `cargo build --bin sdlc --release`. The Playwright `webServer` command (`sdlc ui start --port 7777 --no-open`) requires this binary on PATH.
5. **Add `./target/release` to PATH** — via `$GITHUB_PATH` so the `sdlc` binary is available to all subsequent steps and to the Playwright `webServer` launcher.
6. **Install Node.js 20** — via `actions/setup-node@v4` with npm cache keyed on `frontend/package-lock.json`.
7. **Install frontend dependencies** — `cd frontend && npm ci`.
8. **Install Playwright browsers** — `npx playwright install --with-deps chromium` (Chromium only; matches `playwright.config.ts`).
9. **Run the e2e suite** — `cd frontend && npx playwright test`. The `CI=true` environment variable is set globally so `playwright.config.ts` uses `reuseExistingServer: false` and `retries: 2`.
10. **Upload `playwright-report/`** — always (even on success) so the HTML report is accessible. Retention: 30 days.
11. **Upload `test-results/`** — only on failure, for trace files. Retention: 7 days.

## Environment Variables

| Variable | Value | Purpose |
|---|---|---|
| `CI` | `true` | Enables Playwright CI mode (no server reuse, 2 retries, 4 workers) |

## Success Criteria

- The workflow runs to completion without error on a clean PR.
- A failing Playwright test causes the workflow to exit non-zero and blocks merge.
- The `playwright-report/` artifact is always uploaded and contains `index.html` and `results.json`.
- Build time for a warm cache run is under 5 minutes.

## Non-Goals

- Running tests on push to `main` (post-merge) — the release workflow handles that path.
- Testing on browsers other than Chromium.
- Running Rust unit tests (covered by a separate CI job in `release.yml`).
