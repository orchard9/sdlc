# Spec: Fix Changelog Polling Infinite Loop on null lastVisitAt

## Problem

When `lastVisitAt` is `null` (first-time visitor, nothing stored in `localStorage`), the `useChangelog` hook triggers an infinite loop of API calls to `/api/changelog`.

**Root cause:** `since` is computed inline on every render:

```ts
const since = lastVisitAt
  ? lastVisitAt
  : new Date(Date.now() - SEVEN_DAYS_MS).toISOString()  // ← new string every render
```

Because `Date.now()` changes on every render, `since` is never the same object/value twice. `useCallback([since])` creates a new `refresh` function on every render. `useEffect(() => { refresh() }, [refresh])` fires on every render, calls fetch, calls `setEvents`/`setTotal`, triggers a re-render — and the cycle repeats indefinitely.

**Observed symptom:** 30+ requests per second to `/api/changelog` with microsecond-apart `since` timestamps:

```
← GET /api/changelog?since=2026-02-24T01:16:17.013Z&limit=50 200 (43µs)
← GET /api/changelog?since=2026-02-24T01:16:17.023Z&limit=50 200 (57µs)
← GET /api/changelog?since=2026-02-24T01:16:17.031Z&limit=50 200 (67µs)
...
```

## Fix

Stabilize `since` by computing it once with `useMemo`, keyed only on `lastVisitAt`. Because `lastVisitAt` is initialized from `localStorage` in a `useState` initializer and never mutated (no setter is exposed), it is effectively stable for the lifetime of the component. `useMemo` makes the dependency explicit and correct.

```ts
const since = useMemo(
  () => lastVisitAt ?? new Date(Date.now() - SEVEN_DAYS_MS).toISOString(),
  [lastVisitAt]
)
```

With this change:
- When `lastVisitAt` is a stored string → `since` = that string (stable, unchanged).
- When `lastVisitAt` is `null` → `since` = a single timestamp captured on first render and never changed.
- `refresh` is created exactly once per distinct `since` value.
- `useEffect` fires exactly once on mount and again only if `since` changes (which it never does in the null case).

## Acceptance Criteria

1. When `localStorage` has no `sdlc_last_visit_at` key, the app makes exactly **one** initial request to `/api/changelog` on mount (plus one per SSE-triggered refresh).
2. The request rate does not exceed one per SSE event — no tight polling loop.
3. When `lastVisitAt` is set (returning visitor), behavior is unchanged.
4. The `WhatChangedBanner` dismissal flow (`dismiss()`) continues to work — sets localStorage and hides the banner.

## Files Changed

- `frontend/src/hooks/useChangelog.ts` — add `useMemo` import, replace inline `since` with a memoized value.
