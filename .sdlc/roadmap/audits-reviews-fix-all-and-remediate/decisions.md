## Decision: Finding-Closure Protocol for Approve Audit/Review

**⚑ Decided:** Three concrete changes to enforce that every audit/review finding is explicitly resolved before approval.

### The Problem

Current `approve_audit` / `approve_review` protocol in `sdlc-next`: *"Read the artifact, verify it is complete and correct, approve."* There is no instruction about what to do with individual findings. Agents can (and do) approve past unresolved findings, or incorrectly blast `fix-all`/`remediate` at the whole codebase when a single targeted change was needed.

The security audit example shows exactly what should happen but what the protocol doesn't enforce:
- REQUIRED fix → one targeted code change (`validate_slug`)
- MEDIUM finding → `sdlc task add` for future auth work
- Two LOW findings → accept with documented rationale

### Tensions Resolved

**Dan Reeves:** "Does 'always forward' already cover this?"
No — 'always forward' describes what to do when you *encounter* a problem during implementation. Audit findings arrive as a structured list requiring deliberate classification. Without explicit protocol, agents approve past them or over-correct with broad sweeps.

**Tobias Krenn:** "Is the approval section getting too long?"
The change is a clean split into two tracks (spec/design/tasks/qa/merge vs review/audit). It's clearer, not longer. The audit track adds ~5 lines; the other track stays identical.

**Felix Wagner:** "/ scoping" — These tools operate on the whole codebase. After an audit finding (e.g. 'path traversal in register_route'), the correct response is a 3-line targeted fix, not a sweep. The guidance must say this explicitly.

### Changes

1. **`sdlc-next.rs` COMMAND + PLAYBOOK** — split approval into two tracks; add find/fix/track/accept protocol for review+audit
2. **CLAUDE.md Ethos** — add bullet: *Audits and reviews close every finding*
3. **`.sdlc/guidance.md`** — add §12: Audit & Review Findings (~8 lines, table format)
