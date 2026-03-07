# Design: Init Phase 6 → Specialize Handoff

## Overview

Replace the inline team-design logic in `/sdlc-init` Phase 6 with a handoff instruction that tells the agent to follow the `/sdlc-specialize` workflow. Three template constants need editing in `sdlc_init.rs`.

## Changes by Template

### 1. `SDLC_INIT_COMMAND` (Claude format, ~70 lines removed)

**Remove** (lines 207-277): Phase 6 sub-phases 6a (roster design), 6b (gate), 6c (agent file generation), 6d (AGENTS.md update).

**Replace with:**
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

Phase 7 (Seed First Milestone) stays unchanged and keeps its number.

### 2. `SDLC_INIT_PLAYBOOK` (Gemini/OpenCode format)

**Replace steps 8-10** (design team, create agents, update AGENTS.md) with:
```
8. **Specialize — AI Team** — Follow `/sdlc-specialize` workflow: survey project, summarize for user confirmation, design 3-5 specialists, gate roster approval, generate agent files + AGENTS.md.
```

Renumber step 11 (seed) → 9, step 12 (finish) → 10.

### 3. `SDLC_INIT_SKILL` (Agents format)

**Replace steps 8-10** with:
```
8. **Specialize — AI Team** — Follow `/sdlc-specialize` workflow: survey project, confirm with user, design 3-5 specialists, gate roster, generate `.claude/agents/` + AGENTS.md.
```

Renumber step 11 (seed) → 9.

### 4. Finish block in `SDLC_INIT_COMMAND`

Update the summary checklist to say "Agents: via /sdlc-specialize" instead of listing individual agent names.

## Files Touched

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs` | Edit 3 const strings: COMMAND, PLAYBOOK, SKILL |

## No Other Changes Needed

- `sdlc_specialize.rs` — already works post-architecture, no changes
- State machine / rules — unaffected (template text only)
- CLAUDE.md — `/sdlc-init` description already says "team"; the specialize handoff is an implementation detail
