# Brief: File and Concept Impact Pre-flight

**Origin:** Extracted from team conversation dump (jx12n + Xist), 2026-03-04

## Summary

Before dispatching parallel milestones, compute the set of files each milestone will touch and check for overlap. Milestones with conflicting file footprints either queue behind each other or escalate to the user — depending on the operational mode. Extends beyond file conflicts: conceptual conflicts (stale docs, divergent architecture decisions) can be equally destabilizing and need their own detection layer.

## Key Signals

- **Strong (Engineering):** Xist explicitly proposes P4-style pre-flight: "compute the list of all the files we will change... that list could be used to determine what other milestones can be executed in parallel without any risk of merge conflicts."
- **Strong (Engineering):** jx12n's failure case: parallel feature 4 broke features 1-3 because it lacked their context at conflict resolution time. Pre-flight context loading is the fix.
- **Strong (Product):** "the conceptual conflicts are pretty rough, or even just the mere suggestion of a conceptual conflict thats leftover in docs can throw everything out of whack"
- **Strong (Process):** In zero-conflict autonomous mode, the threshold is: "ZERO MAX CONFLICTS — maybe 5 things at once, but none in areas that have conflicting files involved."
- **Weak (Process):** Escalation path unclear — "i need to spend more time and really dial in on escalation"

## Relevant Excerpts

> "A key to increasing parallelism to max will likely be having a way to understand which part of the code base any given milestone will impact. In p4 one of the things I make it do up front is compute the list of all the files we will change... that list could also then be used to help us determine what other milestones can be executed in parallel without any risk of merge conflicts." — Xist

> "the conceptual conflicts are pretty rough, or even just the mere suggestion of a conceptual conflict thats leftover in docs can throw everything out of whack" — jx12n

> "i had something for it a year ago and the problem i ran into then was that the context was always from one point of view — so what would happen is that id do 3 features, then the 4th would conflict with them all, and because it didnt have the context of features 1 and 2 it would break them" — jx12n

## Open Questions

- Where does the file impact list come from? Agent predicts it? User declares it? Derived from spec/design artifacts?
- Conceptual conflict detection is harder — what's the mechanism? Embedding similarity on design docs?
- Does pre-flight gate dispatch, or just warn?
- How does this interact with `select_parallel_work()` in Rust? Could this become a filter in that function?
- What's the UX for a blocked slot — shown in dashboard, escalated, or silently queued?
