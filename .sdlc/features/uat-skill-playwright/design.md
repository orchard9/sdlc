# Design: Update sdlc-milestone-uat Skill with Playwright Support

## Overview

This is a pure skill-text change — no Rust logic changes, no new data structures, no new API endpoints. The entire implementation is editing the three string constants in `crates/sdlc-cli/src/cmd/init.rs`:

- `SDLC_MILESTONE_UAT_COMMAND` — Claude Code full prompt (~200 lines Markdown + frontmatter)
- `SDLC_MILESTONE_UAT_PLAYBOOK` — Gemini/OpenCode concise variant (~40 lines)
- `SDLC_MILESTONE_UAT_SKILL` — Agents SKILL.md minimal variant (~20 lines)

## Decision Tree

```
/sdlc-milestone-uat <slug>
         │
         ▼
  Does frontend/e2e/milestones/<slug>.spec.ts exist?
         │
    ┌────┴────┐
   YES        NO
    │          │
    ▼          ▼
  Mode A     Mode B
  Run spec   Generate spec
    │          │
    │          ▼
    │       Navigate acceptance_test.md
    │       checklist via Playwright MCP
    │       browser tools; write spec file
    │          │
    │          ▼
    │       Run generated spec; fix selectors
    │          │
    └────┬────┘
         ▼
  Parse playwright-report/results.json
  Cross-reference failures with checklist
  Classify: code bug → task | selector break → fix spec + rerun
         │
         ▼
  Write summary.md to .sdlc/milestones/<slug>/uat-runs/<date>-<id>/
  Write uat_results.md to .sdlc/milestones/<slug>/
  Call sdlc milestone complete (if Pass or PassWithTasks)
```

## Mode A Detail — Existing Spec

1. Verify `frontend/e2e/milestones/<slug>.spec.ts` exists.
2. Run: `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
3. Read `playwright-report/results.json`.
4. For each failed test:
   - Match test title against acceptance_test.md checklist items.
   - If error message mentions element not found / selector / locator → classify as selector break → fix spec + rerun once.
   - Otherwise → classify as code bug → `sdlc task add <feature-slug> "<failure description>"`.
5. Aggregate: passed, failed, tasks created.
6. Write `summary.md` and `uat_results.md`.
7. If verdict is Pass or PassWithTasks: `sdlc milestone complete <slug>`.

## Mode B Detail — Spec Generation

1. Read `acceptance_test.md` content.
2. Parse checklist items (`- [ ]` lines).
3. For each checklist item:
   a. Use Playwright MCP browser tools (`mcp__playwright__navigate`, `mcp__playwright__click`, etc.) to execute the step.
   b. Identify the DOM elements exercised (prefer `getByRole` > `getByTestId` > CSS).
   c. Append a `test('...')` block to the accumulating spec file.
4. Write complete `frontend/e2e/milestones/<slug>.spec.ts`.
5. Run: `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts`
6. If failures: examine error, fix selectors in spec file, rerun. Repeat up to 3 times.
7. Proceed to Mode A synthesis (parse results.json, write summary.md).

## Summary Artifact Format

Path: `.sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/summary.md`

```markdown
# UAT Run — <title>
**Date:** <ISO>
**Verdict:** Pass | PassWithTasks | Failed
**Tests:** <passed>/<total>
**Tasks created:** <list or "none">

## Results
<playwright JSON summary — suite title, counts, duration>

## Failures
<per-failure: test name, classification, task id or fix applied>
```

## Skill Variants Summary

| Variant | Constant | Lines | Format |
|---|---|---|---|
| Claude Code | `SDLC_MILESTONE_UAT_COMMAND` | ~200 | Markdown + YAML frontmatter |
| Gemini/OpenCode | `SDLC_MILESTONE_UAT_PLAYBOOK` | ~50 | Markdown (concise) |
| Agents | `SDLC_MILESTONE_UAT_SKILL` | ~25 | SKILL.md minimal |

All three must mention:
- Mode A / Mode B decision condition
- Playwright test execution step
- Summary.md write step
- Verdict enumeration (Pass, PassWithTasks, Failed)

## Frontmatter Change

`SDLC_MILESTONE_UAT_COMMAND` `allowed-tools` must include Playwright MCP tools for Mode B browser navigation:

```yaml
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, mcp__playwright__navigate, mcp__playwright__click, mcp__playwright__fill, mcp__playwright__screenshot, mcp__playwright__evaluate
```
