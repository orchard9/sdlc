# Spec: Dashboard WhatChangedBanner — since-your-last-visit event feed

## Problem

When a developer returns to the dashboard after time away (or after a long agent run), they have no quick way to know what changed. The dashboard shows current feature/milestone state but gives no "what happened since I was last here" summary. Developers have to navigate to individual features or check runs manually to reconstruct what occurred.

## Solution

Add a `WhatChangedBanner` component to the top of the Dashboard that summarizes project activity since the user's last visit, using `localStorage` to track the timestamp of their last explicit dismissal. The banner has two modes:

1. **Returning user** (has `last_visit_at` in localStorage): Shows count of events + relative time since their last visit. Failed runs (⚠️) are listed first, then other events in descending timestamp order. Up to 7 events are shown inline; if more exist, a "See X more" link appears.

2. **First visit** (no `last_visit_at`): Shows last 7 days of activity under the header "Recent project activity" — giving new developers joining mid-project immediate project context.

## Dismissal Semantics

- **Only the Dismiss button** updates `last_visit_at` in localStorage — setting it to `Date.now()`.
- **SPA navigation** (changing routes within the app) does NOT update `last_visit_at`. The banner persists across route changes until explicitly dismissed.
- **Browser tab close/window close** does NOT update `last_visit_at` either. Only the explicit Dismiss button matters.
- This ensures the banner reflects "since you last acknowledged what changed", not "since you last touched the app".

## Data Source

The banner fetches from `GET /api/changelog?since=<ISO timestamp>&limit=50` (provided by the `changelog-api` feature). The response shape is `{ events: ChangeEvent[], total: number }`.

A `ChangeEvent` has:
```typescript
interface ChangeEvent {
  id: string
  kind: EventKind
  slug: string
  title: string
  timestamp: string  // ISO 8601 UTC
}

type EventKind =
  | 'feature_merged'
  | 'run_failed'
  | 'milestone_wave_completed'
  | 'feature_phase_advanced'
  | 'review_approved'
  | 'audit_approved'
  | 'qa_approved'
```

The banner re-fetches when a `ChangelogUpdated` SSE event arrives.

## Behavior Details

### Returning User Mode

- Header: "**X changes** since [relative time]" (e.g. "12 changes since 2 hours ago")
- Event list: `run_failed` events first (sorted by timestamp desc), then all other events sorted by timestamp desc
- Show at most 7 events inline
- If `total > 7`: show "See X more" link (no navigation target for now — just expands inline)
- Each event shows: icon + kind badge + slug + title
- Icons: ⚠️ for `run_failed`, 🚀 for `feature_merged`, ✓ for `*_approved` kinds, → for `feature_phase_advanced`
- `run_failed` events link to `/runs` page
- `feature_merged` events link to `/features/<slug>`
- Dismiss button: collapses banner, updates `last_visit_at = new Date().toISOString()` in localStorage

### First Visit Mode

- Header: "Recent project activity" (no dismiss option shown — there is nothing to dismiss yet; after first dismiss, `last_visit_at` is set and subsequent visits show returning user mode)
- Shows same event list but fetches `since = 7 days ago` rather than from `last_visit_at`
- Dismiss button still present: sets `last_visit_at` and collapses banner

### Zero Events State

- If no events returned: banner is hidden entirely (don't render it at all)

### Loading State

- Show a skeleton row while fetching

## Positioning

The `WhatChangedBanner` is placed above the main content zones on the Dashboard — specifically, just below the Vision/Architecture missing banner (if shown) and above the stats bar.

## Non-Goals

- No server-side session tracking — purely localStorage-based
- No "mark individual events as read" — banner is all-or-nothing
- No pagination beyond "See X more" inline expansion
- The banner does not appear on any page except Dashboard

## Dependencies

- `changelog-api` feature: provides `GET /api/changelog?since=<ts>&limit=<n>`
- `changelog-core` feature: provides the event data that the API serves
- Both must be implemented before this banner can display live data. The banner should gracefully handle a 404 from `/api/changelog` (show nothing, no error state visible to user).

## Acceptance Criteria

1. Banner renders on Dashboard when `/api/changelog` returns ≥ 1 event
2. Returning user mode shows correct count and relative time
3. `run_failed` events appear before other events in the list
4. Dismiss sets `localStorage.getItem('sdlc_last_visit_at')` to current ISO timestamp
5. SPA route navigation does NOT change `last_visit_at`
6. First-visit mode (no localStorage key) shows "Recent project activity" header with last 7 days
7. If no events, banner is not rendered
8. Banner re-fetches on `ChangelogUpdated` SSE event
9. `run_failed` events link to `/runs`
10. `feature_merged` events link to `/features/<slug>`
