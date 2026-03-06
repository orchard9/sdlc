# Current UAT Failure Flow — Gap Analysis

## What happens today when UAT fails

1. Agent runs acceptance test steps via Playwright
2. Tests fail → agent classifies: selector break vs code bug
3. Selector breaks: fixed inline, rerun once
4. Code bugs: `sdlc task add <feature> "UAT: ..."` 
5. If blocking failures remain: calls `POST /api/milestone/<slug>/uat/fail`
6. Milestone stays in `Verifying`
7. Template outputs: `**Next:** /sdlc-run <first-blocking-feature-slug>`

## The stall point

Step 7 is where things break. The template says "fix, then re-run" but:
- The UAT agent **stops**. It does not fix anything.
- The human must read the output, figure out what went wrong, manually invoke `/sdlc-run`, wait, then manually re-run `/sdlc-milestone-uat`
- If the problem is **architectural** (wrong approach, missing infrastructure), `/sdlc-run` will just hit the same wall

## What "NEVER STALL" requires

The agent must **always take one of the three pathways** before ending:
1. **Fix-and-retry**: Agent fixes the code and reruns UAT within the same session
2. **Escalate**: Agent creates an escalation with the right `EscalationKind` and concrete context
3. **Recap-and-ponder**: Agent produces a structured recap and proposes ponder sessions for hard problems

**Zero valid exit states where the agent just stops and says "you fix it".**
