# QA Plan: Fix Changelog Polling Infinite Loop on null lastVisitAt

## TC-1: No infinite loop on first visit (null lastVisitAt)

**Setup:** Clear `localStorage` (`localStorage.removeItem('sdlc_last_visit_at')`). Open the app.

**Expected:** The browser devtools Network tab shows exactly one request to `/api/changelog` within the first 2 seconds of page load. No rapid burst of requests with incrementing millisecond timestamps.

**Pass condition:** ≤ 1 changelog request per 5-second window (excluding SSE-triggered refreshes).

## TC-2: Returns visitor behavior unchanged

**Setup:** Set `localStorage.setItem('sdlc_last_visit_at', '2026-02-01T00:00:00.000Z')`. Open the app.

**Expected:** One request to `/api/changelog?since=2026-02-01T00:00:00.000Z&limit=50`. No polling loop.

## TC-3: `since` value is stable across renders

**Verification:** Inspect the network log for the null case — all requests to `/api/changelog` share the same `since` value (same timestamp, not advancing millisecond-by-millisecond).

## TC-4: Dismiss flow works

**Setup:** Null `lastVisitAt`. Confirm banner appears. Click dismiss.

**Expected:** Banner hides immediately. `localStorage.getItem('sdlc_last_visit_at')` is set to a recent ISO timestamp.

## TC-5: TypeScript compilation passes

```bash
cd frontend && npm run build
```

**Expected:** Zero TypeScript errors.
