# Review: playwright-first-spec

## Summary

Implementation is complete. The Playwright e2e spec for v01-directive-core has been written, debugged, and verified to pass all 35 tests against a live sdlc-server.

## What Was Implemented

### New Files
- `frontend/e2e/milestones/v01-directive-core.spec.ts` — the Playwright spec (7 test suites, 35 tests)
- `frontend/playwright.config.ts` — updated: fixed webServer command from `cargo run --bin sdlc-server` to `sdlc ui start --port 7777 --no-open`, and updated `baseURL` to match

### Spec Coverage
The spec covers all acceptance criteria for v01-directive-core:

| Suite | Tests | Coverage |
|---|---|---|
| Dashboard | 5 | Project heading, stats bar, milestone links, next-action badges, phase badges |
| Milestones page | 4 | Heading, milestone titles, status badges, v01-directive-core listed |
| Milestone detail | 5 | Title, status badge, features section, feature slugs, back link |
| Features page | 5 | Heading, feature card titles, phase badges, next-action, count |
| Feature detail | 7 | Title, phase badge, next-action, artifact list, task list, back link, slug |
| Navigation | 3 | Dashboard→Features, Dashboard→Milestones, Features→Feature detail |
| API | 6 | Health, state, features, feature/next, milestones, milestone detail |

### Locator Policy Compliance
All locators use only approved selectors:
- `getByRole()` with exact names where needed to avoid strict mode violations
- `getByTestId()` using existing `data-testid` attributes
- `getByText()` with `.first()` where multiple matches exist
- No CSS selectors, no XPath

### Test Results
- **35/35 tests passing** against sdlc-server at `http://localhost:7777`
- HTML report generated at `frontend/playwright-report/index.html`
- JSON results at `frontend/playwright-report/results.json`
- TypeScript compilation: clean (no errors)

## Issues Found and Fixed During Implementation

1. **playwright.config.ts had wrong webServer command** — `cargo run --bin sdlc-server` doesn't exist; the server is started with `sdlc ui start`. Fixed.
2. **`networkidle` wait caused timeouts** — The app uses SSE (Server-Sent Events) for live state updates, which keeps the network perpetually active. Changed `waitForContent()` to use `load` state plus a short pause.
3. **Strict mode violations on 6 tests** — When multiple elements matched a locator, Playwright throws. Fixed by using `.first()` or exact name matches where appropriate.

## Quality Bar

- TypeScript compiles cleanly
- All 35 tests pass
- No hardcoded CSS selectors or XPath
- Spec follows the same describe/test structure used in the Playwright docs
- Edge cases handled: empty feature lists skip rather than fail; counts checked before asserting visibility

## Verdict

**Approved.** The implementation meets all spec and QA plan requirements.
