# QA Results: changelog-dashboard-banner

## Method

The `changelog-api` and `changelog-core` features are not yet deployed, so live end-to-end browser testing is not possible. QA was performed via:

1. **TypeScript compilation** (`npx tsc --noEmit`) — passes with exit 0, zero errors from new files
2. **ESLint** (`npx eslint src/hooks/useChangelog.ts src/components/layout/WhatChangedBanner.tsx`) — clean, no warnings
3. **Code-level verification** — each test case traced through the implementation to verify correctness of behavior

## Test Results

### TC-1: Banner hidden when API returns 404
**Method**: Code review of `useChangelog.ts`
**Result**: PASS
`if (res.status === 404) { setEvents([]); setTotal(0); return }` — events is empty, `WhatChangedBanner` returns null.

### TC-2: Banner hidden when API returns zero events
**Method**: Code review
**Result**: PASS
`if (dismissed || events.length === 0) { return null }` — zero events → null render.

### TC-3: First visit mode — no localStorage key
**Method**: Code review of `useChangelog.ts` and `WhatChangedBanner.tsx`
**Result**: PASS
- `useState(() => localStorage.getItem(STORAGE_KEY))` → null when key absent
- `since = new Date(Date.now() - SEVEN_DAYS_MS).toISOString()` (7 days ago)
- `isFirstVisit = lastVisitAt === null` → banner header renders "Recent project activity"

### TC-4: Returning user mode — localStorage key set
**Method**: Code review
**Result**: PASS
- `lastVisitAt` = stored ISO string; `since` = that string
- `isFirstVisit = false` → banner header renders `"{total} changes since {relativeTime(lastVisitAt)}"`
- `relativeTime()` tested inline: 2-hour-old timestamp → "2 hours ago"

### TC-5: run_failed events appear first
**Method**: Code review of `sortEvents()`
**Result**: PASS
```typescript
const failed = events.filter(e => e.kind === 'run_failed').sort(desc)
const rest = events.filter(e => e.kind !== 'run_failed').sort(desc)
return [...failed, ...rest]
```
Even with the oldest timestamp, `run_failed` events prepend the list.

### TC-6: Dismiss button behavior
**Method**: Code review of `dismiss()` in `useChangelog.ts`
**Result**: PASS
```typescript
const dismiss = useCallback(() => {
  localStorage.setItem(STORAGE_KEY, new Date().toISOString())
  setDismissed(true)
}, [])
```
- Sets `localStorage['sdlc_last_visit_at']` to current ISO timestamp
- Sets `dismissed = true` → `WhatChangedBanner` returns null on same render tick (optimistic)

### TC-7: SPA navigation does NOT update last_visit_at
**Method**: Code review — searched all new files for navigation side effects
**Result**: PASS
`grep -n "useNavigate\|useLocation\|beforeunload\|history\|popstate"` returns no results in `useChangelog.ts` or `WhatChangedBanner.tsx`. Only `dismiss()` writes to localStorage.

### TC-8: Tab close does NOT update last_visit_at
**Method**: Code review
**Result**: PASS
No `beforeunload` event listener anywhere in the new code.

### TC-9: See X more expansion
**Method**: Code review
**Result**: PASS
```typescript
const visible = expanded ? sorted : sorted.slice(0, VISIBLE_COUNT)  // VISIBLE_COUNT = 7
const hiddenCount = sorted.length - VISIBLE_COUNT
{!expanded && hiddenCount > 0 && (
  <button onClick={() => setExpanded(true)}>See {hiddenCount} more</button>
)}
```
With 10 events: shows 7 + "See 3 more". After click: shows all 10, button hidden.

### TC-10: run_failed links to /runs; feature_merged links to /features/<slug>
**Method**: Code review of `EventRow` in `WhatChangedBanner.tsx`
**Result**: PASS
```typescript
const link = event.kind === 'run_failed'
  ? '/runs'
  : event.kind === 'feature_merged'
    ? `/features/${event.slug}`
    : null
```
When `link !== null`, wraps content in `<Link to={link}>`. Otherwise renders a plain div.

### TC-11: Loading state shows skeleton
**Method**: Code review
**Result**: PASS
```typescript
if (loading) {
  return (
    <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6">
      <Skeleton width="w-48" className="h-4" />
    </div>
  )
}
```
Renders skeleton while `loading === true` (initial state before first fetch completes).

### TC-12: SSE re-fetch on state update
**Method**: Code review of `useChangelog.ts`
**Result**: PASS
`useSSE(refresh)` — passes `refresh` as the `onUpdate` callback. Any SSE `update` event triggers `refresh()`, which re-fetches `GET /api/changelog`.

## Build / Static Analysis

| Check | Result |
|---|---|
| `tsc --noEmit` | PASS (exit 0, zero errors from new files) |
| ESLint on new files | PASS (no warnings or errors) |
| No `beforeunload` listener | PASS (grep confirms) |
| No navigation side effects | PASS (grep confirms) |
| React Router `<Link>` used (not `<a href>`) | PASS |
| No `dangerouslySetInnerHTML` | PASS |

## Regression

**Dashboard.tsx change**: Added one import and one `<WhatChangedBanner />` JSX element. When the API returns 404 or empty, the component renders `null` — no layout shift, no extra whitespace. Existing Dashboard sections are unaffected.

## Result: PASS

All 12 test cases pass via code-level verification and static analysis. No blocking issues. Ready for merge.
