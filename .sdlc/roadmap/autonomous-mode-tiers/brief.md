# Brief: Three Operational Modes for Autonomous Work

**Origin:** Extracted from team conversation dump (jx12n + Xist), 2026-03-04

## Summary

jx12n explicitly calls for "3 good workflows" for sdlc-run and the dev-driver. The conversation surfaces three distinct modes organically: (1) fully autonomous with zero conflict tolerance — for product people, PMs, small shops, POCs; (2) hybrid — user is present and available for escalations, can absorb 3-4 simultaneous milestones including conflicting ones; (3) enterprise-permissive — can absorb conflict cost, large teams, ATG-style where refactors can't wait for everyone to clear.

## Key Signals

- **Strong (Strategy):** "i think dialing in 3 good workflows could work really well - and if i separate them and have them focused they should be straight forward - specifically how it applies to this flow and the sdlc-run task" — jx12n, explicit
- **Strong (Product):** "full autonomous mode, ill deal with mistakes later / might be ok for a product person creating a poc / not ok for atg" — jx12n
- **Strong (Product):** "Especially small shops with few/no devs. Put it in 'go as fast as you can while being sure no file/concept conflicts exist in simultaneous milestones'" — Xist
- **Strong (Process):** "if we set it 'Xist is here, he can handle conflicts for the next 8 hours' then maybe we tell it 'max 3-4 things on the same files at the same time'" — Xist, describing hybrid mode
- **Strong (Strategy):** "ATG for example who cannot drop everything for some refactor, and they are willing/able to pay the dev price to deal with merge conflicts" — Xist, describing enterprise mode

## Relevant Excerpts

> "i think dialing in 3 good workflows could work really well - and if i separate them and have them focused they should be straight forward - specifically how it applies to this flow and the sdlc-run task" — jx12n

> "full autonomous mode, ill deal with mistakes later / might be ok for a product person creating a poc / not ok for atg" — jx12n

> "Perhaps for fully-autonomous mode, we set the threshold at 'ZERO MAX CONFLICTS'" — Xist

> "But if we set it 'Xist is here, he can handle conflicts for the next 8 hours' then maybe we tell it 'max 3-4 things on the same files at the same time'" — Xist

## Open Questions

- Where is the mode configured? Per-project in `.sdlc/config.yaml`? Per-run flag? User presence signal?
- How does the dev-driver change behavior per mode? (dispatch rules, escalation thresholds, max parallel)
- Does `sdlc-run` also need a mode concept, or is it always single-feature and mode is a dev-driver concern?
- "User is present" — is this a flag the user sets, or detected (e.g., agent token active, recent activity)?
- What does the escalation path look like in hybrid mode — Slack? UI notification? Blocking agent turn?
