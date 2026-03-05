use crate::cmd::init::registry::CommandDef;

const SDLC_MILESTONE_UAT_COMMAND: &str = r#"---
description: Run the acceptance test for a milestone — Mode A runs an existing Playwright spec, Mode B generates one from the checklist; never pause
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, mcp__playwright__navigate, mcp__playwright__click, mcp__playwright__fill, mcp__playwright__screenshot, mcp__playwright__evaluate, mcp__playwright__select_option, mcp__playwright__hover, mcp__playwright__get_visible_text, mcp__playwright__get_visible_html
---

# sdlc-milestone-uat

Run a milestone's acceptance test using Playwright. Detects whether an e2e spec already exists and routes to Mode A (run it) or Mode B (generate it from the checklist). Both modes parse Playwright results, create tasks for real failures, fix selector breaks, and write `summary.md` plus `uat_results.md`.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Playwright is the user.** UAT means exercising the product through its real UI — not reading code.
- **Never pause.** Decide and act on every failure without asking.
- **Always forward.** Create tasks for code bugs; fix selector breaks inline and rerun.
- **Everything in git.** `summary.md` and `uat_results.md` are committed alongside the code they validate.

## Server lifecycle rule (CRITICAL)

**Never stop, restart, kill, or re-spawn the sdlc server.** When UAT runs from the UI, the agent runs *inside* the server process at `http://localhost:7777`. The server is already up. If `localhost:7777` is unreachable, report it as a hard blocker and stop immediately — never attempt to start or restart the server. Do not call any UAT stop or start endpoints to reset state.

---

## Step 1 — Load the milestone

```bash
ponder milestone info <slug> --json
```

Extract `title`, `vision`, and `acceptance_test` content. If no acceptance test, stop.

## Step 2 — Mode detection

Check whether the e2e spec exists:

```bash
ls frontend/e2e/milestones/<slug>.spec.ts 2>/dev/null && echo "MODE_A" || echo "MODE_B"
```

- **File exists** → proceed to **Mode A**.
- **File absent** → proceed to **Mode B**, then return to Mode A synthesis.

---

## Mode A — Run existing Playwright spec

### A1. Run the spec

```bash
cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
```

Output is written to `playwright-report/results.json`.

### A2. Parse results

Read `frontend/playwright-report/results.json`. Extract:
- Total test count
- Passed count, failed count
- Per-failure: test title, error message, stack trace excerpt

### A3. Cross-reference failures with checklist

For each failed test:
1. Match the test title against the checklist items in `acceptance_test.md`.
2. Examine the error message:
   - **Selector break** — error mentions `locator`, `getByRole`, `getByTestId`, element not found, timeout waiting for element → fix the spec selector and rerun once.
   - **Code bug** — assertion failure, unexpected value, missing route, 4xx/5xx response, etc. → create a task:

```bash
sdlc task add <feature-slug> "UAT: <test title> — <one-line failure description>"
```

Where `<feature-slug>` is the feature in the milestone responsible for the broken behavior. If ambiguous, pick the first feature in the milestone.

### A4. Rerun after selector fixes

If any selector fixes were applied, rerun the spec once:

```bash
cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
```

Re-parse `results.json` for the final counts.

---

## Mode B — Generate spec from checklist

### B1. Parse checklist

Collect every `- [ ]` line from `acceptance_test.md` as an ordered list of steps.

### B2. Navigate each checklist item

For each step, use Playwright MCP browser tools to exercise it:
- `mcp__playwright__navigate` — open URLs
- `mcp__playwright__click` — click buttons, links, tabs
- `mcp__playwright__fill` — type into inputs
- `mcp__playwright__get_visible_text` — read page content for assertions
- `mcp__playwright__screenshot` — capture state for verification

Identify the DOM elements exercised. Prefer locator strategies in this order:
1. `getByRole('button', { name: '...' })` — ARIA roles and accessible names
2. `getByTestId('...')` — `data-testid` attributes
3. `getByText('...')` — visible text (last resort)

### B3. Write the spec file

As you exercise each step, accumulate a `test('...')` block. Write the complete file when all steps are done:

```typescript
// frontend/e2e/milestones/<slug>.spec.ts
import { test, expect } from '@playwright/test';

test.describe('<Milestone Title> — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    // navigate to the app base URL
    await page.goto('/');
  });

  test('<checklist item 1>', async ({ page }) => {
    // steps exercised via Playwright MCP
  });

  // ... one test per checklist item
});
```

Write to `frontend/e2e/milestones/<slug>.spec.ts`.

### B4. Run the generated spec

```bash
cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
```

### B5. Fix selector issues

If failures occur:
1. Read the error for each failing test.
2. Update the locator in the spec file.
3. Rerun.
4. Repeat up to 3 times until the spec passes or remaining failures are genuine code bugs.

### B6. Continue to Mode A synthesis

