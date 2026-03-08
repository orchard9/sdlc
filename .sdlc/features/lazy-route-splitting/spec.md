# Lazy Route Splitting — React.lazy() for all 25 page imports

## Problem

The frontend bundles all 25 page components into a single JavaScript chunk. Every page — including rarely visited pages like Secrets, Network, Docs — is downloaded and parsed on initial load. This inflates the main bundle size and increases time-to-interactive for the most common routes (Dashboard, Features, Milestones).

## Solution

Replace all 25 static page imports in `App.tsx` with `React.lazy()` dynamic imports wrapped in a `<Suspense>` boundary. Each page becomes its own code-split chunk that is fetched on demand when the user navigates to that route.

### Key decisions

1. **Named export compatibility**: All 25 page components use named exports (`export function DashboardPage`). `React.lazy()` requires a default export. Rather than modifying all 25 page files to add default exports (touching every page file), use an inline adapter pattern in the lazy call:
   ```tsx
   const Dashboard = lazy(() =>
     import('@/pages/Dashboard').then(m => ({ default: m.Dashboard }))
   )
   ```
   This keeps page files unchanged and centralizes the lazy-loading concern in `App.tsx`.

2. **Single Suspense boundary**: Wrap the `<Routes>` block in a single `<Suspense fallback={<PageSpinner />}>`. Individual per-route suspense boundaries add complexity without meaningful UX benefit — the loading spinner is the same regardless.

3. **HubPage stays eager**: `HubPage` is rendered outside the router (hub mode check) and must remain a static import. All other 24 pages become lazy.

4. **Spinner component**: Extract a small `PageSpinner` component (centered spinner, matches existing loading pattern in App.tsx) for the Suspense fallback.

## Scope

- **In scope**: `App.tsx` refactor — lazy imports, Suspense boundary, spinner fallback
- **Out of scope**: Vendor chunk separation (separate feature), preloading/prefetching strategies, page file modifications

## Acceptance criteria

- All 24 routed pages load via `React.lazy()` dynamic imports
- `HubPage` remains a static import (used outside router)
- A visible loading spinner displays while lazy chunks load
- `npm run build` produces separate chunk files for each page
- No TypeScript errors (`tsc -b` passes)
- Existing tests continue to pass (`npm run test`)
- No functional regression — all routes render correctly
