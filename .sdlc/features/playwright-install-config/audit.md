# Security Audit: playwright-install-config

## Scope

Install `@playwright/test` as a dev dependency and configure the Playwright test runner. No production runtime changes; all additions are test infrastructure only.

## Attack Surface Analysis

### Dependency: @playwright/test

**Publisher:** Microsoft (Playwright team). Well-established, actively maintained project with a strong security track record.

**Install-time scripts:** Playwright's npm package does not run postinstall scripts that execute arbitrary code. Browser binaries require a separate explicit `npx playwright install` command — they are not auto-downloaded on `npm install`.

**Runtime scope:** `devDependencies` only. The package is never bundled into `sdlc-server` or the frontend production build. It is excluded from the build artifact by Vite's production bundler.

**Supply chain risk:** LOW. `@playwright/test ^1.58.2` pins to a well-known, auditable release series. The caret range is standard for tooling dependencies.

---

### webServer configuration

The `webServer.command` starts `cargo run --bin sdlc-server` — the existing application binary. No new network service is introduced. The server runs on `localhost:8080`, the same port it uses for normal development.

**`reuseExistingServer: !process.env.CI`** — in non-CI environments, if a server is already running on 8080, Playwright reuses it. This is the same server developers run locally. No security concern.

**`webServer.cwd: '../'`** — resolves to the repository root. This is necessary for Cargo to find `Cargo.toml`. No path traversal risk since this is a developer-controlled local path.

---

### Output directories

`playwright-report/`, `test-results/`, and `blob-report/` are added to `.gitignore`. These directories may contain:
- Screenshots and videos of application UI during tests
- Network traces

These are local artifacts and are never committed to the repository. No sensitive credentials appear in the test output given the application's architecture (no login forms, no PII input fields in scope for current tests).

---

### Test scripts

The `test:e2e`, `test:e2e:ui`, and `test:e2e:report` npm scripts are developer-facing only. They require local execution with an authenticated shell. No CI/CD pipeline automation changes are in scope.

---

## Findings

| ID | Severity | Finding | Status |
|---|---|---|---|
| A-01 | INFORMATIONAL | `@playwright/test` added as devDependency; not included in production build | ACCEPTED |
| A-02 | INFORMATIONAL | webServer starts existing sdlc-server on localhost only | ACCEPTED |

No HIGH or MEDIUM severity findings.

## Verdict

**APPROVED.** This change introduces no meaningful security surface. All additions are confined to dev-only tooling and local test infrastructure.
