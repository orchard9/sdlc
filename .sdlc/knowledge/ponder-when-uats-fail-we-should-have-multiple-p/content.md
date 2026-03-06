---
session: 1
timestamp: 2026-03-03T16:15:00Z
orientation:
  current: "Idea fully shaped — three pathways designed, scope bounded to template-only changes + new sdlc-recap command"
  next: "Commit to milestone with /sdlc-ponder-commit — two core features, one optional wave-2 feature"
  commit: "Met — deliverables are concrete, scope is minimal (no Rust changes), and the decision tree is complete"
---

**jordan · Owner**
when uats fail we should have multiple pathways

this is a core template - the uat. when it fails we need to give it multiple pathways that are all actionable - NEVER STALL

1. if it fails and can be fixed, then fix it and re-run the uat
2. if theres a fairly simple escalation path, then go through that
3. if its complicated, then there is a skill called "recap" in the workspace, we want to adopt that into a template, run it, and propose ponder sessions

this is essentially close off the current work session with whatever has been complete and figure out how to move forward from current state

we should have sdlc-recap as a new sdlc skill from this work, we should also have updates to the uat template and pathways in the code for escalation and proposal

---

## Context Load

Loaded the full UAT infrastructure:
- **UAT template**: `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs` — 237 lines, Mode A/B with Playwright, classifies selector breaks vs code bugs
- **Server routes**: `start_milestone_uat`, `fail_milestone_uat`, `stop_milestone_uat`, `submit_milestone_uat_human` in `runs.rs:760-957`
- **Escalation system**: Full CRUD in `crates/sdlc-core/src/escalation.rs` — 4 kinds (SecretRequest, Question, Vision, ManualTest), linked blocker comments on features
- **Existing recap**: `/recap` command at `~/.claude/commands/recap.md` — conversation-scoped, produces Working On / Completed / Remaining / Suggested Next
- **No sdlc-recap exists yet** — this is a new command

## The Problem

Current UAT failure flow has a dead end at Step 5-6:
1. Agent calls `POST /api/milestone/<slug>/uat/fail`
2. Outputs: "**Next:** `/sdlc-run <feature>` — fix, then re-run"
3. Agent **stops**. Human must manually figure out what to do.

This violates the ethos: "Never pause. Decide and act on every failure without asking."

## Team Assembly

Recruited three thought partners:
- **Priya Nair** — state machine correctness, failure boundary analysis
- **Dan Reeves** — systems minimalist, challenges new primitives
- **Dana Cho** — product skeptic, scope and value check

## Session Dialogue

### Dan Reeves — "Does this need a new command?"

Dan challenged whether `sdlc-recap` is warranted given the existing `/recap` command.

**Key distinction**: `/recap` is conversation-scoped — it only sees the current Claude session. `sdlc-recap` is **state-aware** — it reads `sdlc status`, milestone info, feature states, and recent git history before synthesizing. It also creates real artifacts (tasks, escalations, ponder entries) rather than just text output.

Dan accepted the distinction but insisted: if recap only runs after UAT failures, put it in the UAT template. A standalone command is only warranted if it runs independently.

⚑ Decided: `sdlc-recap` is standalone because it applies to any session close (end of day, when stuck, when handing off). The UAT template *invokes* it as part of Pathway 3.

### Priya Nair — "Bound the retry loop"

Priya flagged two critical issues:

1. **Retry bound**: Pathway 1 (fix-and-retry) needs a hard limit. Two retries max, then fall through to Pathway 2 or 3. No infinite fix→break→fix loops.

2. **State during retry**: Milestone stays in `Verifying`. Agent doesn't call `uat/fail` until it has exhausted Pathway 1. No state transitions during the retry loop.

3. **Ponder creation**: Does the agent create ponder entries directly or just suggest them? Per the ethos ("autonomous by default"), the agent creates them via `sdlc ponder create`. They start in `exploring` status.

⚑ Decided: Agent creates ponder entries directly. Max 2 fix-and-retry cycles before falling through.

### Dana Cho — "What Rust code actually changes?"

Dana pushed hard on scope. The answer: **none**.

