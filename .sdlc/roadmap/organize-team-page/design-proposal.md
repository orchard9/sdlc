## Design Proposal

### Changes Required

#### 1. Rename route and sidebar label
- Route: `/agents` → `/team` (keep `/agents` as redirect for bookmarks)
- Sidebar label: "Agents" → "Team"
- Page header: "Agents" → "Team"
- Icon: `Bot` → `Users` (more fitting for "Team" framing)

#### 2. Section organization (already exists, needs polish)
The two-section split already exists in `AgentsPage.tsx`:
- **Project Team** (`<project_root>/.claude/agents/`) — shared via git, specific to this project
- **Workstation** (`~/.claude/agents/`) — personal, not shared

Jordan's brief says `~/.*` which could mean also scanning:
- `~/.agents/skills/` (Agent Skills open standard — used by sdlc init)
- Other AI CLI agent dirs

⚑  Decided: Keep the two-section data model as-is. It's already correct.

#### 3. Visual improvements
- Add agent count badges to section headers (already shows count)
- Add a visual separator or background difference between sections
- Project agents should feel "first-class" — show them first (already does)
- Workstation agents should feel "supplementary" — maybe slightly dimmer cards or an info callout

#### 4. Files to modify
- `frontend/src/pages/AgentsPage.tsx` — rename header, any visual tweaks
- `frontend/src/components/layout/Sidebar.tsx` — label + icon change
- `frontend/src/components/layout/BottomTabBar.tsx` — update `/agents` → `/team`
- `frontend/src/App.tsx` — route path change + redirect
- No backend changes needed (API endpoints stay the same)

### Scope: Small
This is a frontend-only rename + minor visual polish. No new APIs, no new data models.