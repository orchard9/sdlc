# Design: Ponder Layout — CONTEXT and ARTIFACTS Resizable Panels, Mobile Chat/Files/Team Tabs

## Overview

This design covers the structural changes to `PonderPage.tsx` and `DialoguePanel.tsx` required to
implement resizable desktop panels and a mobile tab bar. No server-side changes are needed.

## Desktop Layout Wireframe

```
┌──────────────┬────────────────────────────────────────┬───────────────────┐
│  Entry List  │ [Context Panel]  │  Chat (DialoguePanel)│  Workspace Panel  │
│  w-72        │  ~200px or 32px  │  flex-1 (min-w-0)   │  ~256px resizable │
│  fixed       │  collapsible     │                      │                   │
│              │  ◀ toggle btn    │ [no TeamRow/Orient]  │  drag handle ↔   │
│  EntryRow    │  · slug          │ [sessions / input]   │  WorkspacePanel   │
│  ...         │  · status        │                      │                   │
│              │  · TeamRow       │                      │                   │
│              │  · OrientStrip   │                      │                   │
└──────────────┴────────────────────────────────────────┴───────────────────┘
```

## Mobile Layout Wireframe

```
┌──────────────────────────────────────────┐
│  [← back]  Entry Title      [status][⋮]  │  ← header (unchanged minus Files button)
├──────────────────────────────────────────┤
│                                          │
│   Active tab content (full height)       │
│   Chat   → DialoguePanel (with InputBar) │
│   Files  → WorkspacePanel               │
│   Team   → TeamRow + OrientationStrip   │
│                                          │
├──────────────────────────────────────────┤
│  💬 Chat  │  📁 Files (3)  │  👥 Team   │  ← tab bar
└──────────────────────────────────────────┘
```

## Component Architecture

### `EntryDetailPane` (in `PonderPage.tsx`)

Gains two pieces of local state and one localStorage hook:

```typescript
// Desktop context panel
const [contextOpen, setContextOpen] = useState(() =>
  localStorage.getItem('ponder_context_open') !== 'false'
)
// Desktop workspace width
const [workspaceWidth, setWorkspaceWidth] = useState(() => {
  const stored = localStorage.getItem('ponder_workspace_width')
  return stored ? parseInt(stored, 10) : 256
})
// Mobile active tab
type MobileTab = 'chat' | 'files' | 'team'
const [mobileTab, setMobileTab] = useState<MobileTab>('chat')
```

#### Desktop Render Tree

```
<div className="h-full flex flex-col min-h-0">
  {/* Header row (unchanged) */}
  <div className="shrink-0 ...">...</div>

  {/* Body: context + chat + workspace */}
  <div className="flex-1 flex min-h-0 hidden md:flex">

    {/* Context panel */}
    <ContextPanel
      open={contextOpen}
      onToggle={() => { ... persist to localStorage }}
      slug={entry.slug}
      status={entry.status}
      team={entry.team}
      orientation={orientation}
    />

    {/* Chat */}
    <div className="flex-1 min-w-0 min-h-0">
      <DialoguePanel ... hideContextHeader />
    </div>

    {/* Drag handle */}
    <ResizeDivider
      onWidthChange={(w) => { setWorkspaceWidth(w); persist }}
      minWidth={160}
      maxFraction={0.5}
    />

    {/* Workspace */}
    <div style={{ width: workspaceWidth }} className="shrink-0 border-l border-border flex flex-col min-h-0">
      <WorkspacePanel artifacts={entry.artifacts} />
    </div>
  </div>

  {/* Mobile tab content */}
  <div className="flex-1 flex flex-col min-h-0 md:hidden">
    {mobileTab === 'chat' && <DialoguePanel ... />}
    {mobileTab === 'files' && <WorkspacePanel artifacts={entry.artifacts} />}
    {mobileTab === 'team' && <TeamContextPanel team={entry.team} orientation={orientation} />}
  </div>

  {/* Mobile tab bar */}
  <div className="shrink-0 md:hidden border-t border-border flex">
    <MobileTabButton tab="chat" active={mobileTab === 'chat'} onClick={() => setMobileTab('chat')} />
    <MobileTabButton tab="files" active={mobileTab === 'files'} badge={artifactCount} onClick={() => setMobileTab('files')} />
    <MobileTabButton tab="team" active={mobileTab === 'team'} onClick={() => setMobileTab('team')} />
  </div>
</div>
```

