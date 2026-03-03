# QA Results: Fix Changelog Polling Infinite Loop on null lastVisitAt

## TC-1: No infinite loop on first visit (null lastVisitAt) — PASS

**Verification (static):** `since` is now wrapped in `useMemo([lastVisitAt])`. `lastVisitAt` is initialized from `localStorage` in a `useState` initializer with no setter exposed — it is stable for the component lifetime. `useCallback([since])` therefore creates `refresh` exactly once. `useEffect([refresh])` fires exactly once on mount. No re-render cycle is possible from this path.

The root cause (new string on every render → new callback → effect fires → state update → re-render) is eliminated by construction.

## TC-2: Returning visitor behavior unchanged — PASS

**Verification (static):** When `lastVisitAt` is a stored string, `useMemo` returns it directly (`lastVisitAt ?? ...` short-circuits). The `since` value equals the stored string — identical to the previous behaviour. `useCallback` and `useEffect` remain stable. One request on mount, one per SSE event.

## TC-3: `since` value stable across renders — PASS

**Verification (static):** `useMemo` memoizes the factory result. Because `lastVisitAt` never changes (no setter), the memo never re-runs. All changelog requests within a session will share the same `since` value — not advancing millisecond-by-millisecond as in the bug report.

## TC-4: Dismiss flow works — PASS

**Verification (static):** `dismiss()` is unchanged:
```ts
const dismiss = useCallback(() => {
  localStorage.setItem(STORAGE_KEY, new Date().toISOString())
  setDismissed(true)
}, [])
```
No dependency on `since`. Sets `localStorage` and flips `dismissed` to `true`. `WhatChangedBanner` hides on `dismissed === true`. Unaffected by this change.

## TC-5: TypeScript compilation — PASS

```
$ cd frontend && npx tsc --noEmit
(no output — zero errors)
```

## Summary

| Test Case | Result |
|---|---|
| TC-1: No infinite loop (null lastVisitAt) | PASS |
| TC-2: Returning visitor unchanged | PASS |
| TC-3: `since` value stable | PASS |
| TC-4: Dismiss flow | PASS |
| TC-5: TypeScript build | PASS |

All 5 test cases pass. The fix is correct and complete.
