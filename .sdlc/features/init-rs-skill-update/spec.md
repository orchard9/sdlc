# Feature Spec: Update SDLC_MILESTONE_UAT_COMMAND in init.rs

## Summary

Verify and update the `SDLC_MILESTONE_UAT_COMMAND`, `SDLC_MILESTONE_UAT_PLAYBOOK`, and `SDLC_MILESTONE_UAT_SKILL` const strings in `crates/sdlc-cli/src/cmd/init.rs` to include the complete Mode A / Mode B Playwright workflow.

## Problem

The `/sdlc-milestone-uat` command installed in `~/.claude/commands/sdlc-milestone-uat.md` needs to contain a precise, actionable workflow for running Playwright-based acceptance tests. The skill must cover:

- Mode A: detecting an existing spec and running it via `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
- Mode B: no spec → use Playwright MCP browser tools → write spec → run it
- Step 3: write `summary.md` to `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/`
- Step 4: write `uat_results.md` to `.sdlc/milestones/<slug>/uat_results.md`
- Step 5: call `sdlc milestone complete <slug>` on Pass or PassWithTasks

Parallel variants (Gemini/OpenCode playbook, Agents skill) must be consistent with the Claude command.

## Scope

- `crates/sdlc-cli/src/cmd/init.rs`
  - `SDLC_MILESTONE_UAT_COMMAND` — Claude Code slash command (primary, full detail)
  - `SDLC_MILESTONE_UAT_PLAYBOOK` — Gemini/OpenCode variant (concise)
  - `SDLC_MILESTONE_UAT_SKILL` — Agents SKILL.md variant (minimal)
- Build verification: `SDLC_NO_NPM=1 cargo test --all`
- Install verification: `sdlc update` → check `~/.claude/commands/sdlc-milestone-uat.md`

## Acceptance Criteria

1. `SDLC_MILESTONE_UAT_COMMAND` contains Mode A and Mode B with all 5 numbered steps.
2. `SDLC_MILESTONE_UAT_PLAYBOOK` is consistent (concise but covers same steps).
3. `SDLC_MILESTONE_UAT_SKILL` is consistent (minimal but covers same steps).
4. All four `write_user_*` functions correctly reference the updated consts.
5. `SDLC_NO_NPM=1 cargo test --all` passes.
6. `~/.claude/commands/sdlc-milestone-uat.md` reflects the updated content after `sdlc update`.

## Assessment

After reading the current state of `init.rs`, all three consts already contain complete and correct implementations of Mode A / Mode B. This feature is primarily a verification task. If all criteria are already met, tasks T1-T3 can be marked complete with "no changes needed" notes.
