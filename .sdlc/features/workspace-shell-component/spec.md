# Spec: WorkspaceShell — shared two-pane list/detail page layout component

## Problem

Multiple workspace pages (PonderPage, EvolvePage, InvestigationPage, GuidelinePage) implement the same structural two-pane layout pattern: a fixed-width left list pane and a flexible right detail pane, with mobile stacking, responsive visibility toggling via `showMobileDetail`, a top-level `h-full flex flex-col overflow-hidden` wrapper, and consistent scroll containment. This structural code is duplicated verbatim across all four pages, creating ~30-50 lines of boilerplate per page with no abstraction.

When one page gets a fix (e.g. a scroll container correction or a breakpoint tweak), the fix must be applied to every page manually. This has already led to divergence between pages.

## Goal

Extract the repeated two-pane shell structure into a single shared `WorkspaceShell` component that all workspace pages can use, eliminating the structural duplication without changing any visible behavior or user-facing functionality.

## Scope

- Create `frontend/src/components/layout/WorkspaceShell.tsx` exporting a `WorkspaceShell` component
- Refactor PonderPage, EvolvePage, InvestigationPage, and GuidelinePage to use it
- No changes to the detail or list pane _content_ — only the outer shell structure is extracted

## Component Contract

```tsx
<WorkspaceShell
  listPane={<ReactNode>}       // rendered in the left pane
  detailPane={<ReactNode>}     // rendered in the right pane
  showDetail={boolean}         // true when a detail item is selected (drives mobile stacking)
  listWidth?: string           // Tailwind width class for the left pane, default: "w-72"
/>
```

The shell produces:
```
<div className="h-full flex flex-col overflow-hidden">
  <div className="flex-1 flex min-h-0">
    <div className={cn(listWidth, "shrink-0 border-r border-border flex flex-col bg-card", showDetail ? "hidden lg:flex" : "flex")}>
      {listPane}
    </div>
    <div className={cn("flex-1 min-w-0", showDetail ? "flex flex-col" : "hidden lg:flex lg:flex-col")}>
      {detailPane}
    </div>
  </div>
</div>
```

## Affected Pages

| Page | Route | Current list width | Mobile detail trigger |
|---|---|---|---|
| PonderPage | /ponder/:slug | w-72 | `!!slug` |
| EvolvePage | /evolve/:slug | w-72 | `!!slug` |
| InvestigationPage | /investigations/:slug | w-72 | `!!slug` |
| GuidelinePage | /guidelines/:slug | w-72 | `!!slug` |

KnowledgePage uses a three-column layout (catalog + list + detail) and is **out of scope** for this feature.

## Acceptance Criteria

1. `WorkspaceShell` component exists at `frontend/src/components/layout/WorkspaceShell.tsx`
2. PonderPage, EvolvePage, InvestigationPage, and GuidelinePage each use `WorkspaceShell` for their outer shell
3. No visible behavioral change on any page — layout, mobile stacking, and breakpoints are identical to before
4. TypeScript compiles with no errors (`npm run build` passes)
5. Each consuming page is shorter by at least 10 lines of structural boilerplate

## Non-Goals

- Not changing any content inside the left or right panes
- Not extracting any page-specific header, filter tabs, or item row components
- Not adding new layout features (resizable panes, etc.)
- Not touching KnowledgePage (three-pane layout is different)
- Not touching SpikePage or other non-workspace pages
