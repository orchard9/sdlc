# Tasks: Update SDLC_MILESTONE_UAT_COMMAND in init.rs

## Assessment

After reading the current state of `crates/sdlc-cli/src/cmd/init.rs`, all three target consts already contain the complete Mode A / Mode B Playwright workflow:

- `SDLC_MILESTONE_UAT_COMMAND` (lines 2261–2485): full 6-step workflow with Mode A CLI command, Mode B MCP tools, summary.md step, uat_results.md step, and `sdlc milestone complete` on Pass/PassWithTasks.
- `SDLC_MILESTONE_UAT_PLAYBOOK` (lines 2751–2780): consistent concise coverage.
- `SDLC_MILESTONE_UAT_SKILL` (lines 2890–2909): consistent minimal coverage.

No edits to `init.rs` are needed. All tasks reduce to verification.

## Tasks

### T1: Verify SDLC_MILESTONE_UAT_COMMAND — no changes needed
**Status:** Complete (verified by reading init.rs lines 2261–2485)

The const contains:
- Step 1: `sdlc milestone info <slug> --json`
- Step 2: Mode detection with `ls frontend/e2e/milestones/<slug>.spec.ts`
- Mode A: `cd frontend && npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json` + parse `playwright-report/results.json`
- Mode B: Playwright MCP browser tools + write spec + run + fix selectors
- Step 3: `summary.md` to `.sdlc/milestones/<slug>/uat-runs/<YYYY-MM-DD>-<run-id>/`
- Step 4: `uat_results.md` to `.sdlc/milestones/<slug>/uat_results.md`
- Step 5: `sdlc milestone complete <slug>` on Pass or PassWithTasks

### T2: Verify SDLC_MILESTONE_UAT_PLAYBOOK — no changes needed
**Status:** Complete (verified by reading init.rs lines 2751–2780)

Concise 6-step playbook covers Mode A/B, failure triage, summary.md, uat_results.md, and `sdlc milestone complete`.

### T3: Verify SDLC_MILESTONE_UAT_SKILL — no changes needed
**Status:** Complete (verified by reading init.rs lines 2890–2909)

Minimal 6-step workflow covering same steps as Claude command.

### T4: Verify write_user_* function registrations
**Action:** Confirm all four write functions include `sdlc-milestone-uat`:
- `write_user_claude_commands()` → `("sdlc-milestone-uat.md", SDLC_MILESTONE_UAT_COMMAND)` ✓ (line 569)
- `write_user_gemini_commands()` → `SDLC_MILESTONE_UAT_PLAYBOOK` ✓ (line 642)
- `write_user_opencode_commands()` → `SDLC_MILESTONE_UAT_PLAYBOOK` ✓ (line 852)
- `write_user_agent_skills()` → `("sdlc-milestone-uat", SDLC_MILESTONE_UAT_SKILL)` ✓ (line 1050)

**Status:** All four registrations confirmed. No changes needed.

### T5: Build verification + sdlc update install check
**Action:**
1. Run `SDLC_NO_NPM=1 cargo test --all` — confirm passes
2. Run `sdlc update` — confirm `~/.claude/commands/sdlc-milestone-uat.md` is current

**Status:** To be executed in implementation phase.
