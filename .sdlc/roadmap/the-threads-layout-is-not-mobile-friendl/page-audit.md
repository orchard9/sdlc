# Mobile Layout Audit

## Status by Page

| Page | Mobile Status | Key Issues |
|---|---|---|
| **ThreadsPage** | 🔴 BROKEN | Left pane always shows at 280px — no hide/show logic. Both panes side-by-side on mobile. |
| **PonderPage** | ✅ GOOD | Full treatment: list/detail routing, tab bar (Chat/Files/Team), back button. **Reference impl.** |
| **EvolvePage** | 🟡 PARTIAL | Has list/detail switching + back button ✓. Files panel uses bottom sheet (not tab bar). |
| **GuidelinePage** | 🟡 PARTIAL | Same as EvolvePage. |
| **InvestigationPage** | 🟡 PARTIAL | Same as EvolvePage. |
| **ToolsPage** | 🟡 PARTIAL | Has list hiding when tool selected ✓. No back button ✗. Auto-selects first tool so detail always shows on mobile ✗. |

## Root Cause: ThreadsPage

```tsx
// BROKEN: md:flex does nothing — element is already flex. No hidden class.
<div className="w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]">

// NEEDS:
<div className={cn(
  'w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden',
  slug ? 'hidden md:flex' : 'flex',
)}>
```

## Reference Pattern (PonderPage)

```tsx
// 1. Left pane: hide when detail showing
const showMobileDetail = !!slug

<div className={cn(
  'w-72 shrink-0 border-r border-border flex flex-col bg-card',
  showMobileDetail ? 'hidden md:flex' : 'flex',
)}>

// 2. Detail pane: hide on mobile when no selection
<div className={cn(
  'flex-1 min-w-0',
  showMobileDetail ? 'flex flex-col' : 'hidden md:flex md:flex-col',
)}>

// 3. Back button in detail header (md:hidden)
// 4. Mobile tab bar at bottom of detail pane (Chat / Files / Team)
```

## Workspace Page Pattern (EvolvePage/GuidelinePage/InvestigationPage)

These use a bottom sheet for Files:
```tsx
// In header: Files toggle button (md:hidden)
<button onClick={() => setMobileWorkspaceOpen(o => !o)} className="md:hidden ...">
  <Files className="w-4 h-4" />
</button>

// Bottom sheet
<div className={cn(
  'md:hidden absolute inset-x-0 bottom-0 z-50 ...',
  mobileWorkspaceOpen ? 'translate-y-0' : 'translate-y-full',
)}>
  <WorkspacePanel ... />
</div>
```

Bottom sheet works but is inconsistent with PonderPage's tab bar approach.

## ToolsPage Issues

- Auto-selects first tool on load → mobile always shows detail, never list
- No back button in detail view
- Uses state selection (not URL routing) — minor but different from other pages