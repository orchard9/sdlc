---
session: 1
timestamp: 2026-03-07T22:00:00Z
orientation:
  current: "Idea is fully shaped — rename /agents to /team, keep two-section layout, frontend-only change"
  next: "Commit to milestone and implement"
  commit: "Jordan confirms the rename approach and scope"
---

**jordan · Owner**
organize "/team" page

the team page should be organized by project agents (agents that are located in the project root) and global agents located in ~/.*

---

**Facilitator**

Explored the current state of the `/agents` page. Good news: the two-tier architecture Jordan wants already exists. The page has two sections — "Project Team" (from `.claude/agents/` in the project root) and "Workstation" (from `~/.claude/agents/`). Two separate API endpoints serve them.

### What's actually needed

The ask distills to a **rename + reframe**, not a restructure:

1. **Route rename**: `/agents` → `/team` (sidebar, bottom tab bar, router)
2. **Label rename**: "Agents" → "Team" in sidebar and page header
3. **Icon swap**: `Bot` → `Users` to match the "Team" framing
4. **Section labels are already correct**: "Project Team" and "Workstation" map exactly to "project root agents" and "global agents"

### Tensions explored

**? Open: Should `~/.agents/` be scanned too?**
Jordan's brief says `~/.*` — the workstation endpoint currently only reads `~/.claude/agents/`. The `~/.agents/skills/` directory exists (Agent Skills open standard, populated by `sdlc init`). However, those are skills, not agents — different shape. Recommendation: leave the scan as `~/.claude/agents/` only for now. If Jordan wants `~/.agents/` included, that's a separate backend change.

**⚑ Decided: No backend changes needed.**
The API endpoints (`/api/project/agents` and `/api/agents`) already serve the right data. This is purely a frontend rename.

**⚑ Decided: Keep both section names.**
"Project Team" and "Workstation" are clear and accurate. Project agents are shared via git; workstation agents are local-only.

### Scope

Frontend-only, 4 files:
- `frontend/src/App.tsx` — route path
- `frontend/src/pages/AgentsPage.tsx` — page header text
- `frontend/src/components/layout/Sidebar.tsx` — label + icon
- `frontend/src/components/layout/BottomTabBar.tsx` — route root

This is a small, self-contained change — ready to implement directly without a milestone.
