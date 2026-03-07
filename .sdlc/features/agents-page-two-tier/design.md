# Design: AgentsPage Two-Tier Display

## Approach

Frontend-only change to `AgentsPage.tsx`. No backend changes needed.

## Component Changes

### AgentsPage.tsx

**State additions:**
- `projectAgents: AgentDefinition[]` — fetched from `api.getProjectAgents()`
- `projectLoading: boolean`, `projectError: string | null`

**Data fetching:**
- `load()` calls both `api.getProjectAgents()` and `api.getAgents()` in parallel via `Promise.allSettled`.
- Each result updates its own state independently — one failing doesn't block the other.

**Layout (top to bottom):**

1. **Page header** — existing, update subtitle to mention both sources.
2. **Project Team section**
   - Section heading: "Project Team" with count badge
   - If empty: dashed-border empty state with guidance to add `.claude/agents/*.md`
   - If populated: list of `AgentCard` components
3. **Workstation section**
   - Section heading: "Workstation" with count badge
   - Subtle amber/yellow info line: "Not shared — these agents exist only on your machine"
   - If empty: minimal "No workstation agents" text (not a big empty state)
   - If populated: list of `AgentCard` components
4. **Full empty state** — only shown if both sections have zero agents and no errors.

### Section component (inline)

Extract a small `AgentSection` helper within the file:
```tsx
function AgentSection({ title, agents, warning?, emptyText, loading, error })
```

This keeps the two sections DRY without creating a separate file.

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/AgentsPage.tsx` | Add dual-fetch, two-section layout, warning badge |

## No Backend Changes

Both `/api/agents` and `/api/project/agents` already exist and return `AgentDefinition[]`.
