# Tasks: Dashboard Empty State Redesign

## T1 — Create `DashboardEmptyState` component

**File:** `frontend/src/components/dashboard/DashboardEmptyState.tsx`

Create the directory `frontend/src/components/dashboard/` if it does not exist. Create the component
with the identity headline, tagline, and a "New Ponder" primary button that navigates to `/ponder`.
Use `useNavigate` from `react-router-dom`. Use the `Lightbulb` icon from `lucide-react`.

## T2 — Remove amber "setup incomplete" banner from Dashboard

**File:** `frontend/src/pages/Dashboard.tsx`

Remove:
- The `setupIncomplete` state variable and its `useState` call
- The `hasCheckedSetup` ref and its `useRef` call
- The `useEffect` block that calls `api.getConfig`, `api.getVision`, `api.getArchitecture`, and `api.getProjectAgents` for setup checking
- The rendered amber banner JSX block (the `{setupIncomplete && (...)}` section)
- The `AgentDefinition` import from `@/lib/types` if it is no longer used after removing the above
- The `Key` icon import from `lucide-react` if it is no longer used

Verify the remaining imports and state variables are still used to avoid lint errors.

## T3 — Replace empty state with `DashboardEmptyState`

**File:** `frontend/src/pages/Dashboard.tsx`

Replace the existing empty-state block at the bottom of the render:

```tsx
{state.features.length === 0 && (
  <div className="text-center py-16">
    <p className="text-muted-foreground text-sm">No features yet.</p>
    <p className="text-xs text-muted-foreground mt-1">
      Use <code className="text-primary">sdlc feature create</code> to get started.
    </p>
  </div>
)}
```

With:

```tsx
{state.milestones.length === 0 && state.features.length === 0 && (
  <DashboardEmptyState />
)}
```

Import `DashboardEmptyState` at the top of `Dashboard.tsx`.

## T4 — Rename "setup incomplete" strings across the frontend

Search the entire `frontend/src/` tree for all occurrences of "setup incomplete" (case-insensitive)
and replace with "agents need more context" (preserving surrounding casing conventions):

- "Project setup is incomplete" → "Agents need more context"
- "Setup incomplete" → "Agents need more context"
- "setup is incomplete" → "agents need more context"

Primary file to check: `frontend/src/pages/SetupPage.tsx`. Also check any shared components, hook
files, or other pages that may reference setup status text.
