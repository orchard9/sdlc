# Design: sdlc-init First Milestone Handoff

## Problem

After `/sdlc-init` completes, the user is handed a menu:
> **Next:** `/sdlc-ponder` to explore your first idea — or `/sdlc-plan` if you already know what to build.

But init already heard what the user wants to build — in the argument (`/sdlc-init set up a proper foundation with quality gates`) or in Phase 2's opener answer. The user has to re-describe it as a separate command, which is needless friction and mental overhead at the worst moment: right after an intensive setup interview.

## Insight

Phase 2 Quick Capture collects two distinct things:
1. **Project identity** — what the project IS (domain, stack, users, constraints)
2. **Initial build scope** — what the user wants to BUILD FIRST (a rough description of the first milestone)

These are different. Current init treats them as one blob. The fix is to recognize and tag the "initial build scope" separately during Phase 2, then use it in the Finish section to derive the next step without re-asking.

## Design Decision

**Minimal change — no new phase.** Don't add Phase 7. The Finish section becomes smarter.

### Updated Finish Section Logic

After summarizing what was produced:

**If a build scope was captured during Phase 2** (from the arg OR from the opener/follow-ups):

1. Derive a slug from the build scope description (lowercase, hyphens, max 40 chars)
2. Create a ponder entry seeded with it:
```bash
sdlc ponder create <slug> --title "<title derived from build scope>"
sdlc ponder capture <slug> --content "<verbatim build scope>" --as brief.md
```
3. Assess plan-readiness. Scope is **plan-ready** if ALL of:
   - Has a clear deliverable ("hello world", "working config library", "quality gate CI")
   - Has a clear technology fit (matches the stack just established)
   - Is bounded (won't explode into infinite scope without exploration)

   Scope is **explore-ready** if it's directional but any of the above is unclear.

4. If **plan-ready**: run the feature-shaping protocol inline:
   - What's the MVP? What are the 2-4 features? What makes a milestone done?
   - Write plan to `/tmp/<slug>-plan.md`
   - Feed into state machine via the `/sdlc-plan` flow
   - Mark ponder committed: `sdlc ponder update <slug> --status committed`
   - `**Next:** /sdlc-prepare <milestone-slug>`

5. If **explore-ready**: stop after creating the ponder entry
   - `**Next:** /sdlc-ponder <slug>`

**If no build scope was captured** (user only described project identity):
- `**Next:** /sdlc-ponder` (unchanged — open invitation)

### Example Handoffs

| User description in Phase 2 | Assessment | Next |
|---|---|---|
| "foundation with quality gate checks, hello world, core libs for config/logs/secrets" | Plan-ready: bounded, concrete deliverables, stack is known | `/sdlc-ponder-commit <slug>` inline → `/sdlc-prepare <milestone-slug>` |
| "I want to build something that helps teams ship faster" | Explore-ready: directional but not shaped | `/sdlc-ponder <slug>` |
| "I don't know yet, I just want to set up the project" | No build scope | `/sdlc-ponder` (unchanged) |

## Playbook/Skill Updates

Update Step 11 in both SDLC_INIT_PLAYBOOK and SDLC_INIT_SKILL:

**Before:**
> 11. **Finish** — Summarize what was produced.
> **Next:** `/sdlc-ponder` to explore your first idea — or `/sdlc-plan` if you already know what to build.

**After:**
> 11. **Finish** — Summarize what was produced. Seed first milestone: if Phase 2 captured a build scope, derive a slug, create a ponder entry (`sdlc ponder create ... && sdlc ponder capture ... --as brief.md`). Assess plan-readiness (concrete deliverable + bounded scope). If plan-ready → inline feature-shaping → `/sdlc-plan` → `**Next:** /sdlc-prepare <slug>`. If explore-ready → `**Next:** /sdlc-ponder <slug>`. If no scope captured → `**Next:** /sdlc-ponder`.

## What Does NOT Change

- The phase structure of init (Phases 1-6 stay identical)
- The philosophy: init = foundation. Milestone creation is still the ponder/plan machinery.
  We're just making the handoff smarter, not merging the concerns.
- The user is never asked to re-describe what they want to build

## Commit Signal

This is implementable immediately — it's a template-only change to `sdlc_init.rs`.
No Rust code changes. No new CLI commands. No new data structures.
The change is: add ~30 lines to the Finish section of SDLC_INIT_COMMAND + update the one-liner in SDLC_INIT_PLAYBOOK and SDLC_INIT_SKILL.
