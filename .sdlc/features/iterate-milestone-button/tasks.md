# Tasks: Iterate button on ReleasedPanel

## T1: Add Iterate button and modal wiring to ReleasedPanel

Add the "Iterate" button to `ReleasedPanel.tsx` actions row. Import `RefreshCw` icon, `NewIdeaModal`, `nextIterationSlug`, and `useNavigate`. Add state for modal open/closed, computed slug, and brief. Implement `handleIterate` that fetches existing ponder slugs, computes next version slug, builds brief template from milestone data, and opens the modal. Wire `onCreated` to navigate to `/ponder/{slug}`.

## T2: Verify integration with iterate-slug-utility

Ensure `nextIterationSlug` from `frontend/src/lib/iterate.ts` is available and correctly imported. Test that slug versioning produces expected results (base -> base-v2, base-v2 -> base-v3).
