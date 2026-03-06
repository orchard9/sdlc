# Brief: AI-Powered Conceptual Merge with Acceptance Re-run

**Origin:** Extracted from team conversation dump (jx12n + Xist), 2026-03-04

## Summary

When two parallel milestones produce merge conflicts, AI can resolve them more intelligently than humans by loading BOTH milestones' full context: original files (common ancestor), goals, changes, and acceptance criteria. The agent resolves the conflict conceptually — understanding *why* each side made its changes — then re-runs acceptance criteria for both milestones to verify nothing broke. This is distinct from `git mergetool`-style line resolution; it's semantic conflict resolution with regression verification.

## Key Signals

- **Strong (Engineering):** "The AI has BOTH the projects available. Old version of files (common ancestor) / Project OTHER goals + changes + acceptance criteria / Project THIS goals + changes + acceptance criteria" — Xist, very specific proposal
- **Strong (Engineering):** "we can have the AI resolve those conceptually and ensure that after the merge... it runs acceptance criteria for both the previous merge and the new one, to ensure we didn't break anything" — Xist
- **Strong (Engineering):** "Right the skill is a conceptual merge in addition to figuring out the text that makes that happen. I have done AI conceptual merges a few times so far, it's surprisingly effective." — Xist, validated from experience
- **Strong (Engineering):** jx12n's failure case shows *why* context matters: without feature 1+2 context, feature 4's conflict resolution broke them
- **Weak (Process):** "You need a good way to know the eventual result is actually working to all expected functions" — acceptance criteria re-run is the answer here

## Relevant Excerpts

> "The AI has BOTH the projects available. Old version of files (common ancestor) / Project OTHER goals + changes + acceptance criteria / Project THIS goals + changes + acceptance criteria. So when we merge OTHER into main, and we need to rebase THIS onto main, when we have merge conflicts, we can have the AI resolve those conceptually..." — Xist

> "Right the skill is a conceptual merge in addition to figuring out the text that makes that happen. I have done AI conceptual merges a few times so far, it's surprisingly effective." — Xist

> "i had something for it a year ago and the problem i ran into then was that the context was always from one point of view... the 4th would conflict with them all, and because it didnt have the context of features 1 and 2 it would break them" — jx12n

## Open Questions

- How does the merge agent get access to the common ancestor? Git operations inside the agent run?
- Does this require worktrees (each milestone on its own branch) to function?
- Acceptance criteria re-run: UAT? Just compile + test? How automated?
- Is this a skill or a server-side feature that triggers automatically on conflict detection?
- What's the escalation path if AI can't resolve conceptually — human diff view?
