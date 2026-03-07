# Tasks: Activity Tile Navigation Links

## Task 1: Create `runTargetRoute` utility
- **File**: `frontend/src/lib/routing.ts` (new)
- **Work**: Implement `runTargetRoute(runType: string, target: string): string | null` mapping run types to route paths per the design.
- **Acceptance**: Function returns correct route for each run_type, null for unknown/project-level types, null for empty target.

## Task 2: Add navigation link to RunCard header
- **File**: `frontend/src/components/layout/RunCard.tsx`
- **Work**: Import `Link` from react-router-dom and `ExternalLink` from lucide-react. Call `runTargetRoute(run.run_type, run.target)`. If non-null, render a `<Link>` below the meta line with the route prefix and target slug. Use `onClick={e => e.stopPropagation()}` to avoid toggling expand/collapse.
- **Acceptance**: Clicking the link navigates to the correct entity detail page without expanding/collapsing the card. No link rendered for vision_align or architecture_align runs.

## Task 3: Verify build passes
- **Work**: Run `cd frontend && npx tsc --noEmit` to verify no type errors.
- **Acceptance**: Zero type errors.
