# Tasks: Fix Changelog Polling Infinite Loop on null lastVisitAt

## T1: Wrap `since` in `useMemo` in `useChangelog.ts`

**File:** `frontend/src/hooks/useChangelog.ts`

Replace the inline computation of `since` (lines 50-52) with a stable `useMemo`:

```ts
// Before
const since = lastVisitAt
  ? lastVisitAt
  : new Date(Date.now() - SEVEN_DAYS_MS).toISOString()

// After
const since = useMemo(
  () => lastVisitAt ?? new Date(Date.now() - SEVEN_DAYS_MS).toISOString(),
  [lastVisitAt]
)
```

Add `useMemo` to the React import line.

## T2: Verify the existing hook unit test covers the null case

**File:** `frontend/src/hooks/useHeatmap.test.ts` (check for changelog tests)

If a test file for `useChangelog` exists, ensure it asserts that `fetch` is called exactly once on mount when `lastVisitAt` is null. If no test exists, this is acceptable — the fix is self-evident and the SSE loop behavior is integration-level.
