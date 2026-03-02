# Spec: Dashboard Empty State Redesign

## Problem

The Dashboard's first-run experience teaches the wrong mental model. A new user sees:
1. An amber warning banner: "Project setup is incomplete — agents won't have enough context to work with."
2. A stats bar with all zeros.
3. Nothing actionable.

The user's first emotion is "I broke something." Their first action is to click "Go to Setup" and fill in a Vision document — formal, intimidating, and disconnected from the creative act they came here to do.

## Solution

Replace the empty state with an identity-forward welcome that communicates what the tool does and points directly to the right first action.

## Implementation

### 1. Remove the Amber Warning Banner

File: `frontend/src/pages/DashboardPage.tsx` (or wherever the setup-incomplete banner is rendered)

- Find the component/section that renders the "Project setup is incomplete" amber/yellow warning banner.
- Remove it entirely from the Dashboard page. It should not appear on first load.
- The Setup wizard remains accessible from the Settings or sidebar navigation — it is not deleted, just not forced on users.

### 2. Identity Sentence + New Ponder CTA (Empty State)

Condition: No milestones AND no features exist.

Replace the empty stats area with:

```
SDLC turns ideas into shipped software.
Describe what you're building — agents will build it in parallel waves.

[ New Ponder ]
```

**Component:** Create `DashboardEmptyState.tsx` (or inline in `DashboardPage.tsx`).

**Styling:** Centered card or full-width section. Muted/secondary text for the tagline. Primary button style for "New Ponder."

**"New Ponder" button behavior:** Navigates to `/ponder` with a query param `?new=1` to open the new ponder form immediately (or just navigate to `/ponder` — the Ponder page already has a "New idea" button that can be auto-focused).

### 3. Rename "Setup Incomplete" → "Agents Need More Context"

Search for all instances of the "setup incomplete" messaging across the frontend codebase and replace:

- "Project setup is incomplete" → "Agents need more context"
- "Setup incomplete" → "Agents need more context"
- "setup is incomplete" → "agents need more context"
- Any related CTA that says "Go to Setup →" can remain — the destination page label can stay as "Setup" since it's the page name.

Files likely affected:
- `frontend/src/pages/DashboardPage.tsx`
- `frontend/src/pages/SetupPage.tsx` (any self-referential banners)
- Any shared component that renders setup status

## Acceptance Criteria

- [ ] A new project (no milestones, no features) shows the identity sentence + "New Ponder" CTA, not an amber warning
- [ ] The amber "setup incomplete" warning banner is not visible on the Dashboard for new projects
- [ ] "New Ponder" button navigates to the Ponder page
- [ ] All instances of "setup incomplete" in the UI read "agents need more context" (case-insensitive search)
- [ ] Projects with milestones/features show normal dashboard content (not the empty state)

## Out of Scope

- Changing the Setup page itself (addressed in `ponder-first-onboarding`)
- Pipeline indicator (addressed in `pipeline-visibility`)
- Milestone cards on Dashboard (addressed in `dashboard-milestone-wave`)
