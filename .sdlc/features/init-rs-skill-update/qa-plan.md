# QA Plan: Update SDLC_MILESTONE_UAT_COMMAND in init.rs

## Scope

Verify that the three UAT skill consts in `init.rs` are complete and that `sdlc update` installs them correctly. No runtime behavior changed — this is a content-only verification.

## Test Cases

### QA-1: Build passes
- Run `SDLC_NO_NPM=1 cargo test --all`
- Expected: all tests pass, no compilation errors

### QA-2: Clippy passes
- Run `cargo clippy --all -- -D warnings`
- Expected: no warnings

### QA-3: Content verification — SDLC_MILESTONE_UAT_COMMAND
Read `~/.claude/commands/sdlc-milestone-uat.md` after `sdlc update`. Verify:
- Contains "Mode A" and "Mode B" sections
- Mode A step shows `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
- Mode B step references `mcp__playwright__navigate` and related MCP tools
- Step 3 shows path `.sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/`
- Step 4 shows path `.sdlc/milestones/<slug>/uat_results.md`
- Step 5 shows `sdlc milestone complete <slug>` for Pass/PassWithTasks

### QA-4: Playbook and skill consistency
- `~/.gemini/commands/sdlc-milestone-uat.toml` — covers same Mode A/B steps
- `~/.agents/skills/sdlc-milestone-uat/SKILL.md` — covers same workflow

## Pass Criteria

All four QA checks pass.