### New: `ContextPanel` (inline component within PonderPage.tsx)

Props:
```typescript
interface ContextPanelProps {
  open: boolean
  onToggle: () => void
  slug: string
  status: PonderStatus
  team: PonderTeamMember[]
  orientation: string | null
}
```

Collapsed state (~32 px wide): shows only the toggle icon button (`ChevronRight`).
Expanded state (~200 px wide): shows toggle button, slug, status badge, `TeamRow`, `OrientationStrip`.
Uses `transition-all duration-200` for smooth open/close.

### New: `ResizeDivider` (inline component within PonderPage.tsx)

A `<div>` with:
- Width: 5 px, `cursor-col-resize`
- `onMouseDown` starts a drag listener on `document`
- During drag: computes `parentWidth - mouseX` to derive new workspace width, clamped to [minWidth, maxWidth]
- `onMouseUp` / `onMouseLeave` on document ends drag
- Visual: thin border, brightens on hover/active

```typescript
function ResizeDivider({ onWidthChange, minWidth, maxFraction }: {
  onWidthChange: (w: number) => void
  minWidth: number
  maxFraction: number
}) {
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault()
    const startX = e.clientX
    const parent = e.currentTarget.parentElement!
    const onMove = (ev: MouseEvent) => {
      const parentRect = parent.getBoundingClientRect()
      const newWidth = parentRect.right - ev.clientX
      const max = parentRect.width * maxFraction
      onWidthChange(Math.max(minWidth, Math.min(max, newWidth)))
    }
    const onUp = () => {
      document.removeEventListener('mousemove', onMove)
      document.removeEventListener('mouseup', onUp)
    }
    document.addEventListener('mousemove', onMove)
    document.addEventListener('mouseup', onUp)
  }
  return (
    <div
      onMouseDown={handleMouseDown}
      className="w-1.5 shrink-0 cursor-col-resize bg-border/0 hover:bg-primary/20 active:bg-primary/30 transition-colors"
    />
  )
}
```

### `DialoguePanel` changes

Add optional prop `hideContextHeader?: boolean`. When true, skip rendering `<TeamRow>` and
`<OrientationStrip>` at the top of the panel:

```typescript
// Before (always rendered)
{entry.team.length > 0 && <TeamRow ... />}
<OrientationStrip ... />

// After
{!hideContextHeader && entry.team.length > 0 && <TeamRow ... />}
{!hideContextHeader && <OrientationStrip ... />}
```

On mobile, `hideContextHeader` is not passed (defaults false), so the mobile Chat tab still shows
team/orientation inline.

### Mobile Tab Bar

Three equal-width buttons, each with an icon + label. The Files button shows an artifact count badge
when `artifactCount > 0`.

```typescript
function MobileTabButton({ tab, active, badge, onClick }: {
  tab: MobileTab; active: boolean; badge?: number; onClick: () => void
}) { ... }
```

Uses `MessageSquare`, `Files`, `Users` icons from lucide-react.

### `TeamContextPanel` (mobile Team tab, inline component)

Simple div rendering `TeamRow` and `OrientationStrip` with some padding. No additional logic.

## State Persistence

| Key | Type | Default | Scope |
|---|---|---|---|
| `ponder_context_open` | `'true'`/`'false'` | `'true'` | Desktop context panel |
| `ponder_workspace_width` | number string (px) | `'256'` | Desktop workspace width |

Both read in `useState` initializer and written in `useEffect` / event handler when changed.

## TypeScript / Lint Notes

- `hideContextHeader` prop must be added to `DialoguePanel`'s `Props` interface.
- All new inline components are typed; no `any` usage.
- `ResizeDivider` uses `React.MouseEvent<HTMLDivElement>`.
- No new dependencies required — lucide-react and existing utilities suffice.

## Files Changed

| File | Type of change |
|---|---|
| `frontend/src/pages/PonderPage.tsx` | Main changes — ContextPanel, ResizeDivider, mobile tab bar, state |
| `frontend/src/components/ponder/DialoguePanel.tsx` | Add `hideContextHeader` prop |
