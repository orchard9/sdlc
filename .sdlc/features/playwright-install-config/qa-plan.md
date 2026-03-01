# QA Plan: playwright-install-config

## Scope

Verify that the Playwright installation and configuration are correct and complete. All checks are static or local — no live browser tests are executed as part of this feature's QA.

## Test Cases

### TC-01: Package dependency present

**Check:** `@playwright/test` appears in `frontend/package.json` under `devDependencies`.

**How:** `grep -A1 '@playwright/test' frontend/package.json`

**Pass:** Version string matches `^1.x.x` pattern.

---

### TC-02: Test scripts present

**Check:** `test:e2e`, `test:e2e:ui`, and `test:e2e:report` scripts exist in `frontend/package.json`.

**How:** `cat frontend/package.json | grep test:e2e`

**Pass:** All three script keys are present with the correct commands.

---

### TC-03: playwright.config.ts TypeScript validity

**Check:** `frontend/playwright.config.ts` compiles without errors.

**How:** `cd frontend && npx tsc --noEmit`

**Pass:** Zero errors, zero warnings.

---

### TC-04: Config contains required settings

**Check:** All required config values are present in `frontend/playwright.config.ts`.

**How:** Read the file and verify each setting:
- `baseURL: 'http://localhost:8080'`
- `webServer.command: 'cargo run --bin sdlc-server'`
- `webServer.url: 'http://localhost:8080/api/health'`
- `webServer.reuseExistingServer: !process.env.CI`
- `trace: 'on-first-retry'`
- `screenshot: 'only-on-failure'`
- `video: 'retain-on-failure'`
- `fullyParallel: true`
- `retries: process.env.CI ? 2 : 0`
- `workers: process.env.CI ? 4 : 2`
- HTML reporter present
- JSON reporter with `playwright-report/results.json` output

**Pass:** All settings are present and correct.

---

### TC-05: E2E directory scaffold exists

**Check:** `frontend/e2e/milestones/.gitkeep` and `frontend/e2e/shared/.gitkeep` exist.

**How:** `ls frontend/e2e/milestones/ && ls frontend/e2e/shared/`

**Pass:** Both `.gitkeep` files are present.

---

### TC-06: .gitignore exclusions present

**Check:** Playwright output directories are excluded from version control.

**How:** `grep -E 'playwright-report|test-results|blob-report' .gitignore`

**Pass:** All three patterns appear in `.gitignore`.

---

## Exit Criteria

All 6 test cases pass. The feature is complete when:
1. TypeScript compiles cleanly with the new config file.
2. All required config settings are present with correct values.
3. Directory scaffold and .gitignore are in place.
