# Tasks: AgentsPage Two-Tier Display

## T1: Add project agents state and parallel fetch
- Add `projectAgents`, `projectLoading`, `projectError` state
- Update `load()` to call both `api.getProjectAgents()` and `api.getAgents()` via `Promise.allSettled`
- Handle each result independently

## T2: Extract AgentSection helper
- Create inline `AgentSection` component with props: title, agents, warning, emptyText, loading, error
- Renders section heading with count, warning line if provided, agent cards list, loading/error/empty states

## T3: Restructure AgentsPage layout
- Replace single agent list with two `AgentSection` instances
- Project Team section first (no warning, empty text guides adding `.claude/agents/`)
- Workstation section second (amber "not shared" warning)
- Update page header subtitle to reference both sources
- Show full empty state only when both sections have zero agents

## T4: Verify build
- Run `npm run build` in frontend to confirm no type/lint errors
