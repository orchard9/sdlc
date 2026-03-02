---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Protocol defined and implemented — finding-closure for approve_audit/approve_review is now in templates, CLAUDE.md, and guidance.md"
  next: "Run `sdlc init` / `sdlc update` to propagate the updated sdlc-next template to installed commands"
  commit: "All three changes shipped — ponder can converge"
---

**jordan · Owner**
audits-reviews-fix-all-and-remediate

We need tighter control about what we issue fix all and remediate to, for example, this is after an audit. everything in this list we should either remediate or fix. this is a template chance, a concept change that deserves to be in claude.md, and a guidance.md adjustment

be token efficient in your changes to instruction files like guidance.md and templates

[full security audit omitted — see brief.md]

---

## Session 1

### Framing the problem

The security audit for `orchestrator-webhook-routing` concluded: **APPROVE with one required fix** — add `validate_slug` to `register_route`. It also produced:
- MEDIUM finding → track as task (HMAC auth, future work)
- LOW finding (template injection) → accept, operator-trusted
- LOW finding (payload TTL) → accept, no data leakage

The existing `approve_audit` protocol in `sdlc-next` says: *"Read the artifact, verify it is complete and correct, approve autonomously."* There is no instruction on what to do with findings. An agent reading this could:
1. Approve past the required fix (worst outcome)
2. Run `/fix-all` on the whole codebase (wrong tool, wrong scope)
3. Do nothing with the MEDIUM finding (lost signal)

Jordan's ask: tighter control. Every finding must be either fixed or remediated. Template change, CLAUDE.md concept, guidance.md adjustment. Be token-efficient.

---

### Thought partner: Dan Reeves (systems minimalist)

> "Does 'always forward' already cover this? Create a task, keep moving. That's the existing protocol."

**Dan** is right that "always forward" addresses findings during implementation. But it's not specific enough for the structured list an audit produces. When you're in `approve_audit`, you have a formal list of findings — some required before ship, some future work, some no-action. "Create a task and keep moving" doesn't tell the agent which are which, or that targeted fixes must precede approval for required items.

The gap: **no protocol for classifying findings** at approval time.

**⚑ Decided:** The existing ethos doesn't cover this — a specific protocol is warranted.

---

### Thought partner: Tobias Krenn (skeptical engineering lead)

> "Is the approval section in sdlc-next getting too long? You're adding a special case."

Looking at the current template, `approve_audit` and `approve_review` are already listed alongside `approve_spec`, `approve_design`, etc. — but they are fundamentally different. Spec/design/tasks approval is: *"is the artifact well-written and complete?"* Review/audit approval is: *"what do I do with each specific finding?"*

The change splits the approval section into two tracks, each with its own protocol. This is **clearer**, not longer. The existing approval block stays identical for spec/design/tasks/qa_plan/merge. The new block adds ~6 lines for review/audit.

**⚑ Decided:** Split is the right structure. Not a complexity increase — a precision increase.

---

### Thought partner: Felix Wagner (developer tooling architect)

> "The fix-all / remediate scoping is the real danger. Those tools sweep the whole codebase. After a specific audit finding — 'path traversal in register_route' — you don't want to touch files outside that route."

Felix is right. The wrong pattern an agent might follow:
1. Read audit → see "tool_name lacks slug validation"
2. Run `/fix-all` to sweep for validation issues everywhere
3. Make changes outside scope, possibly introducing regressions

The right pattern:
1. Read audit → see "tool_name lacks slug validation"
2. Add exactly: `if let Err(e) = validate_slug(&body.tool_name) { return Err(...); }`
3. Track HMAC auth as a task. Accept template injection (operator-trusted).

The guidance needs to explicitly say: **targeted fix for a specific finding, fix-all/remediate only for systemic patterns.**

**⚑ Decided:** Guidance must call out fix-all/remediate as wrong tools for individual findings.

---

### Changes implemented

**1. `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`**

COMMAND template: split approval section. `approve_spec/design/tasks/qa_plan/merge` keep the existing two-step protocol. New block for `approve_review` and `approve_audit`:
- Enumerate every finding
- For each: Fix now (targeted) | Track (`sdlc task add`) | Accept (documented rationale)
- No silent skips
- Approve only after all findings are resolved

PLAYBOOK template: added step 5a after step 5 with the same three-action protocol in one line.

**2. `CLAUDE.md` — Ethos section**

New bullet added after "User perspectives are first-class":

> **Audits and reviews close every finding.** When `approve_audit` or `approve_review` is the directive, enumerate every finding and take one explicit action: fix it now (targeted code change), track it (`sdlc task add`), or accept it (documented rationale). Silence is not acceptance. Use targeted fixes for specific findings — `fix-all` and `remediate` are for systemic codebase-wide patterns, not individual audit items.

**3. `.sdlc/guidance.md` — §12 Audit & Review Findings**

New section with a 3-row table (Required fix | Future work | No-action) and an explicit statement that fix-all/remediate are for systemic patterns. ~10 lines.

---

### ? Open: install propagation

The `sdlc-next` template change lives in `init/commands/sdlc_next.rs`. It takes effect when users run `sdlc init` or `sdlc update` to reinstall commands. Existing installs have the old template. Consider whether `sdlc update` should be run as part of committing this.

---

### Commit signal met

All three deliverables are implemented. The concept is clear, the changes are minimal, and the protocol is unambiguous. Ponder can converge.
