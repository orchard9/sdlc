# QA Plan: Dashboard WhatChangedBanner

## Test Environment

- Frontend dev server: `cd frontend && npm run dev`
- Browser DevTools → Application → Local Storage to inspect/manipulate `sdlc_last_visit_at`
- Browser DevTools → Network to inspect `/api/changelog` requests
- Since `changelog-core` and `changelog-api` may not be implemented, the API will return 404 — the banner must be invisible in that case

## TC-1: Banner hidden when API returns 404

**Setup**: `changelog-api` not yet implemented (or mocked to 404)
**Steps**:
1. Clear `sdlc_last_visit_at` from localStorage
2. Load Dashboard page
**Expected**: No banner rendered — page shows normally without any changelog section

## TC-2: Banner hidden when API returns zero events

**Setup**: Mock `GET /api/changelog` to return `{ events: [], total: 0 }`
**Steps**:
1. Load Dashboard page
**Expected**: No banner rendered

## TC-3: First visit mode — no localStorage key

**Setup**:
- Clear `sdlc_last_visit_at` from localStorage
- Mock `GET /api/changelog` to return 3+ events all within the last 7 days

**Steps**:
1. Load Dashboard page
**Expected**:
- Banner renders with header "Recent project activity"
- `since` parameter in the fetch request is approximately 7 days ago (verify in Network tab)
- Events listed in correct order (run_failed first, then desc timestamp)
- Dismiss button present

## TC-4: Returning user mode — localStorage key set

**Setup**:
- Set `sdlc_last_visit_at = new Date(Date.now() - 2 * 3600000).toISOString()` (2 hours ago) in localStorage
- Mock `GET /api/changelog` to return 3 events since that time

**Steps**:
1. Load Dashboard page
**Expected**:
- Banner renders with header containing "{count} changes since 2 hours ago"
- `since` parameter in fetch matches the `sdlc_last_visit_at` value
- Events listed in correct order

## TC-5: run_failed events appear first

**Setup**: Mock API returning mix of events:
- 1x `run_failed` (oldest timestamp)
- 2x `feature_merged` (newer timestamps)
- 1x `review_approved`

**Expected**: `run_failed` event is listed first despite having the oldest timestamp

## TC-6: Dismiss button behavior

**Setup**: Banner visible (events present)
**Steps**:
1. Note current time before clicking Dismiss
2. Click [Dismiss] button

**Expected**:
- Banner disappears immediately (optimistic hide)
- `localStorage.getItem('sdlc_last_visit_at')` is set to a recent ISO timestamp (within ~1 second of click time)
- On page reload, banner fetches with `since` = the dismissed timestamp (so recent events from after dismiss won't show)

## TC-7: SPA navigation does NOT update last_visit_at

**Setup**:
- Set `sdlc_last_visit_at = new Date(Date.now() - 3600000).toISOString()` (1 hour ago) in localStorage
- Load Dashboard

**Steps**:
1. Note the `sdlc_last_visit_at` value
2. Click on "Features" in the sidebar (navigate away from Dashboard)
3. Click back to Dashboard
4. Inspect `localStorage.getItem('sdlc_last_visit_at')`

**Expected**: Value is unchanged — still the same ISO string from before navigation

## TC-8: Tab close does NOT update last_visit_at

**Setup**:
- Set `sdlc_last_visit_at = "2026-01-01T00:00:00.000Z"` in localStorage

**Steps**:
1. Load Dashboard
2. Close and reopen the browser tab
3. Inspect `localStorage.getItem('sdlc_last_visit_at')`

**Expected**: Value is still `"2026-01-01T00:00:00.000Z"` — unchanged

## TC-9: See X more expansion

**Setup**: Mock API returning 10 events

**Steps**:
1. Load Dashboard (banner shows first 7 events + "See 3 more" button)
2. Click "See 3 more"

**Expected**:
- All 10 events now visible
- No "See more" button / collapse button

## TC-10: run_failed links to /runs; feature_merged links to /features/<slug>

**Setup**: Mock API returning 1 `run_failed` event and 1 `feature_merged` event with slug `"my-feature"`

**Steps**:
1. Click the `run_failed` event row
**Expected**: Navigates to `/runs`

2. Click the `feature_merged` event row
**Expected**: Navigates to `/features/my-feature`

## TC-11: Loading state shows skeleton

**Setup**: Artificially delay the `/api/changelog` response (e.g. 2s)

**Steps**:
1. Load Dashboard

**Expected**: A skeleton row is visible in the banner area during the delay, then replaced by the actual content (or hidden if no events)

## TC-12: SSE re-fetch on state update

**Setup**: Banner visible with 2 events

**Steps**:
1. Trigger an SSE `update` event (any state change)
**Expected**: `GET /api/changelog` is called again; banner updates if new events are present

## Regression

- Dashboard renders normally with no banner when no events exist (no layout shift, no extra whitespace)
- Existing Dashboard sections (escalations, milestones, archive) unaffected
- No JavaScript errors in console when banner mounts/unmounts