Once the spec runs (passing or with stable failures), proceed to **Mode A Step A2** to parse results and create tasks.

---

## Step 3 — Write summary.md

Create the run directory and write the summary:

```bash
mkdir -p .sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/
```

Write `.sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/summary.md`:

```markdown
# UAT Run — <milestone-title>
**Date:** <ISO-8601 timestamp>
**Verdict:** Pass | PassWithTasks | Failed
**Tests:** <passed>/<total>
**Tasks created:** <feature>#<id>, ... | none

## Results
Suite: <suite name>
Duration: <ms>ms
Passed: <n> | Failed: <n> | Skipped: <n>

## Failures
| Test | Classification | Resolution |
|---|---|---|
| <test title> | Code bug | Task <feature>#<id> created |
| <test title> | Selector break | Fixed locator — rerun passed |
```

## Step 4 — Write uat_results.md

Write the signed checklist to `.sdlc/milestones/<slug>/uat_results.md`:

```markdown
# UAT Run — <milestone-title>
**Date:** <ISO-8601 timestamp>
**Agent:** <model identifier>
**Verdict:** PASS | PASS WITH TASKS | FAILED

---

- [x] <step text> _(<timestamp>)_
- [x] <step text> _(fixed: <what changed> · <timestamp>)_
- [ ] ~~<step text>~~ _(✗ task <feature>#<id> — <one-line reason>)_

---

**Tasks created:** <feature>#<id>, ...
**<N>/<total> steps passed**
```

## Step 5 — Flip milestone state for passing runs

**Verdict rules (for Pass/PassWithTasks only — failures go to Step 5B):**
- All tests pass → **Pass**
- Some tests fail but only tasks created, none blocking → **PassWithTasks**

**On Pass or PassWithTasks:**

```bash
ponder milestone complete <slug>
```

Then skip to Step 9.

## Step 5B — Triage failures

For each failed test, classify it into exactly one category:

| Classification | Signal | Example |
|---|---|---|
| **Fixable** | Assertion fails on a value the agent can change; route returns wrong status; missing CSS class | Wrong button label, off-by-one count, missing aria attribute |
| **Escalation** | Missing env var; server unreachable; unclear requirement; needs human judgment | `STRIPE_KEY` not set, auth service down, requirement contradicts spec |
| **Complex** | Wrong architectural approach; feature design doesn't match reality; multiple interacting failures | Feature built for old data model, entire flow broken, 4+ failing tests with shared root cause |

Collect classifications before proceeding.

## Step 6 — Pathway 1: Fix and Retry

**If ALL failures are classified as Fixable:**

1. Fix the code (targeted changes — max 3 files, no structural rewrites)
2. Rerun the spec:
   ```bash
   cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
   ```
3. Re-parse `playwright-report/results.json` for updated counts
4. If all pass → verdict is **FixedAndPassed** → proceed to Step 9
5. If still failing after **2 total fix cycles** → reclassify remaining failures and fall through to Pathway 2 or 3

## Step 7 — Pathway 2: Escalate

**If any failure is Escalation AND none are Complex:**

1. Create tasks for any Fixable items:
   ```bash
   sdlc task add <feature-slug> "UAT: <test title> — <one-line description>"
   ```
2. Create an escalation for each Escalation-class failure:
   ```bash
   sdlc escalate create --kind blocker --title "UAT blocker: <description>" \
     --context "<full error context from results.json>" --feature <feature-slug>
   ```
3. Signal milestone failure:
   ```bash
   curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
   ```
4. Proceed to Step 9 with verdict **Escalated**

## Step 8 — Pathway 3: Recap and Propose

**If any failure is Complex:**

1. Create tasks for any Fixable items:
   ```bash
   sdlc task add <feature-slug> "UAT: <test title> — <one-line description>"
   ```
2. Gather project state:
   ```bash
   sdlc status --json
   ```
3. For each Complex failure, synthesize the root cause and create a ponder entry:
   ```bash
   sdlc ponder create "<problem-as-question>" --brief "<context: what failed, why it's architectural, what needs rethinking>"
   ```
4. Commit completed work:
   ```bash
   git add -A && git commit -m "uat: partial progress on <slug>, ponder sessions proposed for complex failures"
   ```
5. Signal milestone failure:
   ```bash
   curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
   ```
6. Proceed to Step 9 with verdict **Recapped**

## Step 9 — Final report

| Verdict | State after | Next |
|---|---|---|
| Pass | `Released` | Commit `summary.md` and `uat_results.md` |
| PassWithTasks | `Released` | Commit results; `/sdlc-run <task-owning-feature-slug>` next cycle |
| FixedAndPassed | `Released` | Commit results (retry succeeded) |
| Escalated | `Verifying` (unchanged) | `**Next:** resolve escalation <id>, then /sdlc-milestone-uat <slug>` |
| Recapped | `Verifying` (unchanged) | `**Next:** /sdlc-ponder <first-ponder-slug>` |

