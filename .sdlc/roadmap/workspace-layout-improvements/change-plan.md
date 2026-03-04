# Change Plan: Workspace Layout Improvements

⚑  Decided: Three distinct tracks of work — pinning fixes, breakpoint shift, code cleanup.

## Track 1 — Chat Bar Pinning (All Layouts)

**Approach: Flex column (keep) + fix the chain**

The flex column pattern is correct. CSS position:fixed would require
dynamic sidebar/panel width compensation — too fragile. The issues are
in the chain, not the pattern.

### Fix 1a — AppShell main: add `min-h-0 pb-11 lg:pb-0`
```tsx
// frontend/src/components/layout/AppShell.tsx
<main className="flex-1 overflow-y-auto min-h-0 pb-11 lg:pb-0">
```
- `min-h-0`: prevents flex item overflow that breaks h-full in children
- `pb-11`: reserves 44px at bottom for fixed BottomTabBar on mobile
- `lg:pb-0`: removes padding once layout switches to desktop (no BottomTabBar)

### Fix 1b — InputBar root elements: add `shrink-0`
Both DialoguePanel and InvestigationDialoguePanel need InputBar to never
shrink. Two options:
- Add `shrink-0` to InputBar's wrapper div/form directly
- OR wrap `<InputBar />` call in `<div className="shrink-0">` at call site

Simpler to fix at source — add `shrink-0` to InputBar's root elements in:
- `frontend/src/components/ponder/DialoguePanel.tsx` (~line 130)
- `frontend/src/components/investigation/InvestigationDialoguePanel.tsx` (~line 90)

### Fix 1c — EvolvePage/InvestigationPage flex wrapper: add `h-full`
```tsx
// Currently:
<div className="flex-1 min-w-0 min-h-0">
// Add:
<div className="flex-1 min-w-0 min-h-0 h-full">
```
Without `h-full`, the InvestigationDialoguePanel's `h-full` resolves to
the content height of an unsized flex item.

## Track 2 — Breakpoint: md (768px) → lg (1024px)

**Rule: Change ONLY layout-mode switches, not visual tweaks.**

Layout-mode switches = things that control whether you see the mobile
single-column view or the desktop multi-column view.

### Files and changes:

**AppShell.tsx**
- Sidebar overlay: `md:hidden` → `lg:hidden`
- Sidebar static/fixed: `md:static md:z-auto md:translate-x-0` → `lg:static lg:z-auto lg:translate-x-0`
- Mobile header: `md:hidden` → `lg:hidden`
- AgentPanel wrapper: `hidden md:flex` → `hidden lg:flex`
- Panel toggle button: `hidden md:flex` → `hidden lg:flex`
- AgentPanelFab: (in its own file) `md:hidden` → `lg:hidden`
- BottomTabBar: (in its own file) `md:hidden` → `lg:hidden`

**PonderPage.tsx**
- Desktop container: `hidden md:flex` → `hidden lg:flex`
- Mobile content: `md:hidden` → `lg:hidden`
- Mobile floating nav: `md:hidden fixed bottom-16` → `lg:hidden fixed bottom-16`
- Mobile tab bar: `md:hidden shrink-0` → `lg:hidden shrink-0`
- Sidebar list mobile: `hidden md:flex` → `hidden lg:flex`

**EvolvePage.tsx, InvestigationPage.tsx, GuidelinePage.tsx**
- Similar pattern — all `md:hidden` layout switches → `lg:hidden`
- All `hidden md:flex` layout switches → `hidden lg:flex`
- Exception: `md:hidden` mobile back buttons/headers inside these pages

**Sidebar.tsx**
- Collapse button on mobile (if present)

? Open: Should `md:` be changed to `lg:` inside individual page components
  for non-layout things like list column switches? Decision: NO — only
  layout mode switches. Visual tweaks stay at `md:`.

## Track 3 — Code Cleanup

### 3a — Consistent flex chain documentation
Add a comment block at the top of each dialogue panel explaining the
height chain contract:
```tsx
// HEIGHT CONTRACT: This component requires an h-full ancestor chain.
// Parent must be a bounded flex item (flex-1 min-h-0) in a flex column
// with known height. See AppShell.tsx for the root h-screen.
```

### 3b — Extract shared WorkspaceLayout pattern (optional, lower priority)
PonderPage, EvolvePage, InvestigationPage, GuidelinePage all share:
```tsx
// Desktop: hidden lg:flex flex-1 min-h-0
//   ├── Dialogue panel (flex-1)
//   ├── Resize handle (optional)
//   └── Workspace panel (shrink-0)
// Mobile: lg:hidden flex-1 min-h-0 flex flex-col
//   ├── Tab content (flex-1)
//   └── Tab bar (shrink-0)
```
A `<WorkspaceLayout>` wrapper could eliminate duplication. DEFERRED —
do the mechanical fixes first, extract after the code is stable.

### 3c — PonderPage tab bar vs global BottomTabBar
The PonderPage has an in-page tab bar (Chat/Files/Team) AND the app-level
BottomTabBar renders on mobile. With `pb-11` on `main`, the in-page tab
bar plus BottomTabBar would mean 88px of bottom bar — too much.

⚑  Decided: PonderPage's in-page mobile tab bar should USE the BottomTabBar
slot via a portal or by restructuring. But that's a larger refactor.
For now: the in-page tab bar sits above the BottomTabBar (in flow, in
the flex column of the content area). The content area has `pb-11` so
the BottomTabBar doesn't cover it. The in-page tab bar at the bottom of
the flex column acts as a visible secondary nav.

? Open: Is double bottom nav (app nav + in-page nav) acceptable UX on
  mobile? Or should ponder/evolve/investigation pages suppress the global
  BottomTabBar when on specific routes?

## Implementation Order

1. Track 1a: AppShell main — `min-h-0 pb-11 lg:pb-0` (1 line)
2. Track 2: Replace all layout-mode `md:` → `lg:` (mechanical grep-replace)
3. Track 1b: InputBar `shrink-0` in DialoguePanel + InvestigationDialoguePanel
4. Track 1c: flex wrapper `h-full` in Evolve/Investigation pages
5. Track 3c: Review double bottom nav — decide suppress vs accept
6. Track 3a: Comment block on height contract (low priority, optional)