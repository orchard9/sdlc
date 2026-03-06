# Tasks: Shift Layout Breakpoint md→lg

## T1 — Update layout components (AppShell, AgentPanel, AgentPanelFab, BottomTabBar)

Replace all layout-switching `md:` breakpoints with `lg:` in the four core layout components:
- `frontend/src/components/layout/AppShell.tsx`
- `frontend/src/components/layout/AgentPanel.tsx`
- `frontend/src/components/layout/AgentPanelFab.tsx`
- `frontend/src/components/layout/BottomTabBar.tsx`

## T2 — Update page-level layout breakpoints

Replace all layout-switching `md:` breakpoints with `lg:` in all affected pages:
- `frontend/src/pages/PonderPage.tsx`
- `frontend/src/pages/InvestigationPage.tsx`
- `frontend/src/pages/EvolvePage.tsx`
- `frontend/src/pages/GuidelinePage.tsx`
- `frontend/src/pages/SpikePage.tsx`
- `frontend/src/pages/KnowledgePage.tsx`
- `frontend/src/pages/ToolsPage.tsx`
- `frontend/src/pages/ThreadsPage.tsx`

## T3 — Verify no unintended md: regressions

After applying T1 and T2, confirm that:
- No layout-switching `md:` classes remain in the identified files
- Content-level `md:` classes (grid columns on Dashboard, FeaturesPage) are untouched
- The iOS zoom fix in `index.css` is untouched
- Build passes: `cd frontend && npm run build`
