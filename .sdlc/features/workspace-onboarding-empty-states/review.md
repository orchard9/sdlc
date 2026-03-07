# Code Review: Rich Onboarding Empty States

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/pages/PonderPage.tsx` | Already had rich empty state from prior iteration — no change needed |
| `frontend/src/pages/InvestigationPage.tsx` | Replaced 6-line minimal placeholder with structured Hero/How-it-works/CTA (~45 lines) |
| `frontend/src/pages/GuidelinePage.tsx` | Replaced 6-line minimal placeholder with structured Hero/How-it-works/CTA (~45 lines) |
| `frontend/src/pages/SpikePage.tsx` | Already had `SpikeEmptyState` component for list pane; detail pane now has rich empty state with Verdicts strip |
| `frontend/src/pages/KnowledgePage.tsx` | Replaced 6-line minimal placeholder with structured Hero/How-it-works/CTA (~45 lines) |
| `frontend/src/pages/HubPage.tsx` | Minor: added dynamic tab title `useEffect` (not part of empty states feature but co-committed) |

## Review Findings

### F1: Consistent pattern across all pages — PASS
All five workspace pages follow the same visual structure: `max-w-xl mx-auto px-6 py-10 space-y-8` container, Hero with icon in `bg-primary/10` box, "How it works" card list, and CTA section. Typography tokens match across all pages.

### F2: CTA buttons wire to correct actions — PASS
- Ponder: `setShowSuggest(true)` and `setShowForm(true)` — both trigger existing modals
- Root Cause: `setShowModal(true)` — triggers CreateWorkspaceModal
- Guidelines: `setShowModal(true)` — triggers CreateWorkspaceModal
- Spikes: Static CLI command display (no interactive button needed)
- Knowledge: Static CLI command display (no interactive button needed)

### F3: Conditional "select from list" hint — PASS
All pages show "Or select ... from the list" text only when `entries.length > 0` (or `spikes.length > 0` for SpikePage). This avoids confusing new users with an empty list.

### F4: No new dependencies or components introduced — PASS
All icons come from existing `lucide-react` imports. No new shared components — the pattern is inline JSX. This is appropriate given the simplicity and prevents premature abstraction.

### F5: TypeScript compilation — PASS
`npx tsc --noEmit` passes with zero errors across all modified files.

### F6: No regressions in detail pane — PASS
The ternary `slug ? <DetailPane /> : <EmptyState />` pattern is preserved unchanged in all pages. When a slug is present, the detail pane renders exactly as before.

### F7: HubPage tab title side-effect — NOTED
The `useEffect(() => { document.title = 'Ponder Hub' }, [])` addition in HubPage.tsx is not part of the empty states feature but was co-committed. It is harmless and correct. Tracked separately in the `ui-title` feature.

## Verdict

**APPROVED** — Clean, consistent implementation across all workspace pages. No issues found.