Always end output with exactly one **Next:** line showing the command to run.
"#;

const SDLC_MILESTONE_UAT_PLAYBOOK: &str = r#"# sdlc-milestone-uat

Use this playbook to run a milestone's acceptance test using Playwright — Mode A runs an existing spec, Mode B generates one from the checklist.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`. Extract acceptance_test and title.
2. **Mode detection** — check `frontend/e2e/milestones/<slug>.spec.ts`:
   - **Mode A** (file exists): run `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`. Parse `playwright-report/results.json`.
   - **Mode B** (no file): read acceptance_test.md checklist, navigate each item using Playwright MCP browser tools (`mcp__playwright__navigate`, `mcp__playwright__click`, `mcp__playwright__fill`, etc.), write `frontend/e2e/milestones/<slug>.spec.ts` using `getByRole`/`getByTestId` locators, run the spec, fix selector issues until passing, then proceed to Mode A synthesis.
3. For each Playwright test failure:
   - **Selector break** (element not found, locator timeout) → fix spec locator and rerun once.
   - **Code bug** (assertion failure, bad response) → `sdlc task add <feature-slug> "UAT: <test> — <reason>"`, continue.
4. Write `summary.md` to `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/`:
   - Verdict: Pass | PassWithTasks | FixedAndPassed | Escalated | Recapped
   - Tests: `<passed>/<total>`
   - Tasks created: list or none
   - Results: Playwright JSON summary
5. Write `uat_results.md` to `.sdlc/milestones/<slug>/uat_results.md` (signed checklist).
6. On Pass or PassWithTasks: `sdlc milestone complete <slug>`. On failure — triage each failure:
   - **Fixable**: fix code (max 3 files), rerun (max 2 cycles). If fixed → FixedAndPassed → `sdlc milestone complete <slug>`.
   - **Escalation** (any, no Complex): `sdlc task add` for fixables, `sdlc escalate create` for blockers, call `uat/fail`. **Next:** resolve escalation, then re-run UAT.
   - **Complex** (any): `sdlc task add` for fixables, `sdlc ponder create "<question>"` for each complex failure, `git commit`, call `uat/fail`. **Next:** `/sdlc-ponder <slug>`.

## Key Rules

- Playwright is the user: exercise the real UI, don't read code.
- Selector breaks are spec bugs — fix them; code bugs become tasks.
- Never pause to ask — triage and act on every failure.
- Always forward: fix-and-retry, escalate, or propose — never strand the human.
"#;

const SDLC_MILESTONE_UAT_SKILL: &str = r#"---
name: sdlc-milestone-uat
description: Run the acceptance test for a milestone using Playwright. Mode A runs an existing spec; Mode B generates one from the checklist. Use when validating that a milestone meets its definition of done.
---

# SDLC Milestone UAT Skill

Use this skill to run a milestone's acceptance test via Playwright.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone: `sdlc milestone info <slug> --json`.
2. **Mode A** — if `frontend/e2e/milestones/<slug>.spec.ts` exists: run `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`, parse `playwright-report/results.json`.
3. **Mode B** — if no spec: navigate each `acceptance_test.md` checklist item via Playwright MCP browser tools, write `frontend/e2e/milestones/<slug>.spec.ts` using `getByRole`/`getByTestId` locators, run and fix until passing, then continue to Mode A synthesis.
4. For each failure: selector break → fix spec + rerun; code bug → classify as Fixable / Escalation / Complex.
5. Write `summary.md` to `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/` with Verdict (Pass/PassWithTasks/FixedAndPassed/Escalated/Recapped), test counts, tasks created, and Playwright results.
6. Write `uat_results.md` to `.sdlc/milestones/<slug>/`. On Pass or PassWithTasks: `sdlc milestone complete <slug>`. On failure — triage and act:
   - **All Fixable**: fix code (≤3 files), rerun (max 2 cycles). If fixed → `sdlc milestone complete <slug>`.
   - **Any Escalation**: create tasks + `sdlc escalate create` for blockers + `curl uat/fail`. Next: resolve + re-run.
   - **Any Complex**: create tasks + `sdlc ponder create` for complex failures + `git commit` + `curl uat/fail`. Next: `/sdlc-ponder <slug>`.
"#;

pub static SDLC_MILESTONE_UAT: CommandDef = CommandDef {
    slug: "sdlc-milestone-uat",
    claude_content: SDLC_MILESTONE_UAT_COMMAND,
    gemini_description: "Run the acceptance test for a milestone",
    playbook: SDLC_MILESTONE_UAT_PLAYBOOK,
    opencode_description: "Run the acceptance test for a milestone",
    opencode_hint: "<milestone-slug>",
    skill: SDLC_MILESTONE_UAT_SKILL,
};
