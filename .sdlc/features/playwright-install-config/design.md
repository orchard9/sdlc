# Design: playwright-install-config

## Summary

This is a pure configuration and scaffolding feature. No production code changes, no new API routes, no UI modifications. The design covers the exact shape of every file to be created or modified.

## File Changes

### 1. `frontend/package.json` (modified)

Add to `devDependencies`:
```json
"@playwright/test": "^1.52.0"
```

Add to `scripts`:
```json
"test:e2e": "playwright test",
"test:e2e:ui": "playwright test --ui",
"test:e2e:report": "playwright show-report"
```

### 2. `frontend/playwright.config.ts` (new)

Full configuration file using `@playwright/test`'s `defineConfig` helper. The config is structured as:

```
defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 4 : 2,
  reporter: [['html'], ['json', { outputFile: 'playwright-report/results.json' }]],
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  webServer: {
    command: 'cargo run --bin sdlc-server',
    url: 'http://localhost:8080/api/health',
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
    timeout: 120_000,
    cwd: '../',
  },
})
```

Key design decisions:
- `cwd: '../'` — since Playwright runs from `frontend/`, `cargo run` must be invoked from the repo root where `Cargo.toml` lives.
- `timeout: 120_000` — Rust compile + server startup can take up to 2 minutes on first run.
- `stdout/stderr: 'pipe'` — suppress server output from test logs unless Playwright redirects them.
- `testDir: './e2e'` — all test files live under `frontend/e2e/`.

### 3. `frontend/e2e/milestones/.gitkeep` (new)

Empty file. Placeholder for milestone-specific E2E test suites (e.g., `milestones/m1-core-flow.spec.ts`).

### 4. `frontend/e2e/shared/.gitkeep` (new)

Empty file. Placeholder for shared test helpers, fixtures, and page-object models.

### 5. `.gitignore` (modified)

Add under the frontend section:
```
/frontend/playwright-report/
/frontend/test-results/
/frontend/blob-report/
```

`playwright-report/` — HTML report output directory.
`test-results/` — Playwright artifact capture directory (traces, screenshots, videos).
`blob-report/` — Playwright blob reports used for shard merging in CI.

## Directory Structure After Implementation

```
frontend/
├── e2e/
│   ├── milestones/
│   │   └── .gitkeep
│   └── shared/
│       └── .gitkeep
├── playwright.config.ts
├── package.json            (modified)
└── ...

.gitignore                  (modified)
```

## Dependency Version

`@playwright/test` follows Playwright's versioning. As of early 2026, `^1.52.0` is the current stable release. The caret allows patch and minor updates within v1.

## TypeScript Compatibility

`playwright.config.ts` uses ESM-style imports (`import { defineConfig } from '@playwright/test'`). The existing `tsconfig.app.json` targets ES2020+ and `frontend/` uses `"type": "module"` — Playwright's TypeScript runner handles the config file natively without additional tsconfig entries.

## Rollout

No deployment steps. Changes take effect immediately on the next `npm install` in `frontend/`. No CI pipeline changes in scope for this feature.