- Escalation CLI already exists (`sdlc escalate create`)
- Ponder CLI already exists (`sdlc ponder create`)
- Task CLI already exists (`sdlc task add`)
- Status CLI already exists (`sdlc status --json`)
- Decision logic belongs in skill text, not Rust code (architecture principle)
- Recap synthesis is agent work, not library work

This is a **template-only change** plus a new command. Two features, maybe three if we add server integration.

? Open: Does the milestone UAT failure UI need a "Run Recap" button? This would require a server endpoint. Parked for wave 2.

## Decision Tree

```
UAT FAILED
  │
  ├─ All failures fixable? (localized code bugs, < 3 files)
  │   └─ YES → PATHWAY 1: Fix → Rerun (max 2 cycles)
  │       └─ Still failing? → reclassify, fall through
  │
  ├─ Any failure needs human input? (missing secret, unclear req)
  │   └─ YES → PATHWAY 2: Escalate
  │       - `sdlc escalate create --kind <type> ...`
  │       - Create tasks for fixable items
  │       - Call uat/fail, output escalation next steps
  │
  └─ Complex/architectural?
      └─ YES → PATHWAY 3: Recap + Ponder
          - Create tasks for fixable items
          - Synthesize state-aware recap
          - `sdlc ponder create` for hard problems
          - Commit completed work
          - Call uat/fail, output ponder next steps
```

## Deliverables

### Milestone: UAT Failure Pathways

**Feature 1: uat-failure-pathways**
- Modify UAT template: add triage classification + 3 pathways
- Update all platform variants (Claude, Gemini, OpenCode, Agent Skills)

**Feature 2: sdlc-recap-command**
- New `sdlc_recap.rs` command template
- State-aware recap: reads sdlc state + git history
- Forward Motion section creates real artifacts
- Four platform variants + guidance table + migration

**Feature 3 (wave 2, optional): recap-server-integration**
- Server endpoint for recap runs
- UI button on milestone UAT failure view
- Parked unless template-only proves insufficient

### Acceptance Test
1. Run UAT on milestone with known failures → agent enters appropriate pathway
2. Pathway 1: agent fixes and reruns within same session
3. Pathway 3: agent creates ponder entries, commits, outputs next steps
4. `/sdlc-recap` standalone produces state-aware recap with forward motion
5. Every session ends with exactly one concrete **Next:** line

---

---
session: 1
timestamp: 2026-03-03T16:15:00Z
orientation:
  current: "Idea fully shaped — three pathways designed, scope bounded to template-only changes + new sdlc-recap command"
  next: "Commit to milestone with /sdlc-ponder-commit — two core features, one optional wave-2 feature"
  commit: "Met — deliverables are concrete, scope is minimal (no Rust changes), and the decision tree is complete"
---

**jordan · Owner**
when uats fail we should have multiple pathways

this is a core template - the uat. when it fails we need to give it multiple pathways that are all actionable - NEVER STALL

1. if it fails and can be fixed, then fix it and re-run the uat
2. if theres a fairly simple escalation path, then go through that
3. if its complicated, then there is a skill called "recap" in the workspace, we want to adopt that into a template, run it, and propose ponder sessions

this is essentially close off the current work session with whatever has been complete and figure out how to move forward from current state

we should have sdlc-recap as a new sdlc skill from this work, we should also have updates to the uat template and pathways in the code for escalation and proposal

---

## Context Load

Loaded the full UAT infrastructure:
- **UAT template**: `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs` — 237 lines, Mode A/B with Playwright, classifies selector breaks vs code bugs
- **Server routes**: `start_milestone_uat`, `fail_milestone_uat`, `stop_milestone_uat`, `submit_milestone_uat_human` in `runs.rs:760-957`
- **Escalation system**: Full CRUD in `crates/sdlc-core/src/escalation.rs` — 4 kinds (SecretRequest, Question, Vision, ManualTest), linked blocker comments on features
- **Existing recap**: `/recap` command at `~/.claude/commands/recap.md` — conversation-scoped, produces Working On / Completed / Remaining / Suggested Next
- **No sdlc-recap exists yet** — this is a new command

## The Problem

Current UAT failure flow has a dead end at Step 5-6:
1. Agent calls `POST /api/milestone/<slug>/uat/fail`
2. Outputs: "**Next:** `/sdlc-run <feature>` — fix, then re-run"
3. Agent **stops**. Human must manually figure out what to do.

