# Feature Specification: Update sdlc-milestone-uat Skill with Playwright Support

## Overview

Rewrite the `sdlc-milestone-uat` skill (`SDLC_MILESTONE_UAT_COMMAND`, `SDLC_MILESTONE_UAT_PLAYBOOK`, `SDLC_MILESTONE_UAT_SKILL`) to support two execution modes — Mode A (run an existing Playwright spec) and Mode B (generate a spec from the acceptance test checklist). All three skill variants must communicate this decision tree clearly.

## Problem Statement

The current `sdlc-milestone-uat` skill performs UAT by manually executing checklist steps as a user. This is slow, non-reproducible, and produces inconsistent results across runs. When a Playwright e2e spec exists for a milestone, the skill should run it programmatically. When no spec exists, the skill should author one by exercising the app via Playwright MCP browser tools, then run it. Both modes converge on the same `summary.md` artifact format.

## Scope

### In Scope

1. **Mode detection** — check whether `frontend/e2e/milestones/<slug>.spec.ts` exists; branch accordingly.
2. **Mode A — Existing spec execution**:
   - Run `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
   - Parse `playwright-report/results.json` for pass/fail counts and individual test outcomes
   - Cross-reference failures with `acceptance_test.md` checklist items
   - For each failure: classify as code bug (create task) or selector break (fix spec and rerun)
   - Write `summary.md` to `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/`
3. **Mode B — Spec generation from checklist**:
   - Read `acceptance_test.md` for checklist items
   - Use Playwright MCP browser tools (`mcp__playwright__*`) to navigate each checklist item
   - Author `frontend/e2e/milestones/<slug>.spec.ts` using `getByRole`/`getByTestId` locators as each step is exercised
   - Run the generated spec: `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts`
   - Fix selector issues until the spec passes
   - Proceed to Mode A synthesis (parse results, write summary.md)
4. **Summary artifact** — consistent format written after both modes:
   ```markdown
   # UAT Run — <title>
   **Date:** <ISO>
   **Verdict:** Pass | PassWithTasks | Failed
   **Tests:** <passed>/<total>
   **Tasks created:** <list or none>

   ## Results
   <playwright JSON summary>
   ```
5. **All three skill variants updated**: `SDLC_MILESTONE_UAT_COMMAND` (Claude Code full), `SDLC_MILESTONE_UAT_PLAYBOOK` (Gemini/OpenCode), `SDLC_MILESTONE_UAT_SKILL` (Agents SKILL.md).

### Out of Scope

- Playwright installation / project setup (assumed already available)
- CI integration
- Multi-milestone batch UAT
- Modifying the `sdlc milestone complete` lifecycle step (kept as-is)

## Acceptance Criteria

1. All three skill variants are updated in `crates/sdlc-cli/src/cmd/init.rs`.
2. Mode A is triggered when `frontend/e2e/milestones/<slug>.spec.ts` exists.
3. Mode B is triggered when no spec file exists, generates the spec, and then runs it.
4. Both modes write `summary.md` in the specified format with Verdict, Tests, and Tasks created.
5. `SDLC_NO_NPM=1 cargo build --all` compiles without errors.
6. After `sdlc update`, `~/.claude/commands/sdlc-milestone-uat.md` contains Mode A / Mode B language.

## Technical Notes

- Edit only `crates/sdlc-cli/src/cmd/init.rs` — no Rust logic changes, purely skill instruction text.
- The `allowed-tools` frontmatter for `SDLC_MILESTONE_UAT_COMMAND` must include Playwright MCP tool namespaces so Mode B browser navigation works.
- Summary output path: `.sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/summary.md`
- Existing `uat_results.md` flow for milestone lifecycle (Step 5-6 of original skill) is preserved alongside the new `summary.md`.
