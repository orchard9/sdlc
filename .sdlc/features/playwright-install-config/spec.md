# Feature Specification: playwright-install-config

## Overview

Install `@playwright/test` as a dev dependency in the frontend package and configure it for end-to-end testing of the `sdlc-server` HTTP API and React UI. This feature establishes the Playwright infrastructure that all subsequent E2E test suites will depend on.

## Problem Statement

The `sdlc` frontend currently has no end-to-end test infrastructure. Developers have no way to write, run, or report on browser-level tests that exercise the full stack (React UI + sdlc-server). Without a configured test runner, E2E coverage is impossible and regressions in routes, UI behavior, and server-client integration go undetected.

## Goals

1. Add `@playwright/test` to `frontend/package.json` devDependencies at a recent stable version.
2. Create `frontend/playwright.config.ts` with production-grade configuration covering:
   - `webServer` that starts `sdlc-server` before tests and waits for it to be healthy
   - Artifact capture: trace on first retry, screenshots on failure only, video retained on failure
   - HTML and JSON reporters for CI and local review
   - Parallel execution with CI-aware worker and retry counts
3. Create the `frontend/e2e/` directory scaffold with `milestones/` and `shared/` subdirectories.
4. Exclude Playwright output directories from version control via `.gitignore`.
5. Add `test:e2e`, `test:e2e:ui`, and `test:e2e:report` npm scripts.

## Non-Goals

- Writing any actual E2E test files (tests live in future features).
- Configuring CI/CD pipelines (handled separately).
- Installing Playwright browsers at build time (developers run `npx playwright install`).

## Acceptance Criteria

1. `@playwright/test` appears in `frontend/package.json` under `devDependencies`.
2. `frontend/playwright.config.ts` exists and is valid TypeScript (passes `npx tsc --noEmit`).
3. The config file sets:
   - `baseURL: 'http://localhost:8080'`
   - `webServer.command: 'cargo run --bin sdlc-server'`
   - `webServer.url: 'http://localhost:8080/api/health'`
   - `webServer.reuseExistingServer: !process.env.CI`
   - `trace: 'on-first-retry'`
   - `screenshot: 'only-on-failure'`
   - `video: 'retain-on-failure'`
   - `reporter: [['html'], ['json', { outputFile: 'playwright-report/results.json' }]]`
   - `fullyParallel: true`
   - `retries: process.env.CI ? 2 : 0`
   - `workers: process.env.CI ? 4 : 2`
4. `frontend/e2e/milestones/.gitkeep` and `frontend/e2e/shared/.gitkeep` exist.
5. `.gitignore` excludes `playwright-report/`, `test-results/`, and `blob-report/`.
6. `package.json` scripts include `test:e2e`, `test:e2e:ui`, and `test:e2e:report`.

## Technical Notes

- The `webServer.command` runs from the repository root (`cwd` relative to where `playwright test` is invoked). Since `playwright test` is run from `frontend/`, the command must be `cargo run --bin sdlc-server` with `cwd` or a relative path that resolves to the repo root.
- `reuseExistingServer: !process.env.CI` allows local developers to have the server already running; in CI, Playwright always starts a fresh server.
- The `test:e2e:report` script calls `playwright show-report` which opens the HTML report in a browser.
- Playwright config lives at `frontend/playwright.config.ts` and uses ESM-compatible TypeScript imports.

## Risks

- None high. This is pure configuration and scaffolding — no logic changes to production code.
