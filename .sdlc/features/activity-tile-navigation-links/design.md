# Design: Activity Tile Navigation Links

## Overview

Add a navigation link to `RunCard` headers so users can jump from any agent run to the entity it targets.

## Component Changes

### 1. New utility: `runTargetRoute(runType, target) → string | null`

Location: `frontend/src/lib/routing.ts` (new file)

```ts
export function runTargetRoute(runType: string, target: string): string | null {
  if (!target) return null
  switch (runType) {
    case 'feature':        return `/features/${target}`
    case 'milestone_uat':
    case 'milestone_prepare':
    case 'milestone_run_wave': return `/milestones/${target}`
    case 'ponder':         return `/ponder/${target}`
    case 'investigation':  return `/investigations/${target}`
    default:               return null
  }
}
```

### 2. `RunCard.tsx` modification

In the header area (line ~130, below the label), add a `<Link>` when `runTargetRoute()` returns non-null:

```
[StatusIcon] [Label]                    [Stop] [Chevron]
             [time · $cost · turns]
             [→ features/slug]   ← NEW: subtle navigation link
```

The link uses:
- `text-[10px] text-primary/70 hover:text-primary hover:underline` for subtle styling
- `ExternalLink` icon from lucide-react (w-2.5 h-2.5) as a visual affordance
- React Router `<Link to={route}>` for client-side navigation
- `onClick={e => e.stopPropagation()}` to prevent triggering the expand/collapse toggle

### 3. No other component changes

Individual tile components (`ToolCallCard`, `RunResultCard`, etc.) remain untouched. The link lives at the run level only.

## Visual Design

See [Mockup](mockup.html) for the visual reference.

## Data Flow

```
RunRecord.run_type + RunRecord.target
        ↓
  runTargetRoute()
        ↓
  string | null
        ↓
  <Link> or nothing
```

No new API calls. No new props threaded to child components.
