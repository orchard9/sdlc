# Enterprise UAT Strategy

## Executive Summary

The sdlc project has a well-structured, agent-driven UAT system — acceptance tests live as Markdown checklists, results are written to git, and the `sdlc-milestone-uat` skill coordinates execution. What it lacks is a **test execution layer with real repeatability**: today's UAT is only as good as the agent's ability to improvise, and there's no mechanical way to replay a passing scenario to confirm nothing regressed.

This document proposes a three-tier UAT architecture that layers deterministic Playwright automation beneath the existing agent-driven workflow, uses the Claude Chrome MCP for visual/exploratory evidence, and introduces the Playwright MCP for token-efficient automated browser control — all with artifacts that commit to `.sdlc/` alongside every other truth in this project.

---

## Current State

### What exists

| Layer | State |
|---|---|
| Acceptance test format | Markdown checklists in `.sdlc/milestones/<slug>/acceptance_test.md` |
| UAT executor | `sdlc-milestone-uat` Claude skill — runs each checklist item as an agent |
| Results storage | `uat_results.md` written by agent, committed to git |
| Server endpoint | `POST /api/milestone/{slug}/uat` → `spawn_agent_run` |
| Frontend | `startMilestoneUat()` / `stopMilestoneUat()` — no results display yet |
| Automated tests | None — zero Playwright, Vitest, or e2e in `frontend/` |
| Regression safety | None — each run is stateless and ad hoc |

### What's missing

1. **Repeatability** — there is no test you can re-run to confirm nothing broke
2. **Regression coverage** — shipping a new milestone could silently break a prior one
3. **Artifact richness** — `uat_results.md` is text; there are no screenshots, traces, or videos
4. **Visual evidence** — GIF-level documentation for stakeholder review
5. **CI gate** — no automated check on pull requests or deploys
6. **History** — test results don't accumulate; the last run overwrites the previous one

---

## The Three-Tier Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ TIER 3 — Exploratory & Visual Layer                             │
│   Claude Chrome MCP                                             │
│   • Authenticated browser (real user session)                   │
│   • GIF recordings as visual proof artifacts                    │
│   • Console/network inspection during exploratory UAT           │
│   • First-pass discovery → feeds Tier 2 spec authoring          │
└─────────────────────────────────────────────────────────────────┘
                              ↓ generates specs
┌─────────────────────────────────────────────────────────────────┐
│ TIER 2 — Automated Regression Layer                             │
│   Playwright MCP + @playwright/test                             │
│   • Deterministic, replayable .spec.ts files                    │
│   • Accessibility-tree-based (token-efficient, not screenshot)  │
│   • Traces, videos, HTML reports as .sdlc/ artifacts            │
│   • Runs on every milestone UAT trigger, every PR               │
└─────────────────────────────────────────────────────────────────┘
                              ↓ results feed
┌─────────────────────────────────────────────────────────────────┐
│ TIER 1 — Agent Synthesis Layer (existing)                       │
│   sdlc-milestone-uat skill                                      │
│   • Reads Playwright JSON results + acceptance checklist        │
│   • Interprets failures, creates tasks, writes uat_results.md  │
│   • Milestone complete / halt decisions remain agent-driven     │
└─────────────────────────────────────────────────────────────────┘
```

Each tier is independently valuable. Tier 2 runs without Tier 3. Tier 1 upgrades cleanly by reading mechanical results instead of improvising.

---

## Tier 3: Claude Chrome MCP — Exploratory Layer

### Role

The Chrome MCP is **not** the repeatable test runner. It is the exploratory instrument used to:

- Perform a first human-quality walkthrough of a new milestone
- Generate GIF recordings as visual proof for async review
- Discover edge cases and real-world friction the acceptance test missed
- Author Playwright specs from observed interactions (see Tier 2)

### When to invoke

| Trigger | Action |
|---|---|
| New milestone reaches `Verifying` for the first time | Chrome MCP exploratory run → GIF artifact → spec authoring |
| Regression investigated after Playwright failure | Chrome MCP deep dive → GIF + console log artifact |
| Stakeholder review requested | Chrome MCP run → GIF exported to `.sdlc/milestones/<slug>/evidence/` |

### Artifact output

```
.sdlc/milestones/<slug>/evidence/
  <date>-exploratory-<session>.gif    # GIF of Chrome MCP walkthrough
  <date>-exploratory-<session>.md     # Observations: console errors, friction, gaps
