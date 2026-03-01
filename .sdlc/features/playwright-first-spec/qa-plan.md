# QA Plan: playwright-first-spec

## Objective

Verify that `frontend/e2e/milestones/v01-directive-core.spec.ts` is a production-quality Playwright spec that:
1. Compiles without TypeScript errors
2. Runs successfully against a live sdlc-server
3. Tests the correct behaviors from the v01-directive-core acceptance criteria

## Quality Checks

### 1. TypeScript Compilation
- Command: `cd frontend && npx tsc --noEmit`
- Pass: exit code 0, no type errors
- Fail: any type errors reported

### 2. Spec Structure Validation
- Contains `test.describe()` blocks covering: Dashboard, Milestones, Features, Feature Detail
- Uses only approved locators: `getByRole`, `getByTestId`, `getByText`
- No raw CSS selectors (`.class`, `#id`) or XPath expressions
- Imports from `@playwright/test`

### 3. Server Build Check
- Command: `cargo build --bin sdlc-server 2>&1 | tail -20`
- Pass: server builds without errors
- Blocked: document reason and continue with syntax-only verification

### 4. Test Execution
- Command: `cd frontend && SDLC_NO_NPM=1 npx playwright test e2e/milestones/v01-directive-core.spec.ts --timeout=60000`
- Pass: all tests green
- Fail on selector mismatch: fix and rerun
- Fail on real app bug: create task, document in qa-results.md

### 5. Report Generation
- `playwright-report/results.json` is produced
- HTML report is generated at `playwright-report/index.html`

## Pass Criteria

- TypeScript compiles cleanly
- All tests pass OR failures are documented as known app bugs with tasks created
- Results captured in qa-results.md
