# Code Review — Lazy Route Splitting

## Changes reviewed

- `frontend/src/App.tsx` — single file changed

## Summary

Converted 24 static page imports to `React.lazy()` dynamic imports with a `<Suspense>` fallback. HubPage remains static (used outside router). A `PageSpinner` component provides the loading fallback, reusing the existing spinner pattern.

## Findings

### 1. Named export adapter pattern — ACCEPTED

Each lazy call uses `.then(m => ({ default: m.ComponentName }))` to adapt named exports for `React.lazy()`. This is the standard React pattern for named-export modules. It avoids modifying all 25 page files and centralizes the lazy-loading concern.

### 2. Single Suspense boundary — ACCEPTED

One `<Suspense>` wraps all `<Routes>`. This is appropriate because:
- The spinner is identical regardless of which route is loading
- Per-route boundaries would add complexity without UX benefit
- The `AppShell` (sidebar/nav) remains visible during page loads

### 3. HubPage remains static — CORRECT

HubPage is rendered before the router (`if (hubMode === 'hub') return <HubPage />`) and cannot be lazy-loaded inside a Suspense boundary that lives within the router. Keeping it static is the correct approach.

### 4. Build output — VERIFIED

`npm run build` produces 59 JS chunk files including individual chunks for each page (Dashboard, FeatureDetail, etc.). TypeScript compilation clean. All 22 existing tests pass.

### 5. No error boundary for chunk load failures — TRACKED

If a lazy chunk fails to load (network error, deployment during navigation), React will throw an unhandled error. A chunk-load error boundary would improve resilience. This is a future enhancement, not a blocker for this refactor.

**Action:** `sdlc task add` tracked below.

## Verdict

**APPROVED** — Clean, minimal change. Single file touched. Build verified. Tests pass.
