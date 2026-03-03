# Spec: Orchestrator-aware empty states — suggestion chips replace generic empty state

## Problem

The dashboard's current empty state (`DashboardEmptyState`) is a generic "New Ponder" button that ignores what the orchestrator already knows about the project. A new user lands on an empty dashboard and sees a vague prompt with no actionable guidance tailored to their actual situation.

Additionally, individual dashboard zones (AttentionZone, CurrentZone, HorizonZone, ArchiveZone) render nothing when empty — leaving blank gaps in the layout with no contextual guidance.

## Goal

Replace the generic empty state with orchestrator-aware suggestion chips that surface the most relevant next actions based on project state. Give each dashboard zone a meaningful empty state with contextual prompts.

## Scope

### 1. Smart global empty state (no milestones + no features)

When `state.milestones.length === 0 && state.features.length === 0`, replace the current generic `DashboardEmptyState` component with a richer orchestrator-aware variant that shows suggestion chips based on what's missing:

- **No Vision defined** → chip: "Define Vision" → `/setup`
- **No Architecture defined** → chip: "Define Architecture" → `/setup`
- **Vision + Architecture exist** → chip: "Start a Ponder" → `/ponder?new=1`
- **Always show** → chip: "Create a Feature directly" → `/features?new=1`

Chips stack in a suggested priority order. Each chip has an icon, a title, and a one-line description of why to do it now.

### 2. Zone-level empty states

Each zone renders a minimal inline empty state when it has no content:

- **AttentionZone** (no escalations, no hitl, no directives): render nothing (already correct — it already hides itself)
- **CurrentZone** (no milestones, no ungrouped features): render a soft prompt "No active work — start a milestone or add a feature" with links to `/milestones` and `/features?new=1`
- **HorizonZone**: remains a null stub (awaiting its own feature)
- **ArchiveZone** (no released milestones): render nothing (appropriate)

### 3. Orchestrator-aware suggestion detection

Read `state` (already loaded in Dashboard) to determine which chips to show:

```ts
const hasVision: boolean  // from api.getVision() — already fetched
const hasArch: boolean    // from api.getArchitecture() — already fetched
```

No new API calls needed — these signals are already available in Dashboard state.

## Out of Scope

- Animation / transitions between states
- Persistent dismissal of suggestions
- HorizonZone implementation (separate feature)

## Acceptance Criteria

1. On a fresh project with no features/milestones, the dashboard shows suggestion chips not a generic button
2. When vision is missing, "Define Vision" chip appears with a link to /setup
3. When architecture is missing, "Define Architecture" chip appears with a link to /setup
4. When both vision and architecture exist, "Start a Ponder" chip appears
5. "Create a Feature directly" chip always appears on the empty state
6. CurrentZone renders a soft prompt when there are no active milestones and no ungrouped features
7. When features/milestones exist, the global empty state is not rendered (existing behavior preserved)
8. No new API calls introduced — chips derive from already-loaded state
