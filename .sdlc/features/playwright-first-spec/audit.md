# Security Audit: playwright-first-spec

## Scope

This feature adds:
- `frontend/e2e/milestones/v01-directive-core.spec.ts` — Playwright test file
- Updates to `frontend/playwright.config.ts` — test runner configuration

## Security Surface

**Minimal.** This is a pure test artifact. No new code runs in production.

## Analysis

### playwright.config.ts changes
- Changed `webServer.command` from `cargo run --bin sdlc-server` (incorrect) to `sdlc ui start --port 7777 --no-open`
- Changed `baseURL` from `http://localhost:8080` to `http://localhost:7777`
- The `reuseExistingServer: !process.env.CI` setting means tests can attach to a running server — standard Playwright practice
- No credentials, tokens, or secrets are embedded

### v01-directive-core.spec.ts
- Makes HTTP requests to `http://localhost:7777` (local only)
- No external network requests
- No file system writes from test code
- API tests read public state endpoints (`/api/state`, `/api/features`, `/api/milestones`) — no mutations
- No sensitive data asserted on

### Risk Assessment

| Risk | Level | Notes |
|---|---|---|
| Data exposure | None | Tests run against localhost only |
| Credential leakage | None | No secrets in test code |
| Supply chain | None | Only uses `@playwright/test` already installed |
| Code execution | Low | Tests execute the local sdlc server, which is already trusted |
| Production impact | None | Test files never run in production |

## Verdict

**Approved.** No meaningful security surface. This is a test-only addition.
