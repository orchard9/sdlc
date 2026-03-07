# Spec: Init Phase 6 → Specialize Handoff

## Problem

`/sdlc-init` Phase 6 (Team) attempts to design and generate specialist agents inline during a 7-phase init marathon. In practice, only a single agent (knowledge-librarian) gets created — the agent runs out of steam navigating 4 sub-phases (6a-6d) of complex roster design, gating, and file generation.

Meanwhile, `/sdlc-specialize` already does this job correctly via a structured 4-session workflow with explicit gates. The work is duplicated and the inferior version runs first.

## Solution

Replace Phase 6 (Team) in the `/sdlc-init` template with a handoff to the `/sdlc-specialize` workflow. Init handles Vision + Architecture + Config; specialize handles team design with real codebase knowledge.

## Scope

### In Scope
- Remove Phase 6 sub-phases 6a-6d (roster design, gate, agent generation, AGENTS.md update) from `sdlc_init.rs`
- Replace with Phase 6: "Specialize — AI Team" that instructs the agent to follow the `/sdlc-specialize` workflow inline
- Apply the same change to all 4 platform variants (Claude, Gemini, OpenCode, Agents)
- Renumber Phase 7 (Seed First Milestone) stays as Phase 7
- Update CLAUDE.md `/sdlc-init` description if needed

### Out of Scope
- Changes to `/sdlc-specialize` itself (already works post-architecture)
- UI fixes for agents page (separate feature: agents-page-two-tier)
- Changes to the specialize workflow logic

## Phase 6 Replacement Text

The new Phase 6 instructs the agent to:
1. Survey the project (reads VISION.md, ARCHITECTURE.md, source dirs, config files)
2. Summarize purpose, stack, domain areas — present to user for confirmation
3. Design 3-5 specialist roles matched to actual codebase structure
4. Gate: present roster table, wait for user approval
5. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each
6. Update AGENTS.md with Team section

This is an instruction-level handoff — init's Phase 6 text says "follow the specialize workflow" rather than reimplementing it.

## Blast Radius

- **Files changed**: `sdlc_init.rs` (Claude template), plus Gemini/OpenCode/Agents platform variants in the same directory
- **Risk**: Low — template text only, no Rust logic changes, no state machine changes
- **Existing agents**: specialize overwrites on re-run, no merge needed

## Acceptance Criteria

- [ ] `/sdlc-init` Phase 6 references the specialize workflow instead of inline roster design
- [ ] All 4 platform variants updated consistently
- [ ] Phase 7 (Seed First Milestone) unchanged and correctly numbered
- [ ] Old Phase 6 sub-phases (6a-6d) fully removed from all templates
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes
- [ ] `cargo clippy --all -- -D warnings` clean
