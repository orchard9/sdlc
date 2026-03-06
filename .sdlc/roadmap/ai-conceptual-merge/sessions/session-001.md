---
session: 1
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Novel capability proposed with real-world validation — Xist has done AI conceptual merges manually"
  next: "Define the agent's context package: what exactly goes in, what comes out, how is it triggered"
  commit: "Skill spec for AI conceptual merge: inputs, resolution protocol, acceptance re-run contract"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation dump.

### Signals extracted

Xist has already done AI conceptual merges manually and calls them "surprisingly effective." The key insight is that AI merge resolution fails when it only has one side's context. With both sides' full context — the common ancestor, each project's goals, each project's changes, and each project's acceptance criteria — the AI can reason about *why* changes were made and produce a resolution that respects both intents.

The protocol Xist describes:
1. Milestone A merges to main
2. Milestone B needs to rebase onto new main
3. Conflicts appear
4. Load: common ancestor files + A's goals/changes/AC + B's goals/changes/AC
5. AI resolves conceptually
6. Re-run acceptance criteria for BOTH A and B to verify

jx12n's failure case is the null hypothesis — without this full-context approach, conflict resolution breaks prior features.

### Why this might matter

This could be what unlocks high-parallelism mode safely. If conflicts are resolved reliably by AI (with regression protection), the cost of conflicts drops significantly. Xist's framing: "I think AI can do that better than humans." The SDLC system already has all the context needed — specs, designs, acceptance criteria — it just needs a merge agent that loads all of it.

### Open questions

- Does this require each milestone to be on its own git branch? (Probably yes — worktrees per milestone)
- What format is the "acceptance criteria" the agent re-runs? The acceptance_test.md from the milestone? UAT scenarios?
- Is this triggered automatically when a rebase produces conflicts, or manually invoked?
- Could this be a skill first (manual invocation) before being a server-side automation?

### Suggested first exploration

Spike: can a skill take two feature slugs, load their full artifact context (spec, design, tasks, acceptance test), stage a synthetic conflict, and resolve it correctly? Validate Xist's claim that it's "surprisingly effective" in the sdlc context specifically.
