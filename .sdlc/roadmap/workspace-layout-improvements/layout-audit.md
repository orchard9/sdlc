# Layout Technical Audit

## Current Architecture

```
AppShell
├── flex h-screen overflow-hidden
├── Sidebar (fixed mobile / static desktop)
├── <div className="flex-1 flex flex-col overflow-hidden min-w-0">
│   ├── <header className="md:hidden ...">   ← mobile top bar
│   └── <main className="flex-1 overflow-y-auto"> ← ⚠️ missing min-h-0
│       └── {children} (PonderPage, EvolvePage, etc.)
├── AgentPanel (hidden md:flex)
└── BottomTabBar (md:hidden fixed bottom-0 h-11 z-30)
```

## Bug 1 — AppShell `<main>` missing `min-h-0`

```tsx
// CURRENT (wrong):
<main className="flex-1 overflow-y-auto">

// CORRECT:
<main className="flex-1 overflow-y-auto min-h-0">
```

Without `min-h-0`, a flex item in a column can grow beyond its allocated
height instead of constraining its children. This means `h-full` in
children could resolve to content height rather than viewport height,
breaking the flex-column pinning in all dialogue panels.

## Bug 2 — InputBar missing `shrink-0`

Both `DialoguePanel.tsx` and `InvestigationDialoguePanel.tsx` render
`<InputBar />` as the last child of a `flex flex-col` container without
the parent wrapping it in `shrink-0`. The InputBar's own root elements
(a `<form>` or `<div>`) have no `shrink-0` class.

```tsx
// CURRENT (fragile):
<div className="h-full flex flex-col min-h-0">
  <div className="shrink-0 ...">Header</div>
  <div className="flex-1 overflow-y-auto min-h-0">Messages</div>
  <InputBar />   ← no shrink-0, can compress under pressure
</div>

// CORRECT:
// Option A — add className prop to InputBar
<InputBar className="shrink-0" />

// Option B — wrap in div
<div className="shrink-0">
  <InputBar />
</div>
```

## Bug 3 — BottomTabBar covers mobile content

`BottomTabBar` uses `fixed bottom-0 h-11` — it floats above all page
content. Pages that render in-flow content at the bottom (PonderPage's
tab bar, chat input) don't account for this 44px overlay.

`AppShell`'s `<main>` needs `pb-11 md:pb-0` to ensure content isn't
covered on mobile:

```tsx
<main className="flex-1 overflow-y-auto min-h-0 pb-11 md:pb-0">
```

Alternatively, each page that has a bottom-anchored element needs to
account for 44px on mobile. The AppShell approach is more reliable.

Note: PonderPage has its OWN in-page mobile tab bar (`md:hidden shrink-0
flex border-t border-border bg-card`) for Chat/Files/Team switching. This
is DIFFERENT from the global BottomTabBar (app navigation). Both render
on mobile. The in-page tab bar sits in the flex flow above the global
fixed bar — but the global bar overlays it visually since it's `fixed`.
The PonderPage's own tab bar should replace the global BottomTabBar on
that page, OR the page needs to add padding to its content so the
InputBar floats above the global bar.

**Clean solution**: The PonderPage in-page tab bar IS the right nav for
that context. Consider hiding `BottomTabBar` on ponder routes, OR simply
add `pb-11 md:pb-0` to `main` in AppShell so content is never covered.
The AppShell global padding approach is simpler.

## Bug 4 — EvolvePage/InvestigationPage missing `min-h-0` in content wrapper

```tsx
// EvolvePage line 295, InvestigationPage line 283:
<div className="flex-1 flex min-h-0">
  <div className="flex-1 min-w-0 min-h-0">
    <InvestigationDialoguePanel .../>  // uses h-full inside
  </div>
```

The `<div className="flex-1 min-w-0 min-h-0">` wrapping the dialogue
panel is a row-flex item with `flex-1 min-w-0 min-h-0`. The dialogue
panel uses `h-full`. In a row flex, `flex-1` items stretch by default
(align-items: stretch), so `h-full` should resolve correctly. But if
there's ever alignment deviation, this could break. Explicit `h-full` on
the wrapper is safer:

```tsx
<div className="flex-1 min-w-0 min-h-0 h-full">
```

## Primary Breakpoint: md vs lg

All layout-mode switching uses `md` (768px). This means:
- iPad mini portrait (768px): desktop layout ← wrong
- iPad Air portrait (820px): desktop layout ← wrong
- iPad Pro portrait (1024px): desktop layout ← borderline

Changing primary breakpoint to `lg` (1024px) means:
- All iPad portraits: mobile layout ← correct
- iPad Pro landscape (1366px) / large tablets: desktop layout ← correct
- Laptops 1024px+: desktop layout ← correct

### Files to change (md → lg for layout boundary):

| File | Classes to change |
|------|------------------|
| `AppShell.tsx` | sidebar `md:static md:translate-x-0`, `md:hidden` overlay, `md:hidden` header, `md:flex` panels |
| `PonderPage.tsx` | `hidden md:flex`, `md:hidden`, `md:hidden fixed bottom-16` |
| `EvolvePage.tsx` | `md:hidden`, `hidden md:flex`, `md:hidden absolute` |
| `InvestigationPage.tsx` | `md:hidden`, `hidden md:flex`, `md:hidden absolute` |
| `GuidelinePage.tsx` | same pattern as Evolve/Investigation |
| `AgentPanel.tsx` | `hidden md:flex` → `hidden lg:flex` |
| `AgentPanelFab.tsx` | `md:hidden` → `lg:hidden` |
| `BottomTabBar.tsx` | `md:hidden` → `lg:hidden` |

Note: NOT all `md:` uses should change — only those that switch between
desktop and mobile LAYOUT MODES. Inner component responsive tweaks
(e.g., `md:flex-row` for visual polish inside a panel) can stay at `md`.