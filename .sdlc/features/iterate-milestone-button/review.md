# Review: Iterate button on ReleasedPanel

## Files changed

| File | Change |
|------|--------|
| `frontend/src/lib/iterate.ts` | New — `nextIterationSlug()` utility function |
| `frontend/src/components/milestones/ReleasedPanel.tsx` | Added Iterate button, NewIdeaModal integration |

## Findings

### F1: TypeScript compilation — PASS
`npx tsc --noEmit` passes with zero errors. All imports resolve correctly.

### F2: Button placement and styling — PASS
The Iterate button is placed in the actions row between "Re-run UAT" and "Submit manually". It uses neutral `border-border bg-muted` styling, appropriately secondary to the green UAT button. Only shows when not running (consistent with existing pattern).

### F3: Data flow — PASS
`handleIterate` correctly:
- Fetches all ponder slugs via `api.getRoadmap(true)`
- Computes next version slug via `nextIterationSlug`
- Builds brief template with milestone title and vision
- Opens `NewIdeaModal` with pre-populated fields
- Silent catch on failure (consistent with existing patterns in this file)

### F4: Modal lifecycle — PASS
`NewIdeaModal` receives `initialTitle`, `initialSlug`, `initialBrief` props which it correctly uses in its `useEffect` reset. On `onCreated`, the modal closes and navigates to `/ponder/{slug}`.

### F5: `nextIterationSlug` edge cases — PASS
- Strips `-vN` suffix before scanning (handles `foo-v2` -> `foo` base correctly)
- Uses `Math.max` to find highest version (handles gaps like v2, v4 -> v5)
- Truncates to 40 chars (respects slug length limit)
- Escapes regex special chars: potential issue if slug contains regex metacharacters (e.g. `.`, `+`). However, ponder slugs are validated to `[a-z0-9-]` on creation, so this is not a practical concern.

### F6: No backend changes needed — PASS
All APIs used already exist: `getRoadmap`, `createPonderEntry`. No new endpoints required.

## Verdict

All findings pass. The implementation is minimal, correct, and follows existing patterns in the codebase.
