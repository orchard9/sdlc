# Design: Smarter sdlc-init Finish

## Overview

This is a text-only change to the `sdlc-init` command template. No Rust code changes are required. The change adds a **Phase 7: Seed First Milestone** section to the end of the `SDLC_INIT_COMMAND` constant in `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`, and updates the companion `SDLC_INIT_PLAYBOOK` and `SDLC_INIT_SKILL` constants for cross-platform consistency.

---

## Decision: Where Does the Logic Live?

Per the Architecture Principle in CLAUDE.md: "Rust = Data, Skills = Logic." The auto-seed logic is agent decision logic — determining whether scope is thick enough, synthesizing milestone structure, deciding how many features to create. This belongs in the skill instruction text, not in Rust code.

The Rust layer (`sdlc-core`) is not touched. All new behavior is expressed as instruction text in the command template.

---

## New Phase 7: Seed First Milestone

The new section is inserted immediately after the current **Finish** section, replacing the existing "Next: /sdlc-ponder or /sdlc-plan" handoff.

### Phase 7a: Assess scope thickness

The agent evaluates whether Phase 2 yielded concrete deliverables:

**Thin scope** (skip seed) — any of:
- User said "I don't know what to build yet" or "exploring an idea"
- Phase 2 captured only a vague mission statement with no features or timeline
- The project description is abstract ("a platform for X") with no specifics

**Thick scope** (proceed to seed) — all of:
- Phase 2 captured at least one concrete deliverable (a named feature, a specific capability, or a timeline milestone)
- The agent can derive a user-observable goal from the scope

### Phase 7b: Synthesize milestone structure

From the Phase 2 build scope, the agent derives:

| Element | Derivation Rule |
|---|---|
| Milestone slug | Lowercase, hyphens, max 40 chars. Prefer: `v1-<domain-noun>` or `v1-<capability>` |
| Milestone title | 4-8 words, user-facing outcome: "Core [noun] [verb phrase]" |
| Milestone vision | One sentence: "A [persona] can [action], which [value]." |
| Features | 2-5, each semantically cohesive. Named from the scope deliverables. |
| Acceptance test | `- [ ]` checklist from the deliverables. Stored at `/tmp/<slug>_acceptance.md`. |

Features stay coarse at this stage — they will be refined by `/sdlc-prepare` later. Do not over-decompose.

### Phase 7c: Seed via sdlc CLI (inline plan execution)

```bash
# Create milestone
sdlc milestone create <slug> --title "<title>"
sdlc milestone update <slug> --vision "<vision>"
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance.md

# For each feature
sdlc feature create <feature-slug> --title "<feature-title>" --description "<description>"
sdlc milestone add-feature <slug> <feature-slug>
```

This mirrors the sdlc-plan command pattern exactly — ensuring idempotency. Re-running sdlc-init on a project that already has the milestone will update, not duplicate.

### Phase 7d: Updated Finish summary

Replace the existing Finish block with:

```
✓ VISION.md
✓ ARCHITECTURE.md
✓ .sdlc/config.yaml (project.name, project.description[, quality thresholds])
✓ Agents: [Name — Role], [Name — Role], ...
✓ AGENTS.md updated
✓ First milestone seeded: <slug> ([N] features)
```

And end with:
```
**Next:** /sdlc-prepare <slug>
```

Or, if scope was too thin:
```
**Next:** /sdlc-ponder   — explore your first idea, then /sdlc-plan when ready
```

---

## Changes to SDLC_INIT_PLAYBOOK

The playbook (Gemini/OpenCode variant) gets a new step 11:
> "**Seed first milestone** — if Phase 2 captured concrete deliverables, synthesize into one milestone + 2-5 features using `sdlc milestone create`, `sdlc feature create`, `sdlc milestone add-feature`. Otherwise skip and go to /sdlc-ponder."

And the final **Next** is updated to include the milestone-focused path.

---

## Changes to SDLC_INIT_SKILL

The skill (generic agents variant) gets an updated Workflow table entry and updated Outcome table:

| Outcome | Next |
|---|---|
| Scope thick → milestone seeded | `**Next:** /sdlc-prepare <slug>` |
| Scope thin → no milestone seeded | `**Next:** /sdlc-ponder` |

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs` | Add Phase 7 to SDLC_INIT_COMMAND, update SDLC_INIT_PLAYBOOK, update SDLC_INIT_SKILL |

After editing, run `sdlc update` to reinstall the updated templates to user directories.

---

## Non-Goals

- No changes to sdlc-core Rust types
- No changes to any other command template
- No changes to the interview gates (Vision/Architecture/Roster approvals remain unchanged)
- No network calls, no LLM calls from Rust — the agent executes the synthesis
