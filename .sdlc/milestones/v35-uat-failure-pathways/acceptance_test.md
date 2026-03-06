# UAT Failure Pathways — Acceptance Test

## Scenario 1: Pathway 1 — Fix and Retry
1. Run `/sdlc-milestone-uat` on a milestone with a fixable test failure (e.g., wrong assertion value)
2. Agent classifies the failure as "Fixable"
3. Agent fixes the code and reruns the UAT within the same session
4. UAT passes on retry — milestone completes

## Scenario 2: Pathway 3 — Recap and Ponder
1. Run `/sdlc-milestone-uat` on a milestone with a complex/architectural failure
2. Agent classifies the failure as "Complex"
3. Agent creates tasks for any fixable items
4. Agent runs recap: gathers sdlc state, synthesizes progress, creates ponder entries
5. Agent commits completed work and calls `uat/fail`
6. Output ends with `**Next:** /sdlc-ponder <slug>`

## Scenario 3: Standalone Recap
1. Run `/sdlc-recap` independently (not from UAT failure)
2. Agent reads `sdlc status --json`, milestone info, and recent git history
3. Produces Working On / Completed / Remaining / Forward Motion sections
4. Forward Motion creates real artifacts (tasks, escalations, or ponder entries)
5. Ends with exactly one `**Next:**` line

## Definition of Done
- UAT template contains triage classification + 3 pathways
- Max 2 fix-and-retry cycles before falling through
- `/sdlc-recap` command installed via `sdlc update` on all 4 platforms
- No session ends without a concrete next action
