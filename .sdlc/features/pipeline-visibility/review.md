# Review: Pipeline Visibility Indicator

## Summary

The implementation adds a `PipelineIndicator` component that renders five horizontal stage pills on the Dashboard, giving users an at-a-glance view of where they are in the Ponder → Plan → Commit → Run Wave → Ship flow.

## Files Changed

- `frontend/src/components/PipelineIndicator.tsx` — new component
- `frontend/src/pages/Dashboard.tsx` — integration

## Review Findings

### Correctness

**Stage logic is correct and greedy.** The `computeCurrentStage` function evaluates stages from highest to lowest (Ship → Run Wave → Commit → Plan → Ponder) with early returns, which correctly implements the "highest reached stage wins" rule from the spec. Verified edge cases:

- New project (no ponders, no milestones): returns 0 → Ponder highlighted ✓
- Committed ponder but no milestone: returns 1 → Plan highlighted ✓
- Any milestone exists: returns 2 → Commit highlighted ✓
- Active/verifying milestone: returns 3 → Run Wave highlighted ✓
- Released milestone: returns 4 → Ship highlighted ✓

**Navigation is correct.** Each pill links to the route specified in the spec:
- Ponder → `/ponder` ✓
- Plan → `/ponder` ✓
- Commit → `/milestones` ✓
- Run Wave → `/milestones` ✓
- Ship → `/milestones` ✓

**Tooltip text matches spec exactly.** ✓

**TypeScript compiles with zero errors.** (`npx tsc --noEmit` passes) ✓

### Component Design

The component is stateless — it receives `ponders` and `milestones` as props and derives everything from them. This is the right approach: no side effects, fully testable, no data-fetching inside the component.

The Dashboard fetches ponders via `api.getRoadmap()` in the existing `Promise.all` initialization block, keeping the fetch pattern consistent with how config, vision, and architecture are loaded.

### Accessibility

- `role="navigation"` with `aria-label` on the container
- `aria-current="step"` on the current stage pill
- Native `title` attributes for tooltip text (accessible to screen readers and keyboard users)
- `<Link>` elements for keyboard navigation

### Visual Hierarchy

The indicator is placed between Project Overview and the Stats bar — visible on page load without scrolling, above the main feature/milestone content. This matches the spec placement requirement.

### No Regressions

The Dashboard change:
- Adds one new state variable (`ponders`)
- Adds one API call to the existing `Promise.all` (non-blocking, fails gracefully with empty array)
- Adds one rendered block between existing sections
- Does not remove or alter any existing functionality

### Code Style

Consistent with the existing codebase:
- Tailwind utility classes throughout
- `cn()` helper for conditional class composition
- `lucide-react` for the checkmark icon
- `react-router-dom` `Link` for navigation

## Acceptance Criteria Verification

- [x] `PipelineIndicator` component renders on the Dashboard
- [x] Five stages display as horizontal pills with arrows between them
- [x] Current stage (furthest reached) is visually highlighted
- [x] Each stage pill is clickable and navigates to the correct page
- [x] Indicator is visible on page load without scrolling
- [x] Tooltips appear on hover for each stage (native `title` attribute)
- [x] New project (no ponders, no milestones): Stage 1 (Ponder) is highlighted as the starting point

## Verdict

**Approved.** No issues found. Implementation matches spec exactly with clean, well-structured code and no regressions.
