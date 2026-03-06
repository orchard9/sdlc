---
session: 1
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Raw signal from conversation — file overlap detection as a parallelism gate"
  next: "Interrogate mechanism: how does a milestone declare or predict its file footprint?"
  commit: "Clear problem statement + proposed detection mechanism + integration point with select_parallel_work()"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation dump.

### Signals extracted

Xist describes P4's pre-flight pattern: before executing work in a workspace, compute the list of files that will be changed. That list serves double duty — it's required for P4's checkout model, but also maps naturally to conflict detection. The insight is that sdlc already has the parallel work queue; what it lacks is a collision layer on top of it.

jx12n's failure case is instructive: AI conflict resolution without full feature context broke prior work. The fix isn't just better conflict resolution — it's preventing the conflict in the first place.

The conceptual conflict thread is separate and harder: even if no files overlap, two milestones can work against each other at the architecture/doc level. Example: one milestone refactors the database schema while another adds features assuming the old schema.

### Why this might matter

The dev-driver currently dispatches up to 4 slots with zero awareness of overlap. For small feature work (UI components, isolated CLI commands) this is fine. For larger structural work (DB migrations, core module refactors) it's a landmine. The gap grows as projects mature and milestones get more systemic.

This could also unlock a meaningful user-facing capability: the dashboard shows which milestones are safe to run together, and which are queued behind a blocking milestone.

### Open questions

- Is file impact prediction feasible pre-implementation? Agents could estimate from spec/design, but it's probabilistic.
- Is the right integration point `select_parallel_work()` (filter), or a separate pre-flight step before dispatch?
- Conceptual conflict detection may need embedding-based similarity on design artifacts — is that in scope here or a separate spike?

### Suggested first exploration

Focus on the file footprint mechanism first — ignore conceptual conflicts for now. Can an agent reading a feature's spec + design produce a reliable file impact estimate? What format? How does it integrate with the parallel work queue?
