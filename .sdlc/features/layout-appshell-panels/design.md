# Design: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Component Map

```
AppShell
├── Sidebar (w-14 collapsed | w-56 expanded)  ← NEW: collapsible
│   ├── Header (logo + toggle button)
│   ├── nav groups
│   │   └── NavItem (icon + optional label + optional tooltip)
│   └── BottomActions (icon + optional label/kbd)
├── main content (flex-1, min-w-0)
├── AgentPanel (inline style width, resizable)  ← NEW: drag-to-resize
│   ├── ResizeHandle (4px left border drag target)  ← NEW
│   └── panel content (unchanged)
└── PanelRightOpen toggle button (unchanged)
```

## Sidebar Collapse Design

### State

```tsx
// In Sidebar component (or lifted to AppShell and passed as prop)
const [collapsed, setCollapsed] = useState<boolean>(() => {
  return localStorage.getItem('sdlc:sidebar-collapsed') === 'true'
})

// Persist on toggle
function toggleCollapsed() {
  const next = !collapsed
  setCollapsed(next)
  localStorage.setItem('sdlc:sidebar-collapsed', String(next))
}
```

State is local to the `Sidebar` component — no context lifting needed, since AppShell does not need to know the sidebar width (content area fills remaining flex space automatically).

### Width Transition

```tsx
<aside
  className={cn(
    'h-full bg-card border-r border-border flex flex-col transition-[width] duration-200 ease-in-out shrink-0',
    collapsed ? 'w-14' : 'w-56'
  )}
>
```

`transition-[width]` limits the transition to width only (avoids unexpected transitions on other properties). `shrink-0` prevents flexbox from squashing it.

### Header

```tsx
<div className="px-3 py-5 border-b border-border flex items-center justify-between">
  {/* Logo — always visible */}
  {!collapsed && (
    <div>
      <h1 className="text-lg font-semibold tracking-tight">SDLC</h1>
      <p className="text-xs text-muted-foreground mt-0.5">Feature Lifecycle</p>
    </div>
  )}
  {collapsed && <div className="w-5" />}  {/* spacer to keep toggle right-aligned */}
  <button
    onClick={toggleCollapsed}
    className="p-1 rounded hover:bg-accent transition-colors text-muted-foreground hover:text-foreground"
    aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
  >
    {collapsed ? <ChevronsRight className="w-4 h-4" /> : <ChevronsLeft className="w-4 h-4" />}
  </button>
</div>
```

### Nav Items (collapsed vs expanded)

Each nav item conditionally renders its label and group header:

```tsx
// Group label — hidden when collapsed
{!collapsed && (
  <p className="px-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/40">
    {group.label}
  </p>
)}

// Nav item — icon always shown, label conditional, tooltip when collapsed
<Tooltip>
  <TooltipTrigger asChild>
    <Link
      key={path}
      to={path}
      onClick={onNavigate}
      className={cn(
        'flex items-center gap-2.5 rounded-lg text-sm transition-colors',
        collapsed ? 'px-3 py-2.5 justify-center' : 'px-3 py-2',
        active
          ? 'bg-accent text-accent-foreground font-medium'
          : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'
      )}
    >
      <Icon className="w-4 h-4 shrink-0" />
      {!collapsed && label}
    </Link>
  </TooltipTrigger>
  {collapsed && (
    <TooltipContent side="right" className="text-xs">
      {label}
    </TooltipContent>
  )}
</Tooltip>
```

Wrap the `nav` block in `<TooltipProvider>` (or rely on the app-level TooltipProvider if one exists).

### Bottom Actions (collapsed)

```tsx
<button ...>
  <Zap className="w-4 h-4" />
  {!collapsed && (
    <>
      <span className="flex-1 text-left">Fix Right Away</span>
      <kbd ...>⌘⇧F</kbd>
    </>
  )}
</button>
```

When collapsed, the button is `justify-center` and shows icon only.

## AgentPanel Resize Design

### State (in AgentPanel)

```tsx
const MIN_WIDTH = 200
const MAX_WIDTH = 520
const DEFAULT_WIDTH = 288  // current w-72

const [width, setWidth] = useState<number>(() => {
  const stored = localStorage.getItem('sdlc:agent-panel-width')
  const parsed = stored ? parseInt(stored, 10) : NaN
  return isNaN(parsed) ? DEFAULT_WIDTH : Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, parsed))
})

// Persist on drag end
function persistWidth(w: number) {
  localStorage.setItem('sdlc:agent-panel-width', String(w))
}
```

### Resize Handle

A 4 px absolute element on the left edge of the panel. Uses pointer capture for reliable drag outside the element bounds:

```tsx
function ResizeHandle({ onResize }: { onResize: (delta: number) => void }) {
  const handleRef = useRef<HTMLDivElement>(null)

  function onPointerDown(e: React.PointerEvent) {
    e.preventDefault()
    handleRef.current?.setPointerCapture(e.pointerId)
    const startX = e.clientX

    function onMove(e: PointerEvent) {
      // Panel grows leftward: moving left increases width
      onResize(startX - e.clientX)
    }

    function onUp() {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
    }

    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
  }

  return (
    <div
      ref={handleRef}
      onPointerDown={onPointerDown}
      className="absolute left-0 inset-y-0 w-1 cursor-col-resize hover:bg-accent/60 transition-colors z-10"
      aria-hidden="true"
    />
  )
}
```

### AgentPanel integration

