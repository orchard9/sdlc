# Design: playwright-first-spec

## Approach

Write a single Playwright spec file at `frontend/e2e/milestones/v01-directive-core.spec.ts`. The spec uses Playwright's `test.describe()` and `test()` API, imports from `@playwright/test`, and relies exclusively on `getByRole`, `getByTestId`, and `getByText` locators.

The playwright.config.ts already exists (from `playwright-install-config`) and configures:
- `testDir: './e2e'`
- `baseURL: 'http://localhost:8080'`
- `webServer` to auto-start `cargo run --bin sdlc-server` from the project root

## File Structure

```
frontend/
  e2e/
    milestones/
      v01-directive-core.spec.ts   ← new file written by this feature
  playwright.config.ts              ← already exists
```

## Spec Structure

```typescript
import { test, expect } from '@playwright/test'

test.describe('v01-directive-core: Directive output is complete and rich', () => {

  test.describe('Dashboard', () => {
    // Verifies the root page loads with project name and features
  })

  test.describe('Milestones page', () => {
    // Verifies /milestones lists milestones with titles and status badges
  })

  test.describe('Milestone detail', () => {
    // Verifies /milestones/v01-directive-core shows title and features
  })

  test.describe('Features page', () => {
    // Verifies /features lists feature cards
  })

  test.describe('Feature detail', () => {
    // Verifies feature detail shows title, phase badge, next-action, artifact-list, task-list
  })

})
```

## Locator Strategy

| Element | Locator |
|---|---|
| Feature title in card | `page.getByTestId('feature-title').first()` |
| Phase badge | `page.getByTestId('phase-badge').first()` |
| Next action | `page.getByTestId('next-action').first()` |
| Artifact list | `page.getByTestId('artifact-list')` |
| Task list | `page.getByTestId('task-list')` |
| Milestone title (in list) | `page.getByTestId('milestone-title').first()` |
| Milestone status | `page.getByTestId('milestone-status').first()` |
| Navigation links | `page.getByRole('link', { name: '...' })` |
| Headings | `page.getByRole('heading', { name: '...' })` |

## Test Execution

The playwright.config.ts `webServer` block starts `cargo run --bin sdlc-server` automatically. Tests run with:

```bash
cd frontend && SDLC_NO_NPM=1 npx playwright test e2e/milestones/v01-directive-core.spec.ts
```

`SDLC_NO_NPM=1` prevents the server from trying to rebuild the frontend inside the webServer boot.

## Failure Modes and Handling

| Scenario | Handling |
|---|---|
| Server starts, tests pass | Record results in qa-results.md |
| Server starts, selector mismatch | Fix locators, rerun |
| Server fails to build | Document in qa-results.md, release anyway (spec is syntactically correct) |
| Missing data-testid | Check if attr was actually added by react-testid-attributes feature; add task if missing |

## Dependencies

- `playwright-install-config` (released): `@playwright/test` installed, `playwright.config.ts`, `e2e/` structure
- `react-testid-attributes` (released): `data-testid` attributes on key components
