# Review: Shift Layout Breakpoint md→lg

## Summary

This review covers the mechanical breakpoint shift from `md:` (768px) to `lg:` (1024px) for all layout-switching responsive classes across the frontend.

## Changes Made

### Layout Components

**`frontend/src/components/layout/AppShell.tsx`**
- `fixed inset-0 z-30 bg-black/50 md:hidden` → `lg:hidden` — mobile overlay backdrop
- `md:static md:z-auto` → `lg:static lg:z-auto` — sidebar position at desktop width
- `md:translate-x-0` → `lg:translate-x-0` — sidebar slide-in override
- `flex items-center … border-b … md:hidden` → `lg:hidden` — mobile top header
- `hidden md:flex items-center …` → `hidden lg:flex` — desktop panel open button

**`frontend/src/components/layout/AgentPanel.tsx`**
- `hidden md:flex flex-col …` → `hidden lg:flex` — inline panel shown only at desktop width

**`frontend/src/components/layout/AgentPanelFab.tsx`**
- Three instances of `md:hidden` → `lg:hidden` — FAB button and drawer (mobile-only elements)

### Page-Level Files

Eight pages updated, each following the same pattern:
- `md:hidden` → `lg:hidden` (back button, mobile workspace toggle, bottom sheet controls)
- `hidden md:flex` → `hidden lg:flex` (desktop side panel)
- `md:flex md:flex-col` → `lg:flex lg:flex-col` (right pane in list-detail layouts)

Pages: PonderPage, InvestigationPage, EvolvePage, GuidelinePage, SpikePage, KnowledgePage, ToolsPage, ThreadsPage

**ThreadsPage specific:** `md:flex md:w-[280px]` → `lg:flex lg:w-[280px]`, `md:flex` → `lg:flex`

## Findings

### No issues found

1. **Correctness**: All `md:` layout-switch classes have been updated to `lg:`. The two remaining `md:` usages (`Dashboard.tsx:52`, `FeaturesPage.tsx:71`) are content grid classes (`md:grid-cols-2`) that are correctly excluded per spec.

2. **Completeness**: Verified via grep — zero layout-switching `md:` classes remain in any component or page file. Only `md:grid-cols-2` instances remain (Dashboard, FeaturesPage).

3. **Build passes**: `npm run build` completes successfully with no errors. The chunk size warning is pre-existing.

4. **iOS zoom fix untouched**: `index.css` `@media (max-width: 768px)` for input font-size is unchanged.

5. **Paired flex-col classes**: The pattern `'hidden lg:flex md:flex-col'` (created by replace_all running in two passes) was manually corrected to `'hidden lg:flex lg:flex-col'` in all five affected list-detail pages. This is correct — `flex-col` is needed alongside `flex` to establish column direction on the right pane.

6. **No regressions in scope**: BottomTabBar doesn't yet exist in main branch (it's in other feature worktrees). When it lands, its `md:hidden` should also become `lg:hidden` — but that's out of scope for this feature.

## Verdict

All changes are correct, complete, and non-regressive. Ready for audit.
