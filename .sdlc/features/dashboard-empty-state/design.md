# Design: Dashboard Empty State Redesign

## Overview

Three targeted changes to the Dashboard page (`frontend/src/pages/Dashboard.tsx`):

1. Remove the amber "setup incomplete" warning banner
2. Replace the generic "No features yet" empty state with an identity-forward welcome + "New Ponder" CTA
3. Rename all "setup incomplete" message strings to "agents need more context"

No new routes, no API changes, no backend changes — pure frontend.

## Component Map

```
Dashboard.tsx
├── setupIncomplete banner        ← REMOVE entirely
├── empty state (features.length === 0)  ← REPLACE with DashboardEmptyState
└── (rest unchanged)

DashboardEmptyState.tsx (new component)
├── Identity headline
├── Tagline
└── "New Ponder" button → /ponder
```

## ASCII Wireframe — Empty State

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│  [project name]                                    v1.0.0   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                                                      │   │
│  │        SDLC turns ideas into shipped software.       │   │
│  │   Describe what you're building — agents will        │   │
│  │   build it in parallel waves.                        │   │
│  │                                                      │   │
│  │              [ + New Ponder ]                        │   │
│  │                                                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

The empty state appears when `state.milestones.length === 0 && state.features.length === 0`.
If either has content, the normal dashboard content renders.

## Detailed Changes

### 1. Remove amber banner

The `setupIncomplete` state variable, the `hasCheckedSetup` ref, and the `useEffect` that checks
`api.getConfig / getVision / getArchitecture / getProjectAgents` can all be removed along with the
rendered banner JSX block (lines 201–211). The imports `useRef` and `Key` (if unused elsewhere) may
also be removed to avoid lint errors.

### 2. New `DashboardEmptyState` component

Create `frontend/src/components/dashboard/DashboardEmptyState.tsx`:

```tsx
import { useNavigate } from 'react-router-dom'
import { Lightbulb } from 'lucide-react'

export function DashboardEmptyState() {
  const navigate = useNavigate()
  return (
    <div className="flex flex-col items-center justify-center py-20 text-center">
      <div className="max-w-md space-y-4">
        <p className="text-lg font-semibold leading-snug">
          SDLC turns ideas into shipped software.
        </p>
        <p className="text-sm text-muted-foreground">
          Describe what you're building — agents will build it in parallel waves.
        </p>
        <button
          onClick={() => navigate('/ponder')}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
        >
          <Lightbulb className="w-4 h-4" />
          New Ponder
        </button>
      </div>
    </div>
  )
}
```

### 3. Swap existing empty state in Dashboard.tsx

Replace the current bottom-of-page empty state block:

```tsx
{state.features.length === 0 && (
  <div className="text-center py-16">...</div>
)}
```

With the condition-guarded component:

```tsx
{state.milestones.length === 0 && state.features.length === 0 && (
  <DashboardEmptyState />
)}
```

### 4. Rename "setup incomplete" strings

Search for all occurrences across the frontend and replace:

| Find | Replace |
|---|---|
| `Project setup is incomplete` | `Agents need more context` |
| `Setup incomplete` | `Agents need more context` |
| `setup is incomplete` | `agents need more context` |
| `setup incomplete` (case-insensitive) | `agents need more context` |

Files to check (from spec):
- `frontend/src/pages/Dashboard.tsx` (banner being removed — handled by step 1)
- `frontend/src/pages/SetupPage.tsx`
- Any shared components referencing setup status

## Acceptance Criteria Mapping

| Criterion | Implementation |
|---|---|
| New project shows identity + CTA | `DashboardEmptyState` renders when 0 milestones + 0 features |
| Amber banner gone | `setupIncomplete` logic + JSX removed |
| "New Ponder" navigates to Ponder page | `navigate('/ponder')` in component |
| "setup incomplete" → "agents need more context" | String search + replace across frontend |
| Projects with content show normal view | Empty state condition guards on 0 milestones AND 0 features |

## Files Changed

- `frontend/src/pages/Dashboard.tsx` — remove banner, replace empty state, remove unused imports
- `frontend/src/components/dashboard/DashboardEmptyState.tsx` — new component (create directory if needed)
- `frontend/src/pages/SetupPage.tsx` — rename any "setup incomplete" strings
- Any other frontend files where "setup incomplete" text appears
