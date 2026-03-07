## Current State Analysis

The AgentsPage (`/agents`) already has a two-section layout:
1. **Project Team** — reads from `<project_root>/.claude/agents/*.md` via `GET /api/project/agents`
2. **Workstation** — reads from `~/.claude/agents/*.md` via `GET /api/agents`

Both sections use the same `AgentSection` component with different titles/icons.

### What Jordan wants
- Page title should say "Team" (currently says "Agents")
- Route is `/agents` — should it be `/team`?
- Clearer visual distinction between project-scope vs global-scope agents
- The `~/.claude/agents/` path is the only global source currently. Jordan's brief says `~/.*` which may mean scanning `~/.agents/` as well.

### Current counts (this repo)
- Project agents: 23 (`.claude/agents/`)
- Workstation agents: 23 (`~/.claude/agents/`)

⚑  Decided: The two-tier data architecture (two API endpoints) is already correct.

?  Open: Should `~/.agents/` also be scanned? (Agent Skills open standard uses that path)
?  Open: Route rename `/agents` → `/team`?