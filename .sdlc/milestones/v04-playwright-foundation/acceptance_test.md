# Acceptance Test: Playwright Foundation

- [ ] `@playwright/test` is present in `frontend/package.json` devDependencies
- [ ] `frontend/playwright.config.ts` exists with `webServer` config pointing at `sdlc-server`
- [ ] Trace, screenshot, and video capture are configured (`on-first-retry`, `only-on-failure`, `retain-on-failure`)
- [ ] HTML and JSON reporters are configured in `playwright.config.ts`
- [ ] `frontend/e2e/milestones/` directory exists with at least one `.spec.ts` file
- [ ] `cd frontend && npx playwright test` runs and produces a passing result against a running sdlc-server
- [ ] `npx playwright show-report` opens a self-contained HTML report with test results
- [ ] Key React components have `data-testid` attributes: `phase-badge`, `directive-panel`, `artifact-list`, `approve-button`, `reject-button`, `milestone-status`, `feature-title`
- [ ] `frontend/e2e/` is listed in `.gitignore` exclusions for large binary artifacts (`playwright-report/`, `test-results/`)
