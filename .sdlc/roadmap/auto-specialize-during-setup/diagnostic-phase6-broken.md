# Diagnostic: Why Phase 6 Is Broken

## Bug 1: Phase 6 only creates the librarian

Init Phase 6 (`sdlc_init.rs:207-277`) instructs the agent to design 2-4 specialist agents. In practice, only the `knowledge-librarian` was created. The other 22 agents in `.claude/agents/` were created by `/sdlc-specialize` and `/sdlc-recruit` — not by init.

**Root cause:** Phase 6 is a wall of text with 4 sub-phases (6a-6d). The agent likely runs out of steam or gets confused by the complexity. The specialize command (`sdlc_specialize.rs`) does the same job but with clear session boundaries and explicit gates — it works because it's structured as a 4-session workflow, not a single phase in a 7-phase marathon.

## Bug 2: Agents don't appear in the UI

`AgentsPage.tsx:99` calls `api.getAgents()` → `/api/agents` → reads `~/.claude/agents/` (user-level).
All actual agents live at `<project>/.claude/agents/` (project-level).
`~/.claude/agents/` is empty.

The `api.getProjectAgents()` endpoint exists (`/api/project/agents`) but the UI never calls it.

**Fix:** AgentsPage needs to fetch both user-level and project-level agents, or at minimum show project agents.

## Implication for this ponder

Both bugs validate the Session 1 decision: replace Phase 6 with specialize. But we also need a UI fix — that's a separate task, not template work.