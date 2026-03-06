# Design: Shift Layout Breakpoint md→lg

## Overview

This is a pure mechanical substitution. There is no new UI to design — the goal is to change the threshold at which the existing mobile vs. desktop layout is selected.

**Current:** `md` (≥768px) → desktop layout
**Target:** `lg` (≥1024px) → desktop layout

No new components, no new CSS, no new logic. Only class string changes.

## Breakpoint Reference

| Tailwind token | Pixel value | Note |
|---|---|---|
| `md` | 768px | Current threshold — tablets land here |
| `lg` | 1024px | Target threshold — excludes all tablet sizes |

## Change Map

For each file, only the layout-switching occurrences are updated. Content-level responsive classes (grid columns) are unchanged.

### `components/layout/AppShell.tsx`

| Current class | Replacement | Purpose |
|---|---|---|
| `md:hidden` | `lg:hidden` | Mobile overlay backdrop |
| `md:static md:z-auto` | `lg:static lg:z-auto` | Sidebar position at wide viewport |
| `md:translate-x-0` | `lg:translate-x-0` | Sidebar slide-in override |
| `md:hidden` (header) | `lg:hidden` | Mobile top header |
| `hidden md:flex` (panel btn) | `hidden lg:flex` | Desktop panel open button |

### `components/layout/AgentPanel.tsx`

| Current class | Replacement | Purpose |
|---|---|---|
| `hidden md:flex` | `hidden lg:flex` | Show inline panel only on desktop |

### `components/layout/AgentPanelFab.tsx`

| Current class | Replacement | Purpose |
|---|---|---|
| `md:hidden` (×3) | `lg:hidden` (×3) | Hide FAB and drawer on desktop |

### `components/layout/BottomTabBar.tsx`

| Current class | Replacement | Purpose |
|---|---|---|
| `md:hidden` | `lg:hidden` | Hide bottom bar on desktop |

### Page-level files (PonderPage, InvestigationPage, EvolvePage, GuidelinePage, SpikePage, KnowledgePage, ToolsPage, ThreadsPage)

Each page has its own mobile/desktop split using the same pattern. All instances follow the same replacement:

| Current pattern | Replacement |
|---|---|
| `md:hidden` | `lg:hidden` |
| `hidden md:flex` | `hidden lg:flex` |
| `md:flex` | `lg:flex` |

## Files NOT Changed

- `frontend/src/pages/Dashboard.tsx` — `md:grid-cols-2` is a content grid, not a layout switch
- `frontend/src/pages/FeaturesPage.tsx` — `md:grid-cols-2` same reason
- `frontend/src/index.css` — raw `@media (max-width: 768px)` targets iOS zoom bug, unrelated to layout
- `frontend/src/components/shared/CommandBlock.tsx` — unrelated

## Verification Approach

After implementation, open the browser at exactly 768px, 900px, and 1023px width and confirm:
- BottomTabBar is visible
- Sidebar is not shown
- Mobile top header is visible
- AgentPanelFab is visible

At 1024px and above, confirm the reverse.
