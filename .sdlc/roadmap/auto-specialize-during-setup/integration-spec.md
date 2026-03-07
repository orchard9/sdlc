# Integration Spec: Auto-Specialize During Init

## Decision
Replace `/sdlc-init` Phase 6 (Team) with an inline call to the `/sdlc-specialize` workflow. Init handles Vision + Architecture + Config; specialize handles team design with real codebase knowledge.

## Changes Required

### `sdlc_init.rs` (all 4 platform variants)
- **Remove** Phase 6 (Team) — roster design, gate, agent generation, AGENTS.md update
- **Replace** with Phase 6: 'Specialize — AI Team' that instructs the agent to follow the `/sdlc-specialize` workflow inline
- **Renumber** Phase 7 (Seed First Milestone) stays as Phase 7

### Phase 6 replacement text (Claude format)
```markdown
## Phase 6: Specialize — AI Team

Now that Vision and Architecture are written, survey the codebase and design a tailored AI team.

Follow the `/sdlc-specialize` workflow:
1. Survey the project (reads VISION.md, ARCHITECTURE.md, source dirs, config files)
2. Summarize purpose, stack, domain areas — present to user for confirmation
3. Design 3-5 specialist roles matched to actual codebase structure
4. Gate: present roster table, wait for user approval
5. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each
6. Update AGENTS.md with Team section

This replaces any previously generated agents.
```

### `sdlc_specialize.rs`
No changes needed — already works post-architecture.

### CLAUDE.md
Update `/sdlc-init` description to mention specialize integration.

## Key Decisions
- Handoff: instruction-level (init says 'follow specialize workflow')
- Timing: after Architecture written to disk
- Existing agents: specialize overwrites, no merge
- No skip flag — team setup is the value
- Blast radius: template text only, 4 platform variants
