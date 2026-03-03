# Design: Orchestrator-aware empty states

## Overview

Two components get redesigned: the global `DashboardEmptyState` and the `CurrentZone` empty state. All logic derives from data already loaded in `Dashboard.tsx` — no new API calls.

## Component Architecture

```
Dashboard.tsx
├── DashboardEmptyState (global — shown when 0 features + 0 milestones)
│   └── SuggestionChip[] (ordered by priority)
└── CurrentZone (zone-level — shown when no milestones + no ungrouped features)
    └── CurrentZoneEmpty (inline soft prompt)
```

## Global Empty State: DashboardEmptyState

### Chip priority logic

```
chips = []

if !hasVision → chips.push({ label: "Define Vision", desc: "...", to: "/setup", icon: Target })
if !hasArch   → chips.push({ label: "Define Architecture", desc: "...", to: "/setup", icon: Layers })
if hasVision && hasArch → chips.push({ label: "Start a Ponder", desc: "...", to: "/ponder?new=1", icon: Lightbulb })
always         → chips.push({ label: "Create a Feature directly", desc: "...", to: "/features?new=1", icon: Plus })
```

Props needed from Dashboard:
- `hasVision: boolean`
- `hasArch: boolean`

### Visual design

```
┌────────────────────────────────────────┐
│                                        │
│   ● SDLC turns ideas into shipped      │
│     software.                          │
│                                        │
│   Where do you want to start?          │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │ 🎯  Define Vision                │  │
│  │     Agents use this to stay      │  │
│  │     aligned on every decision.   │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ 🏗  Define Architecture          │  │
│  │     Gives agents the system map  │  │
│  │     before they write code.      │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ ➕  Create a Feature directly    │  │
│  │     Skip planning, go straight   │  │
│  │     to implementation.           │  │
│  └──────────────────────────────────┘  │
│                                        │
└────────────────────────────────────────┘
```

Each chip is a `Link` (react-router-dom) styled as a bordered card with:
- Icon (16px, muted-foreground)
- Bold label (sm, font-medium)
- One-line description (xs, muted-foreground)
- Hover: `bg-accent` background transition

### Props interface

```ts
interface DashboardEmptyStateProps {
  hasVision: boolean
  hasArch: boolean
}
```

## Zone-level empty: CurrentZone

When `milestones` is empty AND `ungrouped` is empty, show a soft inline prompt:

```
┌────────────────────────────────────────┐
│  No active work.                       │
│  Start a milestone or add a feature.   │
│                     [Milestones] [+Feature] │
└────────────────────────────────────────┘
```

Rendered as a card matching the existing card style (`bg-card border border-border rounded-xl p-4`) with muted text and two small link buttons.

## Data flow changes in Dashboard.tsx

`DashboardEmptyState` currently takes no props. After this change it takes `hasVision` and `hasArch`. These booleans are already derived from the existing `missingVisionOrArch` effect — we expand that to track them separately:

```ts
const [hasVision, setHasVision] = useState(false)
const [hasArch, setHasArch] = useState(false)

useEffect(() => {
  Promise.all([api.getVision(), api.getArchitecture()]).then(([v, a]) => {
    setHasVision(!!v?.exists)
    setHasArch(!!a?.exists)
    setMissingVisionOrArch(!v?.exists || !a?.exists)
  })
}, [])
```

Then pass to `DashboardEmptyState`:
```tsx
<DashboardEmptyState hasVision={hasVision} hasArch={hasArch} />
```

## Files changed

| File | Change |
|---|---|
| `frontend/src/components/dashboard/DashboardEmptyState.tsx` | Full rewrite — chip-based layout, accepts `hasVision`/`hasArch` props |
| `frontend/src/components/dashboard/CurrentZone.tsx` | Add `CurrentZoneEmpty` component, render when no content |
| `frontend/src/pages/Dashboard.tsx` | Split `missingVisionOrArch` into `hasVision`/`hasArch`, pass props to `DashboardEmptyState` |
