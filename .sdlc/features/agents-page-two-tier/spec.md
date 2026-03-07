# Spec: AgentsPage Two-Tier Display

## Summary

Restructure the AgentsPage to show two distinct sections: **Project Team** agents (from `<project>/.claude/agents/`) as the primary section, and **Workstation** agents (from `~/.claude/agents/`) as the secondary section with a "not shared with teammates" warning.

## Problem

Currently the AgentsPage only fetches and displays workstation-level agents (`/api/agents`). The backend already serves project-level agents via `/api/project/agents`, but the frontend ignores them. Users cannot see which agents are committed to the project (shared with the team) vs which are personal to their machine.

## Requirements

1. **Two-section layout**: Project Team agents appear first, Workstation agents appear below.
2. **Project Team section**: Fetches from `/api/project/agents`. Header: "Project Team". Shows agents from `<project>/.claude/agents/`. If empty, show a concise empty state ("No project agents — add `.claude/agents/*.md` to share agents with the team").
3. **Workstation section**: Fetches from `/api/agents` (existing). Header: "Workstation". Displays a subtle warning badge/note: "Not shared — these agents exist only on your machine".
4. **Shared AgentCard**: Both sections reuse the existing `AgentCard` component unchanged.
5. **Count labels**: Each section shows its count (e.g., "3 agents").
6. **Loading/error states**: Both fetches run in parallel. Each section handles its own loading and error state independently.
7. **Empty page**: If both sections are empty, show the existing empty state.

## Non-goals

- No backend changes required (both endpoints already exist).
- No new API types or fields.
- No agent CRUD — display only.

## Acceptance Criteria

- Project agents appear in the top section with "Project Team" heading.
- Workstation agents appear below with "Workstation" heading and a "not shared" indicator.
- Both sections load independently and show their own counts.
- Empty sections show appropriate messaging.
- Existing AgentCard expand/collapse and model badges work identically in both sections.
