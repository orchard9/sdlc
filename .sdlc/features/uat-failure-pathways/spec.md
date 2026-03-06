# Spec: UAT Failure Triage and 3 Pathways in milestone-uat Template

## Problem

When `/sdlc-milestone-uat` runs and tests fail, the current template tells the agent to signal failure and instructs the human to "fix the feature tasks, then re-run this command." This leaves every UAT failure with the same dead-end outcome: the milestone stays in Verifying and no forward motion is taken.

A sophisticated UAT failure can be any of:
- A small fixable code bug the agent can repair inline
- A blocker requiring human judgment (missing env var, ambiguous requirement)
- A complex architectural mismatch that needs a rethink session

All three are currently treated identically. The agent takes no action and kicks the problem to the human.

## Solution

Add failure **triage classification** and **three structured pathways** to the `SDLC_MILESTONE_UAT_COMMAND` template in `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`.

Additionally, introduce a standalone `/sdlc-recap` command that can be run independently to synthesize project state and produce concrete forward-motion artifacts.

## Scope

### Feature 1: UAT Failure Triage + 3 Pathways (this feature)

Modify `sdlc_milestone_uat.rs`:

1. **Step 5 — Triage failures** (replaces current Step 5 "Flip milestone state")
   - Classify each failure as Fixable, Escalation, or Complex
   - Fixable: assertion fails on a value the agent can change; route returns wrong status; missing CSS class
   - Escalation: missing env var; server unreachable; unclear requirement; needs human judgment
   - Complex: wrong architectural approach; feature design doesn't match reality; multiple interacting failures

2. **Step 6 — Pathway 1: Fix and Retry**
   - If ALL failures are Fixable: fix code (< 3 files), rerun spec, re-parse results
   - Max 2 fix cycles; reclassify remaining after 2 cycles and fall through

3. **Step 7 — Pathway 2: Escalate**
   - If any failure is Escalation (and none Complex): create tasks for fixable items, create escalation for blocking items, call `uat/fail` endpoint
   - End with: `**Next:** resolve escalation <id>, then /sdlc-milestone-uat <slug>`

4. **Step 8 — Pathway 3: Recap and Propose**
   - If any failure is Complex: create tasks for fixable items, run recap (synthesize state, create ponder entries for complex failures), commit completed work, call `uat/fail` endpoint
   - End with: `**Next:** /sdlc-ponder <first-ponder-slug>`

5. **Step 9 — Final report** (updated table)
   - Verdicts: Pass, PassWithTasks, FixedAndPassed, Escalated, Recapped

Update Gemini playbook (`SDLC_MILESTONE_UAT_PLAYBOOK`) and Agent Skill (`SDLC_MILESTONE_UAT_SKILL`) variants to match.

### Feature 2: /sdlc-recap Command (companion feature `sdlc-recap-command`)

New command: `sdlc-recap`
- Platform variants for Claude Code, Gemini, OpenCode, Agent Skills
- State-aware: reads `sdlc status --json`, milestone info, recent git history
- Sections: Working On / Completed / Remaining / Forward Motion
- Forward Motion creates real artifacts: tasks, escalations, or ponder entries
- Ends with exactly one `**Next:**` line
- Registered in `write_user_*` functions, guidance table, and migration

## Out of Scope

- Server-side recap endpoint / frontend button (parked as optional wave 2)
- Changes to existing UAT mode detection (Mode A / Mode B)
- Changes to Pass / PassWithTasks handling (unchanged)

## Acceptance Criteria

- UAT template has Steps 5–9 with triage classification and 3 pathways
- Pathway 1 retries up to 2 times before falling through
- Pathway 2 creates escalations and signals failure
- Pathway 3 creates ponder entries, commits work, signals failure
- `/sdlc-recap` installed via `sdlc update` on all 4 platforms
- Every session ends with exactly one `**Next:**` line

## Definition of Done

Per the acceptance test in `.sdlc/milestones/v35-uat-failure-pathways/acceptance_test.md`:
- UAT template contains triage classification + 3 pathways
- Max 2 fix-and-retry cycles before falling through
- `/sdlc-recap` command installed via `sdlc update` on all 4 platforms
- No session ends without a concrete next action
