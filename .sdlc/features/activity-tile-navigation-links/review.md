# Code Review: Activity Tile Navigation Links

## Changes

### New file: `frontend/src/lib/routing.ts`
- `runTargetRoute(runType, target)` — pure function mapping run types to entity routes.
- Returns `null` for empty target or unknown/project-level run types.
- Clean switch statement, no side effects.

### Modified: `frontend/src/components/layout/RunCard.tsx`
- Added imports: `Link` (react-router-dom), `ExternalLink` (lucide-react), `runTargetRoute` (lib/routing).
- Added navigation link rendering at lines 138-152 inside the header button's info div.
- Uses `e.stopPropagation()` to prevent expand/collapse toggle when clicking the link.
- Subtle styling: `text-[10px] text-primary/70` with hover states for underline and full primary color.

## Findings

1. **Link inside button** — The `<Link>` is rendered inside a `<button>` element (the expand/collapse toggle button). While this works in practice because `stopPropagation` prevents the button click, it's technically invalid HTML (interactive content inside interactive content). However, this matches the existing pattern in this component where the entire header area is a button, and the stop button also lives adjacent with `stopPropagation`. The alternative (restructuring the entire header) would be a much larger change for minimal benefit. **Accepted** — matches existing pattern, works correctly.

2. **No XSS risk** — `run.run_type` and `run.target` come from the server-side `RunRecord` and are used in route paths via React Router's `<Link>`, which handles encoding. No raw HTML injection vectors.

3. **No unnecessary re-renders** — `runTargetRoute` is a pure function called inline; since `run` is already a prop, no additional state or effects are introduced.

## Verdict

**Approved.** Minimal, focused change. Two files touched, clear utility extraction, consistent with existing component patterns.
