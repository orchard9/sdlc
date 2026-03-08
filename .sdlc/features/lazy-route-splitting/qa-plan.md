# QA Plan — Lazy Route Splitting

## Build verification

1. Run `npm run build` — must succeed with no errors
2. Confirm Vite output includes multiple chunk files (not a single monolithic bundle)
3. Run `tsc -b` — must produce zero TypeScript errors

## Test suite

1. Run `npm run test` — all existing unit tests must pass
2. No test modifications should be required (lazy loading is transparent to component tests)

## Route verification (manual/visual)

1. Navigate to Dashboard (`/`) — renders correctly
2. Navigate to Features (`/features`) — renders correctly after lazy load
3. Navigate to a feature detail (`/features/:slug`) — renders correctly
4. Navigate to Milestones, Ponder, Settings, Docs — each renders after brief spinner
5. Hub mode detection still works — HubPage renders when hub API responds 200

## Regression checks

1. No flash of unstyled content on route transitions
2. Suspense fallback (spinner) displays briefly while chunk loads
3. Browser dev tools Network tab shows separate chunk requests per route navigation
4. No console errors or warnings related to lazy loading
