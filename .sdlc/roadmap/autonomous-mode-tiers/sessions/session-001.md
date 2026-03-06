---
session: 1
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Three modes named but not yet specified — zero-conflict, hybrid, enterprise-permissive"
  next: "Define each mode's concrete behavior: dispatch rules, parallelism cap, escalation path, config surface"
  commit: "Spec for how dev-driver and sdlc-run behave differently per mode, with config schema"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation dump.

### Signals extracted

jx12n explicitly calls for "3 good workflows" and says they should be focused and straight forward. The three modes emerge naturally from the conversation:

**Mode 1: Zero-conflict autonomous**
- Target user: product person, PM, solo dev, POC builder
- Behavior: only dispatch milestones with zero file/concept overlap; queue the rest
- Escalation: none (fully unattended)
- Tolerance: "ill deal with mistakes later"

**Mode 2: Hybrid (user present)**
- Target user: active developer, "I'm here for the next 8 hours"
- Behavior: allow 3-4 concurrent milestones even on overlapping files; escalate conflicts to user
- Escalation: user handles conflicts as they come in
- Tolerance: willing to context-switch for conflict resolution

**Mode 3: Enterprise-permissive**
- Target user: ATG-style large teams, can't pause for refactors
- Behavior: high parallelism, accept merge cost, teams have devs to handle conflicts
- Escalation: specialist agents + human devs, not the project owner
- Tolerance: merge conflict is a known, budgeted cost

### Why this might matter

Currently the dev-driver has one behavior: dispatch up to 4 slots. There's no concept of tolerance, user presence, or conflict budget. For the PM building a POC this is fine — they don't care about conflicts. But for ATG using this for real production work, the difference between these modes is the difference between the tool being usable and unusable.

The mode could also drive different UX in the dashboard — showing "conflict queue" in zero-conflict mode, "active escalations" in hybrid mode.

### Open questions

- Is mode a project-level config or a session-level flag?
- How does zero-conflict mode interact with the current 4-slot cap? Does it reduce the cap dynamically?
- Hybrid mode needs a real escalation surface — what does that look like in the current UI?
- Does the enterprise mode need any special server-side support, or is it just "permissive dispatch rules"?

### Suggested first exploration

Start with zero-conflict mode (easiest to specify — just filter out overlapping milestones). What does the config surface look like? What does `select_parallel_work()` need to know to apply the filter?
