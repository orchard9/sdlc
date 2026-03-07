# AgentsPage UI Spec: Two-Tier Agent Display

## Decision (from Session 2 + Owner directive)

Project-level agents (`.claude/agents/`) are the **primary** list. User-workstation-level agents (`~/.claude/agents/`) are a **secondary** list with a clear warning that they are NOT shared with other developers.

## Data Flow

- `api.getProjectAgents()` -> `/api/project/agents` -> reads `<project>/.claude/agents/` (already exists)
- `api.getAgents()` -> `/api/agents` -> reads `~/.claude/agents/` (already exists)

Both endpoints return `AgentDefinition[]`. No backend changes needed.

## Frontend Changes (AgentsPage.tsx only)

### State
```tsx
const [projectAgents, setProjectAgents] = useState<AgentDefinition[]>([])
const [userAgents, setUserAgents] = useState<AgentDefinition[]>([])
```

### Fetch
```tsx
const load = useCallback(async () => {
  const [proj, user] = await Promise.all([
    api.getProjectAgents().catch(() => []),
    api.getAgents().catch(() => []),
  ])
  setProjectAgents(proj)
  setUserAgents(user)
})
```

### Layout

**Section 1: Project Team** (primary)
- Header: "Project Team" with count badge
- Subtext: path to `.claude/agents/` — shared via git
- Uses existing `AgentCard` component
- Empty state: "No project agents. Run `/sdlc-specialize` to create a team."

**Section 2: Your Workstation** (secondary, visually demoted)
- Header: "Your Workstation" with count badge
- Warning banner (always visible, not dismissable):
  "These agents live on YOUR machine only. They are **not** shared with other developers on this project."
- Visually distinct: muted/dimmed cards, maybe a border-dashed container
- Empty state: small text, no big illustration

### Visual Hierarchy
- Project section: normal card styling, full opacity
- Workstation section: slightly muted, with info/warning banner at top
- The warning should use an icon (Lock or User) + amber/yellow accent to draw attention without being alarming

## Key Principle
A developer glancing at this page should immediately understand: top section = the team everyone shares, bottom section = just mine.