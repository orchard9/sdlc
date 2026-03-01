# Code Review: Update sdlc-milestone-uat Skill with Playwright Support

## Summary

This review covers the changes to `crates/sdlc-cli/src/cmd/init.rs` implementing Mode A (run existing Playwright spec) and Mode B (generate spec from checklist) in the `sdlc-milestone-uat` skill.

**Verdict: APPROVED**

## Changes Reviewed

### 1. `SDLC_MILESTONE_UAT_COMMAND`

**Before:** ~100 lines. Manual checklist execution, no Playwright, `allowed-tools: Bash, Read, Write, Edit, Glob, Grep`.

**After:** ~150 lines. Full Mode A/B decision tree, Playwright execution, result parsing, failure classification, summary.md format, `allowed-tools` extended with 9 Playwright MCP tool namespaces.

**Assessment:**
- Mode A flow is complete: spec detection → `npx playwright test --reporter=json` → parse `results.json` → classify failures (selector vs code bug) → create tasks for bugs, fix and rerun for selectors → write summary.md and uat_results.md.
- Mode B flow is complete: read checklist → navigate via Playwright MCP tools → write spec with `getByRole`/`getByTestId` locators → run → fix selectors (up to 3 iterations) → continue to Mode A synthesis.
- Summary.md format matches spec (Verdict, Tests, Tasks created, Results, Failures table).
- Existing `uat_results.md` and `sdlc milestone complete` lifecycle steps preserved.
- Verdict enumeration correct: Pass, PassWithTasks, Failed.
- Ethos section updated to reflect Playwright-driven approach.
- No regressions to the milestone complete lifecycle.

### 2. `SDLC_MILESTONE_UAT_PLAYBOOK`

**Before:** 19 lines. Generic steps, no Playwright.

**After:** 28 lines. Mode A / Mode B decision in Step 2, Playwright test execution referenced, summary.md write step, selector break vs code bug classification included.

**Assessment:** Appropriate concise format for Gemini/OpenCode. All required Mode A/B language present.

### 3. `SDLC_MILESTONE_UAT_SKILL`

**Before:** 8 lines. Generic workflow, no Playwright.

**After:** 10 lines. Mode A / Mode B clearly enumerated in Workflow section, Playwright test execution and summary.md referenced, verdict enumeration present.

**Assessment:** Appropriate minimal format for Agents SKILL.md.

## Build Verification

```
SDLC_NO_NPM=1 cargo build --all → Finished (0 errors, 0 warnings)
```

## Install Verification

```
sdlc update → updated: ~/.claude/commands/sdlc-milestone-uat.md
              updated: ~/.gemini/commands/sdlc-milestone-uat.toml
              updated: ~/.opencode/command/sdlc-milestone-uat.md
              updated: ~/.agents/skills/sdlc-milestone-uat/SKILL.md
```

Content checks on `~/.claude/commands/sdlc-milestone-uat.md`:
- "Mode A" occurrences: 7 ✓
- "Mode B" occurrences: 4 ✓
- "playwright test" occurrences: 3 ✓
- "results.json" occurrences: 3 ✓
- "summary.md" occurrences: 5 ✓

## No Issues Found

The change is a pure string constant edit in one file. No logic changes, no data model changes, no API changes. The build is clean and all three skill variants correctly describe the Mode A/B decision and Playwright execution.