```tsx
export function AgentPanel() {
  const { panelOpen, setPanelOpen } = useAgentRuns()
  const [fullscreen, setFullscreen] = useState(false)
  const [width, setWidth] = useState<number>(() => { /* localStorage init */ })
  const baseWidth = useRef(width)

  function handleResize(delta: number) {
    const next = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, baseWidth.current + delta))
    setWidth(next)
  }

  // On drag end, persist. Use onPointerUp on the handle to finalize.
  // Alternatively, persist in the onUp closure above.

  if (!panelOpen) return null

  return (
    <>
      <aside
        className="hidden md:flex flex-col border-l border-border bg-background shrink-0 overflow-hidden relative"
        style={{ width: `${width}px` }}
      >
        <ResizeHandle
          onResize={(delta) => {
            const next = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, baseWidth.current + delta))
            setWidth(next)
          }}
          onResizeEnd={() => {
            baseWidth.current = width
            localStorage.setItem('sdlc:agent-panel-width', String(width))
          }}
        />
        {/* Header and content unchanged */}
      </aside>
      <FullscreenModal ...>
        {/* No ResizeHandle here */}
      </FullscreenModal>
    </>
  )
}
```

The `baseWidth` ref captures the width at the start of each drag so incremental deltas compute correctly when a new drag begins.

## ASCII Wireframes

### Expanded sidebar + AgentPanel (normal state)

```
┌──────────────────┬──────────────────────────────────┬──────────────┐
│ SDLC    [«]      │                                  │ Agent Activity│
│ Feature Lifecycle│         Main Content              │  [↗] [×]     │
│──────────────────│                                  │──────────────│
│ work             │                                  │              │
│  Dashboard       │                                  │  Run cards   │
│  Milestones      │                                  │              │
│  Features        │                                  │              │
│ plan             │                                  │              │
│  Feedback        │                                  │              │
│  Ponder          │                                  │              │
│  ...             │                                  │              │
│──────────────────│                                  │              │
│ [⚡] Fix Right   │                                  │              │
│ [🔍] Search  ⌘K  │                                  │              │
└──────────────────┴──────────────────────────────────┴──────────────┘
  w-56                  flex-1                             w-72 (default)
```

### Collapsed sidebar (icon rail)

```
┌─────┬──────────────────────────────────────────┬──────────────┐
│[»]  │                                          │ Agent Activity│
│     │          Main Content                    │  [↗] [×]     │
│─────│  (more horizontal space available)       │──────────────│
│ [■] │                                          │              │
│ [M] │                                          │  Run cards   │
│ [F] │                                          │              │
│ [💡]│                                          │              │
│ [🔬]│                                          │              │
│ [📈]│                                          │              │
│ [🔧]│                                          │              │
│─────│                                          │              │
│ [⚡]│                                          │              │
│ [🔍]│                                          │              │
└─────┴──────────────────────────────────────────┴──────────────┘
  w-14       flex-1 (wider)                         w-72 (default)
```

### AgentPanel drag resize

```
                                          ◄── drag handle (left edge)
┌──────────────────┬──────────────────────────────┬─────────────────────┐
│ SDLC    [«]      │                              ▌│  Agent Activity     │
│ Feature Lifecycle│     Main Content             ▌│  [↗] [×]           │
│                  │                              ▌│                     │
│                  │                              ▌│  wider panel now    │
│                  │                              ▌│  shows more text    │
│                  │                              ▌│                     │
│                  │                              ▌│                     │
└──────────────────┴──────────────────────────────┴─────────────────────┘
                                        cursor: col-resize on ▌
```

## File Changes

| File | Change |
|---|---|
| `frontend/src/components/layout/Sidebar.tsx` | Add `collapsed` state, toggle button (`ChevronsLeft`/`ChevronsRight`), conditional label rendering, Tooltip on collapsed items |
| `frontend/src/components/layout/AgentPanel.tsx` | Add `width` state, `ResizeHandle` sub-component, inline style width, remove `w-72` class |
| `frontend/src/components/layout/AppShell.tsx` | No structural changes needed — sidebar width is self-contained; agent panel width is inline |

## Dependencies

- `ChevronsLeft`, `ChevronsRight` from `lucide-react` (already installed)
- `Tooltip`, `TooltipContent`, `TooltipProvider`, `TooltipTrigger` from `@/components/ui/tooltip` (shadcn/ui — verify installed; add if missing)
- No new npm packages required

## localStorage Keys

| Key | Type | Default | Purpose |
|---|---|---|---|
| `sdlc:sidebar-collapsed` | `"true" \| "false"` | `"false"` | Sidebar collapse preference |
| `sdlc:agent-panel-width` | numeric string | `"288"` | AgentPanel width in px |

## Edge Cases

- **Tooltip package absent:** If `@/components/ui/tooltip` does not exist, use a native `title` attribute as fallback and add a task to install shadcn tooltip.
- **Width out of range on load:** Clamp on read (`Math.min(MAX, Math.max(MIN, parsed))`).
- **AgentPanel closed:** When `panelOpen` is false, `AgentPanel` returns `null` — width state is preserved in memory for when it re-opens; `localStorage` already has the last persisted value.
- **Fullscreen modal:** `ResizeHandle` is not rendered inside `FullscreenModal`. The modal fills the viewport independently.
- **Mobile:** Sidebar collapse toggle is desktop-only concern. The sidebar on mobile is the slide-in overlay — its behavior is driven by `sidebarOpen` in `AppShell`, not the `collapsed` state. The `collapsed` state only affects the desktop layout (`md:static` context).
