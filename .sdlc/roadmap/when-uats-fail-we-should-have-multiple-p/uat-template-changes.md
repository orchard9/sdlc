# UAT Template Changes — Detailed Diff

## Current Step 5 (verdict and state flip)
```
On Failed: do NOT call milestone complete. Call the fail endpoint...
The milestone stays in Verifying. Fix the feature tasks, then re-run this command.
```

## Current Step 6 (final report) 
```
| Failed | Verifying (unchanged) | /sdlc-run <first-blocking-feature-slug> — fix, then re-run /sdlc-milestone-uat <slug> |
```

## Proposed: Replace Step 5-6 with Steps 5-8

### Step 5 — Triage failures (NEW)

For each failure, classify:

| Classification | Signal | Example |
|---|---|---|
| **Fixable** | Assertion fails on a value the agent can change; route returns wrong status; missing CSS class | Fix in Pathway 1 |
| **Escalation** | Missing env var; server unreachable; unclear requirement; needs human judgment | Escalation in Pathway 2 |
| **Complex** | Wrong architectural approach; feature design doesnt match reality; multiple interacting failures | Recap in Pathway 3 |

### Step 6 — Pathway 1: Fix and Retry

If ALL failures are classified as **Fixable**:

1. Fix the code (< 3 files, targeted changes only)
2. Rerun: `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
3. Re-parse results
4. If still failing after 2 fix cycles, reclassify remaining failures and fall through to Pathway 2 or 3

### Step 7 — Pathway 2: Escalate

If any failure is classified as **Escalation** and none are **Complex**:

1. Create tasks for fixable items: `sdlc task add <feature> "UAT: ..."`
2. Create escalation for blocking items:
   ```bash
   sdlc escalate create --kind <type> --title "UAT blocker: <description>" \
     --context "<full error context>" --feature <feature-slug>
   ```
3. Call `POST /api/milestone/<slug>/uat/fail`
4. **Next:** `resolve escalation <id>, then /sdlc-milestone-uat <slug>`

### Step 8 — Pathway 3: Recap and Propose

If any failure is classified as **Complex**:

1. Create tasks for anything fixable: `sdlc task add <feature> "UAT: ..."`
2. Run recap:
   - `sdlc status --json` to gather current state
   - Synthesize what was accomplished vs what failed
   - For each complex failure, create a ponder entry:
     ```bash
     sdlc ponder create "<problem-as-question>" --brief "<context from UAT failure>"
     ```
3. Commit completed work: `git add -A && git commit -m "uat: partial progress on <slug>, ponder sessions proposed"`
4. Call `POST /api/milestone/<slug>/uat/fail`
5. **Next:** `/sdlc-ponder <first-ponder-slug>`

### Step 9 — Final report (unchanged structure, updated table)

| Verdict | State after | Next |
|---|---|---|
| Pass | Released | Commit results |
| PassWithTasks | Released | Commit results; `/sdlc-run <feature>` next cycle |
| FixedAndPassed | Released | Commit results (retry succeeded) |
| Escalated | Verifying | Resolve escalation, then `/sdlc-milestone-uat <slug>` |
| Recapped | Verifying | `/sdlc-ponder <slug>` |
