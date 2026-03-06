# Spike UI Page — Code Review

## Summary

The implementation adds a complete `SpikePage` React component with list view, detail view, verdict-driven sections, and sidebar/router integration. The build passes with zero TypeScript errors.

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/pages/SpikePage.tsx` | New — SpikePage component (~320 lines) |
| `frontend/src/lib/types.ts` | Added `SpikeVerdict`, `SpikeSummary`, `SpikeDetail` types |
| `frontend/src/api/client.ts` | Added `getSpikes`, `getSpike`, `promoteSpike` API methods |
| `frontend/src/components/layout/Sidebar.tsx` | Added `FlaskConical` import + Spikes nav entry in `plan` group |
| `frontend/src/components/layout/BottomTabBar.tsx` | Added `/spikes` to Plan tab roots |
| `frontend/src/App.tsx` | Added `SpikePage` import + `/spikes` and `/spikes/:slug` routes |

## Review Findings

### Correctness

- **Verdict badge colors**: ADOPT=green, ADAPT=yellow, REJECT=red — matches spec exactly. Both light and dark mode classes provided.
- **ADOPT detail**: Shows "What's Next" card explaining ADOPT = proven approach, with copyable `/sdlc-hypothetical-planning <slug>` command.
- **ADAPT detail**: Shows "Promote to Ponder" button when `ponder_slug` is absent; shows link to existing ponder entry when `ponder_slug` is set. Calls `POST /api/spikes/:slug/promote` and navigates on success. Error shown inline on failure.
- **REJECT detail**: Shows "Stored in Knowledge" card. Links to `/knowledge/<knowledge_slug>` when set.
- **Empty state**: Shows `FlaskConical` icon, heading, explanation, and CLI example format.
- **Ponder lineage in list rows**: ADAPT rows with `ponder_slug` set show `"Ponder: <slug>"` text in the row metadata.
- **Routing**: Both `/spikes` and `/spikes/:slug` are registered in App.tsx. Mobile/desktop responsive split works correctly (same pattern as InvestigationPage).
- **Sidebar**: `FlaskConical` icon, "Spikes" label, `/spikes` route, `exact: false` — correct for sub-routes.
- **BottomTabBar**: `/spikes` added to Plan tab roots — tapping Plan highlights when on spike routes.

### Code Quality

- Component is self-contained in a single file following project conventions.
- Sub-components (`VerdictBadge`, `SpikeRow`, `SpikeDetailPane`, `AdoptSection`, `AdaptSection`, `RejectSection`, `CopyButton`) are named clearly and scoped locally.
- No `unwrap()` calls — errors are caught and shown to the user.
- Loading states use the existing `Skeleton` component.
- API calls follow the `request<T>` pattern from `client.ts`.
- Types are minimal and accurate to the REST shape described in the spec.

### No Regressions

- Build output is clean (no TypeScript errors, no new warnings beyond the pre-existing chunk size warning).
- Existing pages and routes are unmodified except for the additive changes to Sidebar, BottomTabBar, and App.tsx.

### Findings to Track

- **No SSE subscription** in SpikePage — this is correct per spec (spikes are immutable). If the backend eventually emits spike events (e.g., after `/sdlc-spike` completes), a future feature can add the SSE hook.
- The ADAPT "Promote to Ponder" button uses a hardcoded yellow background (`bg-yellow-500`) rather than a semantic design token. This is acceptable for now given the verdict-driven color scheme, but could be aligned with the design system in a future design-consistency pass.

## Verdict

APPROVED. All spec requirements are met, build is clean, and the implementation follows established patterns.