```

GIFs are committed to git-lfs (or linked via external store if too large). The `.md` observation file is always committed.

### Limitations

- Cannot run headlessly — requires a live Chrome window
- Cannot parallelize
- Not a test oracle — it doesn't produce pass/fail verdicts
- Cannot replay its own session — only documents it

---

## Tier 2: Playwright — Regression Layer

### Toolchain

| Component | Role |
|---|---|
| `@playwright/test` | Test runner, assertion library |
| `@microsoft/playwright-mcp` | MCP server — Claude controls Playwright without screenshots |
| Playwright codegen | Interactive recording of initial spec (supplements Chrome MCP exploration) |
| `playwright-report/` | HTML artifact per run |
| Trace viewer | DOM snapshots + timeline per failed test |

### Why Playwright MCP (not just `@playwright/test` directly)

The Playwright MCP exposes browser automation via the accessibility tree rather than pixel coordinates or screenshots. This matters for sdlc because:

- **Token efficiency**: ~27,000 tokens per task vs ~114,000 for screenshot+vision — 4x cheaper
- **Determinism**: Semantic selectors (`getByRole('button', {name: 'Start UAT'})`) survive minor UI reshuffles
- **Claude integration**: The `sdlc-milestone-uat` skill can invoke Playwright MCP tools directly when running an agent run via `spawn_agent_run`
- **Self-healing potential**: When a selector breaks, the LLM can identify the correct element from the accessibility tree without a human

### Playwright MCP setup

```bash
# Registered in Claude Code's MCP config
claude mcp add playwright npx '@playwright/mcp@latest'
```

Add to `~/.claude.json` or project `.mcp.json`. The sdlc server's `spawn_agent_run` calls receive `allowed_tools` — Playwright MCP tools can be included here for UAT runs.

### Spec location and format

```
frontend/e2e/
  milestones/
    <slug>.spec.ts        # One spec per milestone acceptance test
  shared/
    auth.setup.ts         # Authentication state setup (once per suite)
    fixtures.ts           # Shared page objects
```

**Example generated spec** (from `acceptance_test.md` checklist):

```typescript
import { test, expect } from '@playwright/test';

// Generated from: .sdlc/milestones/v01-directive-core/acceptance_test.md
// Generated: 2026-02-28 via sdlc-milestone-uat exploration session

