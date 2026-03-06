# Design: WorkspaceShell component

## Overview

`WorkspaceShell` is a pure layout component. It renders a responsive two-pane shell and delegates content entirely to its props. No data fetching, no domain knowledge.

## Component Interface

```tsx
// frontend/src/components/layout/WorkspaceShell.tsx

interface WorkspaceShellProps {
  /** Content for the fixed-width left list pane. */
  listPane: React.ReactNode
  /** Content for the flexible right detail pane. */
  detailPane: React.ReactNode
  /** When true (a detail slug is active), the right pane is visible and the left pane hides on mobile. */
  showDetail: boolean
  /** Tailwind width class for the left pane. Defaults to "w-72". */
  listWidth?: string
}

export function WorkspaceShell({ listPane, detailPane, showDetail, listWidth = 'w-72' }: WorkspaceShellProps) {
  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 flex min-h-0">
        {/* Left pane */}
        <div className={cn(
          listWidth,
          'shrink-0 border-r border-border flex flex-col bg-card',
          showDetail ? 'hidden lg:flex' : 'flex',
        )}>
          {listPane}
        </div>

        {/* Right pane */}
        <div className={cn(
          'flex-1 min-w-0',
          showDetail ? 'flex flex-col' : 'hidden lg:flex lg:flex-col',
        )}>
          {detailPane}
        </div>
      </div>
    </div>
  )
}
```

## Layout Behavior

### Desktop (lg+)
Both panes are always visible side-by-side. The left pane has a fixed width set via `listWidth`. The right pane fills remaining width with `flex-1 min-w-0`.

### Mobile (< lg)
- When `showDetail` is false: only the left list pane is shown.
- When `showDetail` is true: only the right detail pane is shown.
This matches the existing behavior in all four pages.

## Consuming Pages — Before/After

### PonderPage (example — others follow the same pattern)

**Before:**
```tsx
<div className="h-full flex flex-col overflow-hidden">
  <div className="flex-1 flex min-h-0">
    <div className={cn('w-72 shrink-0 border-r border-border flex flex-col bg-card', showMobileDetail ? 'hidden lg:flex' : 'flex')}>
      {/* left pane content */}
    </div>
    <div className={cn('flex-1 min-w-0', showMobileDetail ? 'flex flex-col' : 'hidden lg:flex lg:flex-col')}>
      {/* right pane content */}
    </div>
  </div>
</div>
```

**After:**
```tsx
<WorkspaceShell
  showDetail={showMobileDetail}
  listPane={/* left pane content */}
  detailPane={/* right pane content */}
/>
```

## File Locations

| File | Action |
|---|---|
| `frontend/src/components/layout/WorkspaceShell.tsx` | Create (new file) |
| `frontend/src/pages/PonderPage.tsx` | Refactor outer shell to use WorkspaceShell |
| `frontend/src/pages/EvolvePage.tsx` | Refactor outer shell to use WorkspaceShell |
| `frontend/src/pages/InvestigationPage.tsx` | Refactor outer shell to use WorkspaceShell |
| `frontend/src/pages/GuidelinePage.tsx` | Refactor outer shell to use WorkspaceShell |

## No Mockup Required

This is a structural refactor with no visual change. The output is identical to the current UI. No mockup HTML is needed.
