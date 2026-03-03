---
session: 1
timestamp: 2026-03-02T00:00:00Z
orientation:
  current: "Design complete — sdlc-init Finish section gets smarter handoff logic using Phase 2 captured build scope"
  next: "Implement: edit sdlc_init.rs SDLC_INIT_COMMAND Finish section + update SDLC_INIT_PLAYBOOK + SDLC_INIT_SKILL"
  commit: "Design artifact approved — ready to commit now"
---

## Session 1 — sdlc-init First Milestone Handoff

**Trigger:** User observed that after `/sdlc-init`, you have to manually run `/sdlc-ponder <description>` to start the first milestone. The description was already captured in init — needless re-work.

**Evidence from the example:**
```
/sdlc-ponder set up a proper foundation with full quality gate checks - it should be a hello world that has working core libraries for things like configs, logs, etc. It should integrate with our secrets properly, etc
```
This was run AFTER init. The description describes the first milestone. Init already knew this.

## Recruited team
- Ben Hartley · Developer productivity UX — `.claude/agents/ben-hartley.md`
- Dan Reeves · Systems minimalist — `.claude/agents/dan-reeves.md`
- Felix Wagner · CLI ergonomics — `.claude/agents/felix-wagner.md`

## Phase 2 disambiguation

⚑ Decided: Phase 2 Quick Capture collects two distinct things:
1. **Project identity** — what the project IS
2. **Initial build scope** — what the user wants to BUILD FIRST

Current init treats them as one blob. The fix is to tag them separately.

**Ben Hartley · Developer Productivity UX**
"Activation-energy problem. User just did an intensive interview. Handing them a menu ('ponder OR plan') at the end puts routing burden on them at the worst moment. The system should route for them based on what it just learned."

**Dan Reeves · Systems minimalist**
"Don't add Phase 7. Init = foundation. Minimal fix: make the `**Next:**` smarter. No new phases. The slug is already derivable. One smarter sentence."

**Felix Wagner · CLI ergonomics**
"Output contract is wrong. A CLI command ends with 'here's exactly what to type next' — not a menu. If the user described 'foundation with quality gates', the output should say `**Next:** /sdlc-ponder foundation-quality-gates-hello-world`. Pre-filled. Derivable from Phase 2."

## Design synthesized

⚑ Decided: Minimal change — no new phase. Finish section becomes smarter.

**Logic added to Finish section:**
1. If build scope was captured in Phase 2 (from arg OR opener):
   - Derive slug from build scope
   - `sdlc ponder create <slug>` + `sdlc ponder capture <slug> ... --as brief.md`
   - Assess plan-readiness: has concrete deliverable + bounded scope?
   - Plan-ready → inline feature-shaping → `/sdlc-plan` → `**Next:** /sdlc-prepare <milestone-slug>`
   - Explore-ready → `**Next:** /sdlc-ponder <slug>`
2. If no build scope captured → `**Next:** /sdlc-ponder` (unchanged)

⚑ Decided: "Plan-ready" checklist (all must be true):
- Has a clear deliverable
- Has a clear technology fit (stack is known from Phase 2/3)
- Is bounded (won't require extensive exploration to shape)

? Open (minor): Should inline feature-shaping in Finish produce tasks, or just milestone + features? Recommend: milestone + features only (tasks come from `/sdlc-run` or `/sdlc-prepare`).

## Scrapbook artifacts
- `brief.md` — verbatim user description + key questions
- `design.md` — full design: problem, insight, decision, example handoffs, implementation notes

## Implementation scope
Template-only change to `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`:
- SDLC_INIT_COMMAND Finish section: add ~30 lines of new handoff logic
- SDLC_INIT_PLAYBOOK: update Step 11 one-liner
- SDLC_INIT_SKILL: update outcome table row

No Rust code changes. No new CLI commands. No new data structures.
