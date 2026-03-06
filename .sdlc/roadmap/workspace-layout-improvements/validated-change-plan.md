# Validated Change Plan (Session 1)

Previous artifacts (layout-audit.md, change-plan.md) contained stale findings. This is the corrected plan validated against actual code as of 2026-03-04.

## Corrections from Previous Audit

- **Bug 1 (min-h-0 missing on main)** — ALREADY FIXED in code. No action needed.
- **Bug 3 (BottomTabBar covers content)** — PHANTOM. BottomTabBar is dead code, never rendered. pb-11 fix is unnecessary.
- **Track 1a (pb-11 lg:pb-0)** — REMOVED from plan. No BottomTabBar to clear.
- **Track 3c (double bottom nav)** — MOOT. Only PonderPage has a bottom tab bar, and BottomTabBar is dead code.

## Track 1 — InputBar shrink-0 (4 elements, 2 files)

DialoguePanel.tsx:
- Line 129: `<div className="shrink-0 flex items-center gap-2 ...">` (running)
- Line 146: `<form className="shrink-0 flex items-end gap-2 ...">` (idle)

InvestigationDialoguePanel.tsx:
- Line 112: `<div className="shrink-0 flex items-center gap-2 ...">` (running)
- Line 129: `<form className="shrink-0 flex items-end gap-2 ...">` (idle)

## Track 2 — Breakpoint md → lg (~30 replacements, 8 files)

Only layout-mode switches. Interior visual tweaks stay at md/sm.

Files: AppShell.tsx (5), PonderPage.tsx (6), EvolvePage.tsx (7), InvestigationPage.tsx (7), GuidelinePage.tsx (7), AgentPanel.tsx (1), AgentPanelFab.tsx (3), panel-open button (1).

## Track 3 — Cleanup

- Delete BottomTabBar.tsx (dead code)
- AgentPanelFab: bottom-[56px] → bottom-4
- PonderPage floating nav: bottom-16 → bottom-12

## Optional

- AppShell main: overflow-y-auto → overflow-hidden (both work, hidden is more correct)

## Implementation Order

1. Delete BottomTabBar.tsx
2. InputBar shrink-0 (4 elements)
3. Breakpoint md → lg (mechanical replacements)
4. AgentPanelFab + floating nav position fixes
5. Optional: main overflow
6. Visual testing at 768px and 1024px