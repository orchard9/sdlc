# Design: Update SDLC_MILESTONE_UAT_COMMAND in init.rs

## Overview

This is a pure content update — no new types, no new functions, no architectural changes. The work is to verify and (if needed) update three `const &str` values in `crates/sdlc-cli/src/cmd/init.rs`.

## Target Constants

### `SDLC_MILESTONE_UAT_COMMAND` (Claude Code)
- Full-detail slash command installed to `~/.claude/commands/sdlc-milestone-uat.md`
- Must contain all 6 steps: load milestone, mode detection, Mode A run, Mode B generate, write summary.md, write uat_results.md, flip milestone state

### `SDLC_MILESTONE_UAT_PLAYBOOK` (Gemini/OpenCode)
- Concise playbook installed to `~/.gemini/commands/sdlc-milestone-uat.toml` and `~/.opencode/command/sdlc-milestone-uat.md`
- Must cover same logical steps in compact form

### `SDLC_MILESTONE_UAT_SKILL` (Agents)
- Minimal SKILL.md installed to `~/.agents/skills/sdlc-milestone-uat/SKILL.md`
- Must cover same workflow steps minimally

## Verification Steps

1. Read each const — confirm Mode A command, Mode B MCP tool list, step 3 path pattern, step 4 path pattern, step 5 `sdlc milestone complete <slug>` on Pass/PassWithTasks
2. If any step is missing or incomplete, patch the const in place
3. `SDLC_NO_NPM=1 cargo test --all` — verify build and tests pass
4. `sdlc update` — verify `~/.claude/commands/sdlc-milestone-uat.md` reflects final content

## Assessment of Current State

After reading `init.rs` lines 2261–2485 (`SDLC_MILESTONE_UAT_COMMAND`) and lines 2751–2780 (`SDLC_MILESTONE_UAT_PLAYBOOK`) and lines 2890–2909 (`SDLC_MILESTONE_UAT_SKILL`):

- `SDLC_MILESTONE_UAT_COMMAND`: complete. All steps present including Mode A CLI, Mode B MCP tools, step 3 summary.md path, step 4 uat_results.md path, step 5 `sdlc milestone complete`.
- `SDLC_MILESTONE_UAT_PLAYBOOK`: complete. Concise coverage of Mode A/B, failure triage, steps 4-6.
- `SDLC_MILESTONE_UAT_SKILL`: complete. Workflow steps 1-6 present.

**Conclusion:** No source edits needed. Tasks T1-T3 are already done. Tasks T4-T5 (verify write_user_* registrations + build test + sdlc update) are the remaining verification steps.