test.describe('v01-directive-core', () => {
  test('sdlc next emits a complete directive with action type', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Features' }).click();
    const featureSlug = 'directive-richness';
    await page.getByText(featureSlug).click();
    await expect(page.getByTestId('next-action')).toBeVisible();
    await expect(page.getByTestId('next-action')).toContainText('implement_task');
  });

  test('gate hints appear in directive output when gates are configured', async ({ page }) => {
    await page.goto(`/features/gate-hint-format`);
    await expect(page.getByTestId('directive-panel')).toContainText('gate');
  });
});
```

### Playwright configuration

```typescript
// frontend/playwright.config.ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 4 : 2,

  use: {
    baseURL: process.env.SDLC_BASE_URL ?? 'http://localhost:8080',
    trace: 'on-first-retry',       // Full DOM snapshot on first retry
    screenshot: 'only-on-failure', // Screenshots attached to failed tests
    video: 'retain-on-failure',    // Video retained for failed tests
  },

  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'playwright-results.json' }],
  ],

  // Start sdlc server before tests, tear down after
  webServer: {
    command: 'cargo run --bin sdlc-server',
    url: 'http://localhost:8080/api/health',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
```

### Artifact storage per milestone run

```
.sdlc/milestones/<slug>/uat-runs/
  <date>-<run-id>/
    results.json             # Playwright JSON report (machine-readable)
    report/                  # Full HTML report (self-contained)
    traces/                  # Per-test .zip trace files (on-first-retry)
    screenshots/             # Per-failed-test screenshots
    summary.md               # Agent-authored: what passed, what failed, tasks created
```

The `summary.md` is the new `uat_results.md` — structurally identical but backed by machine evidence.

### Artifact retention policy

| Artifact type | Retention | Rationale |
|---|---|---|
| `results.json` | Forever | Small, auditable, machine-readable |
| `summary.md` | Forever | Human audit trail |
| `report/` (HTML) | 90 days | Interactive debugging, then prune |
| `traces/` | 30 days | Debugging value decays quickly |
| `screenshots/` | 30 days | Same |
| `.gif` (Chrome MCP) | Forever | Visual proof at milestone sign-off |

A `sdlc uat prune --older-than 90d` command can manage this. Until then, git-lfs handles binary artifacts.

---

## Tier 1: Agent Synthesis — Upgraded

### What changes in `sdlc-milestone-uat`

The skill's workflow becomes:

```
1. Load acceptance_test.md                       (unchanged)
2. Run Playwright suite for this milestone        (NEW)
   → npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
   → parse playwright-results.json
3. Cross-reference Playwright results with checklist items
4. For any Playwright failure:
   → inspect trace if available
   → determine: fixable now, or task
5. Write summary.md to .sdlc/milestones/<slug>/uat-runs/<date>/
6. Milestone complete / halt decision               (unchanged)
```

### When no Playwright spec exists yet

The skill runs in **generation mode** before execution mode:

```
1. Load acceptance_test.md
2. Use Chrome MCP to walk through each checklist step
3. Author playwright spec from observed interactions
4. Save to frontend/e2e/milestones/<slug>.spec.ts
5. Run the newly created spec immediately
6. Iterate if tests fail (fix spec, not product, if element targeting is wrong)
7. Only after spec passes on first run → proceed to synthesis step
```

This bootstraps specs organically from the same acceptance test workflow already in use.

---

## Spec Generation: Acceptance Test → Playwright

The `acceptance_test.md` format is already structured. The mapping is direct:

```markdown
# acceptance_test.md
- [ ] The features list shows all active features
- [ ] Clicking a feature opens its detail panel
- [ ] sdlc next command output is visible in the directive tab
- [ ] Approving a spec transitions the feature to Specified phase
```

Each checklist item maps to a `test()` block. The agent reading the accessibility tree can target elements by role and text — no brittle CSS selectors needed.

**Locator strategy (priority order):**

| Priority | Locator | Example |
|---|---|---|
| 1 | `getByRole` | `getByRole('button', { name: 'Approve' })` |
| 2 | `getByText` | `getByText('implement_task')` |
| 3 | `getByTestId` | `getByTestId('phase-badge')` — requires `data-testid` in JSX |
| 4 | `getByLabel` | `getByLabel('Feature slug')` |
| Avoid | CSS / XPath | Fragile, breaks on refactor |

Adding `data-testid` attributes to key UI elements in the React frontend is a one-time investment that pays off in spec stability. This is the only frontend change this strategy requires.

---

## CI Integration

### GitHub Actions workflow

```yaml
# .github/workflows/uat.yml
name: Milestone UAT

on:
  push:
    branches: [main]
  pull_request:
    paths:
      - 'frontend/**'
      - 'crates/**'
      - '.sdlc/milestones/**'

jobs:
  playwright:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: cd frontend && npm ci && npm run build
      - run: npx playwright install --with-deps chromium
      - run: |
          SDLC_NO_NPM=1 cargo build --bin sdlc-server --release
          SDLC_BASE_URL=http://localhost:8080 \
          npx playwright test --reporter=list,html,json
        env:
          CI: true
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: frontend/playwright-report/
          retention-days: 30
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-traces
          path: frontend/test-results/
          retention-days: 7
```

### Local developer workflow

```bash
# Run all e2e tests (starts server automatically via webServer config)
cd frontend && npx playwright test

# Run tests for a specific milestone
npx playwright test e2e/milestones/v01-directive-core.spec.ts

# Record a new spec interactively
npx playwright codegen http://localhost:8080 \
  --output e2e/milestones/new-milestone.spec.ts

# View trace of a failed test
npx playwright show-trace test-results/*/trace.zip

# Debug a specific test
npx playwright test --debug e2e/milestones/v01-directive-core.spec.ts
```

---

## Playwright MCP: Direct Agent Control

For `spawn_agent_run`-based UAT, the Playwright MCP enables the agent to control a browser directly without screenshots, at 4x lower token cost.

### Allowed tools in UAT agent runs

```rust
// In routes/runs.rs when spawning milestone-uat
let tools = vec![
    "Read", "Write", "Bash",
    "mcp__playwright__browser_navigate",
    "mcp__playwright__browser_click",
    "mcp__playwright__browser_type",
    "mcp__playwright__browser_snapshot",   // Accessibility tree — not screenshot
    "mcp__playwright__browser_take_screenshot",
    "mcp__playwright__browser_console_messages",
    "mcp__playwright__browser_wait_for",
];
```

### Two-mode UAT agent prompt

**Mode A — Spec exists:**
```
Run the Playwright test suite for milestone {slug}:
  npx playwright test e2e/milestones/{slug}.spec.ts --reporter=json

Parse playwright-results.json. For each failure:
  1. Use browser_snapshot to inspect the relevant page
  2. Determine: code bug, selector break, or spec gap
  3. If code bug: create task, mark failed
  4. If selector break: fix spec immediately, rerun
  5. If spec gap: update spec, rerun

Write summary.md to .sdlc/milestones/{slug}/uat-runs/{date}-{id}/
```

**Mode B — No spec yet:**
```
No Playwright spec found for milestone {slug}. Generate one:

1. Read acceptance_test.md for the checklist
2. Navigate to http://localhost:8080
3. Walk through each checklist item using browser tools
4. Write frontend/e2e/milestones/{slug}.spec.ts as you go
5. Run the spec: npx playwright test e2e/milestones/{slug}.spec.ts
6. Fix any selector issues until spec passes
7. Proceed to Mode A
```

---

## Data Model: UAT Runs

The current milestone struct has `load_uat_results` / `save_uat_results` writing a single file. The upgraded model supports history:

```rust
// crates/sdlc-core/src/milestone.rs additions

pub struct UatRun {
    pub id: String,               // "20260228-143022-abc"
    pub milestone_slug: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub verdict: UatVerdict,      // Pass | PassWithTasks | Failed
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub playwright_report_path: Option<String>,  // Relative to .sdlc/
    pub tasks_created: Vec<String>,
    pub summary_path: String,     // .sdlc/milestones/{slug}/uat-runs/{id}/summary.md
}

pub enum UatVerdict {
    Pass,
    PassWithTasks,
    Failed,
}

impl Milestone {
    pub fn list_uat_runs(&self, root: &Path) -> Result<Vec<UatRun>>;
    pub fn latest_uat_run(&self, root: &Path) -> Result<Option<UatRun>>;
    pub fn save_uat_run(&self, root: &Path, run: &UatRun) -> Result<()>;
}
```

### Server routes to add

```
GET  /api/milestones/{slug}/uat-runs              → list all UAT run records
GET  /api/milestones/{slug}/uat-runs/latest        → latest run + verdict
GET  /api/milestones/{slug}/uat-runs/{id}/summary  → markdown summary
GET  /api/milestones/{slug}/uat-runs/{id}/report   → serve HTML Playwright report
```

The milestone detail page can then show a UAT history panel: date, verdict badge, test counts, link to full HTML report.

---

## Frontend: UAT Panel

The `MilestoneDetail` page gets a new `UatHistoryPanel` component:

```
┌─────────────────────────────────────────────────────────┐
│ UAT History                              [Start UAT ▶]  │
├─────────────────────────────────────────────────────────┤
│ 2026-02-28  PASS       12/12  ──── no tasks             │
│ 2026-02-25  PASS+TASKS  9/10  ──── 1 task: #47          │
│ 2026-02-22  FAILED      6/10  ──── 3 tasks created      │
├─────────────────────────────────────────────────────────┤
│ Latest: 2026-02-28 · 12 tests · 0 failures              │
│ [View Report] [View Trace] [View GIF]                   │
└─────────────────────────────────────────────────────────┘
```

**No polling required** — an SSE `MilestoneUatCompleted { slug }` event triggers a refresh of the UAT runs list. The `useSSE` hook already handles this pattern.

---

## Implementation Waves

### Wave 1 — Foundation (Playwright in the project)

- [ ] Add `@playwright/test` to `frontend/package.json`
- [ ] Add `frontend/playwright.config.ts` with `webServer` config pointing at `sdlc-server`
- [ ] Create `frontend/e2e/milestones/` directory structure
- [ ] Add `data-testid` attributes to key React components (phase badge, directive panel, artifact list, approve/reject buttons)
- [ ] Write first spec manually for the most stable milestone
- [ ] Verify `npx playwright test` passes locally

### Wave 2 — Agent integration

- [ ] Register `@microsoft/playwright-mcp` in `.mcp.json`
- [ ] Update `sdlc-milestone-uat` skill to run Playwright suite before synthesis
- [ ] Add Mode B (spec generation) to the skill prompt
- [ ] Update `spawn_agent_run` call in `routes/runs.rs` to include Playwright MCP tools

### Wave 3 — Artifact persistence

- [ ] Add `UatRun` struct and file I/O to `sdlc-core`
- [ ] Add `save_uat_run` to `Milestone` impl
- [ ] Add server routes: `GET /api/milestones/{slug}/uat-runs`
- [ ] Add `SseMessage::MilestoneUatCompleted { slug }` variant
- [ ] Build `UatHistoryPanel` in frontend
- [ ] Wire SSE event to panel refresh

### Wave 4 — CI gate

- [ ] Add `.github/workflows/uat.yml`
- [ ] Add `npx playwright install` step to CI
- [ ] Configure artifact upload with 30-day retention
- [ ] Add branch protection rule: Playwright must pass before merge to main

### Wave 5 — Visual evidence (Chrome MCP GIFs)

- [ ] Add `evidence/` directory to milestone spec
- [ ] Update `sdlc-milestone-uat` skill to optionally record GIF during exploratory pass
- [ ] Add GIF storage note to `UatRun` struct
- [ ] Surface GIF link in `UatHistoryPanel`

---

## Decision Record

### Why Playwright MCP over headless Playwright alone

Headless Playwright run via `npx playwright test` is the regression layer — deterministic, parallelizable, CI-runnable. The Playwright MCP complements it by giving the `sdlc-milestone-uat` agent direct browser control for spec generation and failure investigation, at 4x lower token cost than screenshot-based vision. Both are needed; neither replaces the other.

### Why not Claude Chrome MCP as the test runner

The Chrome MCP operates in an authenticated, user-owned browser. It cannot:
- Run headlessly in CI
- Parallelize
- Produce machine-readable pass/fail verdicts (only visual documentation)
- Be invoked from `spawn_agent_run` (it requires a live browser session)

Chrome MCP remains the right tool for exploratory documentation and GIF evidence. It is not a test harness.

### Why not Cypress or Vitest browser mode

Playwright has the best-in-class trace viewer (DOM snapshots + action timeline), the official MCP server from Microsoft for agent integration, and the strongest adoption trajectory in 2025. The Playwright MCP's accessibility-tree approach aligns directly with how Claude agents reason about UI — by semantic structure, not pixels. Vitest browser mode is for unit/component tests, not milestone acceptance flows.

### Why acceptance tests stay as Markdown

The `.sdlc/milestones/<slug>/acceptance_test.md` format is the source of truth. Playwright specs are derived artifacts — generated from the Markdown by the agent. This means:
- Non-technical stakeholders can write and review acceptance criteria without touching TypeScript
- The spec regenerates if lost
- The human-language checklist remains the canonical statement of "what done means"

Gherkin (`.feature` files) is a reasonable alternative if we ever want stakeholder tooling (Testomat.io, Cucumber) — the migration from Markdown checklists to Gherkin `Scenario:` blocks is mechanical. For now, the existing format is sufficient and already integrated with the state machine.

### Why store UAT run history instead of overwriting

The current `uat_results.md` is overwritten on each run. This loses the ability to:
- Show a regression (milestone was passing, now failing)
- Correlate a failure with a specific commit
- Show audit history for compliance

Run history costs storage but provides traceability. The `results.json` files are small (< 50KB typical); only HTML reports and traces are large, and those have a defined retention window.

---

## Quick-Start for Next Session

```bash
# 1. Add Playwright
cd frontend
npm install --save-dev @playwright/test
npx playwright install chromium

# 2. Register Playwright MCP
claude mcp add playwright npx '@playwright/mcp@latest'

# 3. Run the codegen recorder against a running server
cargo run --bin sdlc-server &
npx playwright codegen http://localhost:8080 \
  --output e2e/milestones/first-milestone.spec.ts

# 4. Run the generated spec
SDLC_BASE_URL=http://localhost:8080 npx playwright test

# 5. View the HTML report
npx playwright show-report
```

---

## References

- [Playwright Test Documentation](https://playwright.dev/docs/intro)
- [Playwright Codegen](https://playwright.dev/docs/codegen)
- [Playwright Trace Viewer](https://playwright.dev/docs/trace-viewer)
- [Playwright MCP (Microsoft)](https://github.com/microsoft/playwright-mcp)
- [Playwright in CI / GitHub Actions](https://playwright.dev/docs/ci-github)
- [Playwright Test Sharding](https://playwright.dev/docs/test-sharding)
- `docs/plan-act-pattern.md` — two-phase agent workflow (UAT follows same pattern)
- `crates/sdlc-server/src/routes/runs.rs` — `spawn_agent_run` reference implementation
- `crates/sdlc-core/src/milestone.rs` — current `load_uat_results` / `save_uat_results`
- `~/.claude/commands/sdlc-milestone-uat.md` — current UAT skill (to be updated in Wave 2)
