# Tasks — Lazy Route Splitting

## T1: Convert 24 static page imports to React.lazy() in App.tsx

Replace all 24 routed page imports (everything except HubPage) with `React.lazy()` calls using the named-export adapter pattern. Add `import { lazy, Suspense } from 'react'`. Keep HubPage as a static import.

## T2: Add Suspense boundary with loading spinner

Wrap the `<Routes>` block in `<Suspense fallback={<PageSpinner />}>`. Create a small inline `PageSpinner` component (centered spinner div matching the existing loading pattern already in App.tsx).

## T3: Verify build produces per-page chunks

Run `npm run build` and confirm Vite output includes separate chunk files for each lazy-loaded page. Verify no TypeScript errors with `tsc -b`.

## T4: Run existing tests

Run `npm run test` and confirm all existing tests pass without modification.
