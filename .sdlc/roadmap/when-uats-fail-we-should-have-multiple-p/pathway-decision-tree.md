# Pathway Decision Tree

## After UAT fails, the agent evaluates:

```
UAT FAILED
  │
  ├─ Can I fix this right now?
  │   ├─ YES: failures are code bugs with clear fixes
  │   │   └─ PATHWAY 1: Fix → Rerun UAT → Loop
  │   │       Conditions:
  │   │       - Failures are in files the agent can edit
  │   │       - Fix is localized (< 3 files changed)
  │   │       - Not architectural (wrong pattern, missing infrastructure)
  │   │       Max retries: 2 (original run + 2 fix cycles)
  │   │
  │   └─ NO: ─┐
  │            │
  ├─ Is there a simple escalation path?
  │   ├─ YES: Missing secret, broken infra, unclear requirement
  │   │   └─ PATHWAY 2: Escalate → Create tasks → Close session
  │   │       Actions:
  │   │       - `sdlc escalate create --kind <type> --title "..." --context "..."`
  │   │       - Create tasks for what CAN be done meanwhile
  │   │       - Link escalation to blocking feature
  │   │       EscalationKinds that fit: secret_request, question, manual_test
  │   │
  │   └─ NO: complex/architectural problem
  │       └─ PATHWAY 3: Recap → Propose ponder → Close session
  │           Actions:
  │           - Run `/sdlc-recap` to capture current state
  │           - Identify hard problems → propose ponder sessions
  │           - Commit completed work
  │           - Output concrete next steps as ponder slugs
```

## Key design constraint

The decision tree must be **in the skill template**, not in Rust code.
Per architecture principle: Rust = Data, Skills = Logic.
The UAT template gains a new "Step 6 — Triage failures" section.
