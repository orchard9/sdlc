# Spec: Shift Layout Breakpoint md→lg for Tablet UX

## Problem

The current responsive layout uses Tailwind's `md:` breakpoint (768px) as the threshold between the mobile UX and the desktop sidebar/panel UX. Tablets — iPad Mini (768px), iPad (820px), iPad Air (820px), and iPad Pro (1024px) — fall at or just above this boundary, landing them in the desktop layout.

The desktop layout (sidebar nav, collapsible agent panel) is poorly suited for tablet use: the sidebar is narrow and awkward on touch, and the bottom tab bar (the intended tablet/mobile nav pattern) is hidden. Tablets need the mobile UX: BottomTabBar, the mobile header, and the AgentPanelFab drawer instead of the inline AgentPanel.

## Goal

Shift every layout-switching `md:` breakpoint to `lg:` (1024px) so that devices narrower than 1024px — including all current tablet form factors — receive the mobile UX. Devices 1024px and wider retain the desktop layout.

## In Scope

All Tailwind classes that control the mobile↔desktop split in layout components and pages:

| File | Pattern |
|---|---|
| `components/layout/AppShell.tsx` | Sidebar overlay, mobile header visibility, panel open button |
| `components/layout/AgentPanel.tsx` | `hidden md:flex` → `hidden lg:flex` |
| `components/layout/AgentPanelFab.tsx` | `md:hidden` → `lg:hidden` |
| `components/layout/BottomTabBar.tsx` | `md:hidden` → `lg:hidden` |
| `pages/PonderPage.tsx` | Desktop/mobile layout branches |
| `pages/InvestigationPage.tsx` | Desktop/mobile layout branches |
| `pages/EvolvePage.tsx` | Desktop/mobile layout branches |
| `pages/GuidelinePage.tsx` | Desktop/mobile layout branches |
| `pages/SpikePage.tsx` | Desktop/mobile layout branches |
| `pages/KnowledgePage.tsx` | Desktop/mobile layout branches |
| `pages/ToolsPage.tsx` | Desktop/mobile layout branches |
| `pages/ThreadsPage.tsx` | Desktop/mobile layout branches |

## Out of Scope

- Content-level `md:` classes unrelated to the mobile/desktop layout split (e.g. `md:grid-cols-2` on Dashboard/FeaturesPage). These control grid column counts within the content area and are not part of the layout toggle.
- The iOS Safari zoom fix in `index.css` which uses a raw media query at 768px — this addresses a font-size zoom bug and is independent of the layout breakpoint.
- The `CommandBlock.tsx` `md:` usage (line 4 — unrelated to layout switching).

## Acceptance Criteria

1. On a device or browser viewport of 768px–1023px wide: the bottom tab bar is visible, the sidebar is hidden by default, the mobile header is shown, and the AgentPanelFab is visible.
2. On a device or browser viewport of 1024px and wider: the sidebar is always shown, the bottom tab bar is hidden, the mobile header is hidden, and the inline AgentPanel is shown.
3. The visual layout at 1024px and above is identical to the current layout at 768px and above.
4. No regressions on desktop (≥1280px) or mobile (≤480px).

## Implementation Approach

A simple mechanical find-and-replace of layout-switching `md:` → `lg:` across the identified files. Each `md:hidden`, `hidden md:flex`, `md:flex`, `md:static`, `md:z-auto`, `md:translate-x-0` occurrence in the layout context must be updated. Non-layout `md:` uses (grid columns, etc.) are left untouched.
