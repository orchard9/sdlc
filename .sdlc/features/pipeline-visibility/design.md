# Design: Pipeline Visibility Indicator

## Overview

A persistent horizontal pipeline indicator on the Dashboard that shows the five SDLC workflow stages, highlights the current stage (furthest progress), and lets users click each stage pill to navigate.

The component is pure UI — it derives its state from data already available in `ProjectState` (ponders and milestones) with no new API endpoints needed.

## Component Architecture

### `PipelineIndicator` (`frontend/src/components/PipelineIndicator.tsx`)

A single new component consuming `ProjectState` data (passed as props or obtained via `useProjectState`). It renders five stage pills with arrow connectors.

```
[ Ponder ] → [ Plan ] → [ Commit ] → [ Run Wave ] → [ Ship ]
```

**Props:**

```ts
interface PipelineIndicatorProps {
  ponders: PonderSummary[]
  milestones: MilestoneSummary[]
}
```

Both arrays are already available on `ProjectState` (fetched by `useProjectState` in the Dashboard).

### Stage definitions

```ts
const STAGES = [
  { label: 'Ponder',   href: '/ponder',     tooltip: 'Explore ideas before committing to a plan' },
  { label: 'Plan',     href: '/ponder',     tooltip: 'Review and refine the auto-generated milestone plan' },
  { label: 'Commit',   href: '/milestones', tooltip: 'Commit the plan — creates features in wave order' },
  { label: 'Run Wave', href: '/milestones', tooltip: 'Start a wave — agents build features in parallel' },
  { label: 'Ship',     href: '/milestones', tooltip: 'Features shipped — milestone complete' },
]
```

### Stage determination logic

Greedy: the current stage is the highest stage number reached. Stage numbers are 0-indexed.

```
0 (Ponder):    always true — shown as reached even on a brand-new project
               (highlights Ponder as the entry point, prompts the user to start)
1 (Plan):      ponders.some(p => p.status === 'committed') || milestones.length > 0
2 (Commit):    milestones.length > 0
3 (Run Wave):  milestones.some(m => m.status === 'active' || m.status === 'verifying')
4 (Ship):      milestones.some(m => m.status === 'released')
```

The current stage is the highest index for which the condition is true.

### Visual states per pill

- **Current stage** (index === currentStage): filled primary background, white text, slightly larger or bold label
- **Completed stages** (index < currentStage): filled muted/success background, checkmark icon prefix, dimmed text
- **Future stages** (index > currentStage): outlined ghost pill, muted/foreground text

Arrow connectors (`→`) between pills are rendered as plain text spans with `text-muted-foreground/50`.

### Tooltip implementation

Native HTML `title` attribute is sufficient for v1 (no shadcn/ui dependency needed for a simple tooltip). Each pill anchor element gets `title={stage.tooltip}`.

If the project already has a shadcn `Tooltip` component installed, prefer it — check `frontend/src/components/ui/` for an existing `tooltip.tsx`. If not present, use `title`.

### Layout

- Compact: ~48–60px tall
- Flex row, gap between pills
- Pills are `<Link>` elements from `react-router-dom` wrapping a styled `<span>`
- Full-width container, pills distributed with `flex gap-1` or `gap-2`

## Placement

### Dashboard (`frontend/src/pages/Dashboard.tsx`)

Insert `PipelineIndicator` immediately after the Project Overview block and before the Stats bar:

```tsx
{/* Project Overview */}
<div className="mb-6">…</div>

{/* Pipeline Indicator */}
<PipelineIndicator ponders={state.ponders ?? []} milestones={state.milestones} />

{/* Stats bar */}
<div className="flex items-center gap-4 mb-6 …">…</div>
```

`state.ponders` may not yet be on `ProjectState`. If it is not, use the API directly inside the component via `useEffect` + `api.getPonders()`. Prefer adding `ponders` to `ProjectState` server response if feasible; otherwise the component fetches independently.

## Data Availability

`ProjectState` already includes `milestones: MilestoneSummary[]`. For ponders, check whether `GET /api/project/state` returns ponders. If not, the component calls `GET /api/ponders` (already used by `PonderPage`) and caches locally with a simple `useState` + `useEffect`. This is a one-time load with no polling needed.

## Styling

Use Tailwind utility classes consistent with the existing codebase:

- Filled current: `bg-primary text-primary-foreground`
- Filled completed: `bg-muted text-muted-foreground`
- Ghost future: `border border-border text-muted-foreground/70`
- Pill shape: `rounded-full px-3 py-1 text-xs font-medium`
- Link hover: `hover:opacity-80 transition-opacity`

## ASCII Wireframe

```
┌──────────────────────────────────────────────────────────────────┐
│  ✓ Ponder  →  ✓ Plan  →  ● Commit  →  ○ Run Wave  →  ○ Ship    │
└──────────────────────────────────────────────────────────────────┘
  [muted]       [muted]    [primary]    [ghost]        [ghost]
```

## Out of Scope

- Per-milestone pipeline state
- Animation between stages
- Embedding on every page
- New backend API endpoints
