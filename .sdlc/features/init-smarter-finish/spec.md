# Spec: Smarter sdlc-init Finish — Auto-Seed First Milestone

## Problem

The `sdlc-init` command (in `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`) ends with a generic "Next: /sdlc-ponder or /sdlc-plan" handoff. By the time Phase 2 (Quick Capture) is complete, the agent already knows:

- What the project is building
- The scope of work for the first deliverable
- The tech stack and key constraints

All this context lives in the interview thread but is immediately abandoned. The user must re-describe their project scope in a separate `/sdlc-plan` invocation, duplicating effort and creating friction at the most important moment — when momentum is highest.

## Goal

At the end of `sdlc-init`, automatically synthesize the captured Phase 2 build scope into a first milestone and seed it into the state machine without requiring a separate `/sdlc-plan` invocation.

## Desired Behavior

After the agent writes AGENTS.md (the final setup step), instead of printing "Next: /sdlc-ponder or /sdlc-plan" and stopping, the sdlc-init command:

1. **Assesses whether enough scope exists** — checks if Phase 2 captured concrete deliverables (not just a vague description). If only a vague "what we're building" answer exists (no specifics on features or timeline), skips auto-seed and falls back to current behavior.

2. **Synthesizes the first milestone** — from the captured build scope, derives:
   - A milestone slug and title (e.g., `v1-core-foundation`)
   - A one-sentence milestone vision: what a user can do when this ships
   - 2-5 features derived from the build scope (each semantically cohesive)
   - An acceptance test checklist

3. **Seeds the state machine** — runs the sdlc-plan flow inline to create the milestone, features, and any obvious tasks.

4. **Updates the Finish summary** to include what was seeded:
   ```
   ✓ VISION.md
   ✓ ARCHITECTURE.md
   ✓ .sdlc/config.yaml
   ✓ Agents: [Name — Role], ...
   ✓ AGENTS.md updated
   ✓ First milestone seeded: v1-core-foundation (3 features)
   ```

5. **Ends with a concrete Next** pointing at the seeded milestone:
   ```
   Next: /sdlc-prepare v1-core-foundation
   ```
   or, if no milestone was seeded (scope too thin):
   ```
   Next: /sdlc-ponder   # explore your first idea
   ```

## What Does NOT Change

- Phases 1–6 of sdlc-init are unchanged. The smarter finish is additive — it runs after the existing Team step.
- The gates (Vision approval, Architecture approval, Roster approval) are unchanged.
- If the user says "I don't know what to build yet" or scope is clearly too thin, sdlc-init gracefully skips the seed and falls back to the existing "Next: /sdlc-ponder" path.
- The underlying CLI commands used for the seed follow existing sdlc-plan conventions exactly (idempotent, deterministic slugs).

## Scope

This feature is a change to the **sdlc-init command template** only — a text change in `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`. No Rust logic changes are required. The Finish section of the SDLC_INIT_COMMAND constant is updated to add the auto-seed phase. The SDLC_INIT_PLAYBOOK and SDLC_INIT_SKILL constants are also updated for consistency across platforms.

`sdlc update` will re-install the updated templates to user directories.

## Success Criteria

1. After running sdlc-init on a project with clear build scope, the finish summary shows a seeded milestone.
2. The seeded milestone and features are immediately visible via `sdlc milestone list` and `sdlc feature list`.
3. The final "Next" line points at `/sdlc-prepare <seeded-milestone-slug>`.
4. On a project with thin scope ("building something, not sure what"), sdlc-init gracefully skips the seed and prints "Next: /sdlc-ponder".
5. Re-running sdlc-init is idempotent — the seed step uses sdlc-plan conventions, so existing milestones/features are updated, not duplicated.
