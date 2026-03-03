# Spec: Product Summary format contract in /sdlc-ponder skill

## Problem

Ponder session logs are **event logs** — sequential transcripts that record tool calls, agent scaffolding, and conversation. Product stakeholders need **documents** — structured by importance, readable in 2 minutes, free of operational noise. The failure is not jargon alone: 60% of session content is overhead, not thinking.

The `/sdlc-ponder` skill currently defines the session log format (YAML frontmatter + free-form dialogue) but provides no convention for a human-readable summary at the end of each session. Without this, the ponder workspace is opaque to anyone who isn't reading the full transcript.

## Solution

Add a required `## Product Summary` section to the session log format in the `/sdlc-ponder` skill instruction. This is an **authoring convention**, not infrastructure — no new Rust code, no new CLI commands, no schema changes. The skill instruction enforces it at the agent level.

The format contract defines four stable H3 subsections:
- **What we explored** — 1–2 sentences in plain English
- **Key shifts** — decisions made, assumptions revisited, prior beliefs updated
- **Implications** — what this means for the feature/milestone, in product language
- **Still open** — 1–2 unresolved questions phrased as decisions to be made

H3 subsection labels are locked — agents cannot rename them. This stability is required so the UI can extract and display the summary on session cards without parsing arbitrary headings.

## Scope

**In scope:**
- Update `SDLC_PONDER_COMMAND` constant in `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs` to include the Product Summary format contract in the Session Log Protocol section
- Update `SDLC_PONDER_PLAYBOOK` (Gemini/OpenCode) and `SDLC_PONDER_SKILL` (Agents) variants for the same content
- The format contract must be clearly documented: schema, rules, and enforcement note
- Existing session logs are NOT retroactively modified (forward-only convention)

**Out of scope:**
- UI extraction of `## Product Summary` for session card preview (future milestone)
- Knowledge librarian indexing of Implications sections (future)
- CLI-level validation of session file format (future)
- Any changes to Rust data structures, YAML schemas, or REST endpoints

## Acceptance Criteria

1. The `SDLC_PONDER_COMMAND` constant includes the `## Product Summary` schema with all four H3 subsections (What we explored, Key shifts, Implications, Still open)
2. The format rules are stated: H3 labels are stable and locked, Implications must use product language not tech jargon, Still open must be phrased as decisions not investigations
3. The session file format example in the skill includes the `## Product Summary` section after the dialogue body
4. The `SDLC_PONDER_PLAYBOOK` and `SDLC_PONDER_SKILL` variants include a concise summary of the Product Summary requirement
5. The skill instruction makes clear this section is what gets surfaced on session cards in the UI (motivating the stability requirement)
6. `sdlc init` and `sdlc update` propagate the updated skill content to user directories

## Source

This feature was crystallized from the ponder entry `ponder-conversations-need-to-be-more-acc`. The format contract was defined in `.sdlc/roadmap/ponder-conversations-need-to-be-more-acc/roll-up-format-contract.md`.
