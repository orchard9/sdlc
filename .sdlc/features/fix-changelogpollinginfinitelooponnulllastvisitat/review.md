# Code Review: Fix Changelog Polling Infinite Loop on null lastVisitAt

## Change Summary

`frontend/src/hooks/useChangelog.ts` — added `useMemo` to the React import and wrapped the `since` computation in `useMemo([lastVisitAt])`.

**Diff (logical):**
```diff
-import { useCallback, useEffect, useState } from 'react'
+import { useCallback, useEffect, useMemo, useState } from 'react'

-  const since = lastVisitAt
-    ? lastVisitAt
-    : new Date(Date.now() - SEVEN_DAYS_MS).toISOString()
+  const since = useMemo(
+    () => lastVisitAt ?? new Date(Date.now() - SEVEN_DAYS_MS).toISOString(),
+    [lastVisitAt]
+  )
```

## Correctness

**Bug fixed:** `since` is now computed once per distinct `lastVisitAt` value. Because `lastVisitAt` is initialized from `localStorage` in a `useState` initializer with no setter exposed, it is stable for the component lifetime. `useMemo` makes this explicit: `since` is created once on mount (null case: timestamp captured at mount time, never advancing) or derived from the stored string (returning visitor). `useCallback([since])` no longer creates a new `refresh` on every render. `useEffect([refresh])` fires once on mount only.

**Null case stability:** When `lastVisitAt` is null, `new Date(Date.now() - SEVEN_DAYS_MS).toISOString()` runs exactly once (the `useMemo` factory), producing a single stable string for the lifetime of the component. The infinite loop is eliminated.

**Returning visitor unchanged:** When `lastVisitAt` is a stored string, `since = lastVisitAt` — same as before.

## No Regressions

- `dismiss()` still writes to localStorage and sets `dismissed = true`. The banner hides immediately. This flow is unaffected.
- SSE-triggered refresh via `useSSE(refresh)` still works — `refresh` is the same stable callback, called on each SSE event.
- TypeScript: `npx tsc --noEmit` passes with zero errors.
- The `??` operator is valid TypeScript and transpiles correctly under the project's Vite/esbuild config.

## Findings

None. The change is minimal, targeted, and correct. No secondary issues identified.

## Verdict: APPROVE