This violates the ethos: "Never pause. Decide and act on every failure without asking."

## Team Assembly

Recruited three thought partners:
- **Priya Nair** — state machine correctness, failure boundary analysis
- **Dan Reeves** — systems minimalist, challenges new primitives
- **Dana Cho** — product skeptic, scope and value check

## Session Dialogue

### Dan Reeves — "Does this need a new command?"

Dan challenged whether `sdlc-recap` is warranted given the existing `/recap` command.

**Key distinction**: `/recap` is conversation-scoped — it only sees the current Claude session. `sdlc-recap` is **state-aware** — it reads `sdlc status`, milestone info, feature states, and recent git history before synthesizing. It also creates real artifacts (tasks, escalations, ponder entries) rather than just text output.

Dan accepted the distinction but insisted: if recap only runs after UAT failures, put it in the UAT template. A standalone command is only warranted if it runs independently.

⚑ Decided: `sdlc-recap` is standalone because it applies to any session close (end of day, when stuck, when handing off). The UAT template *invokes* it as part of Pathway 3.

### Priya Nair — "Bound the retry loop"

Priya flagged two critical issues:

1. **Retry bound**: Pathway 1 (fix-and-retry) needs a hard limit. Two retries max, then fall through to Pathway 2 or 3. No infinite fix→break→fix loops.

2. **State during retry**: Milestone stays in `Verifying`. Agent doesn't call `uat/fail` until it has exhausted Pathway 1. No state transitions during the retry loop.

3. **Ponder creation**: Does the agent create ponder entries directly or just suggest them? Per the ethos ("autonomous by default"), the agent creates them via `sdlc ponder create`. They start in `exploring` status.

⚑ Decided: Agent creates ponder entries directly. Max 2 fix-and-retry cycles before falling through.

### Dana Cho — "What Rust code actually changes?"

Dana pushed hard on scope. The answer: **none**.

- Escalation CLI already exists (`sdlc escalate create`)
- Ponder CLI already exists (`sdlc ponder create`)
- Task CLI already exists (`sdlc task add`)
- Status CLI already exists (`sdlc status --json`)
- Decision logic belongs in skill text, not Rust code (architecture principle)
- Recap synthesis is agent work, not library work

This is a **template-only change** plus a new command. Two features, maybe three if we add server integration.

? Open: Does the milestone UAT failure UI need a "Run Recap" button? This would require a server endpoint. Parked for wave 2.

## Decision Tree

```
UAT FAILED
  │
  ├─ All failures fixable? (localized code bugs, < 3 files)
  │   └─ YES → PATHWAY 1: Fix → Rerun (max 2 cycles)
  │       └─ Still failing? → reclassify, fall through
  │
  ├─ Any failure needs human input? (missing secret, unclear req)
  │   └─ YES → PATHWAY 2: Escalate
  │       - `sdlc escalate create --kind <type> ...`
  │       - Create tasks for fixable items
  │       - Call uat/fail, output escalation next steps
  │
  └─ Complex/architectural?
      └─ YES → PATHWAY 3: Recap + Ponder
          - Create tasks for fixable items
          - Synthesize state-aware recap
          - `sdlc ponder create` for hard problems
          - Commit completed work
          - Call uat/fail, output ponder next steps
```

## Deliverables

### Milestone: UAT Failure Pathways

**Feature 1: uat-failure-pathways**
- Modify UAT template: add triage classification + 3 pathways
- Update all platform variants (Claude, Gemini, OpenCode, Agent Skills)

**Feature 2: sdlc-recap-command**
- New `sdlc_recap.rs` command template
- State-aware recap: reads sdlc state + git history
- Forward Motion section creates real artifacts
- Four platform variants + guidance table + migration

**Feature 3 (wave 2, optional): recap-server-integration**
- Server endpoint for recap runs
- UI button on milestone UAT failure view
- Parked unless template-only proves insufficient

### Acceptance Test
1. Run UAT on milestone with known failures → agent enters appropriate pathway
2. Pathway 1: agent fixes and reruns within same session
3. Pathway 3: agent creates ponder entries, commits, outputs next steps
4. `/sdlc-recap` standalone produces state-aware recap with forward motion
5. Every session ends with exactly one concrete **Next:** line
