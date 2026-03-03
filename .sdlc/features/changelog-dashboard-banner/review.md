# Code Review: changelog-dashboard-banner

## Files Changed

- `frontend/src/hooks/useChangelog.ts` (new)
- `frontend/src/components/layout/WhatChangedBanner.tsx` (new)
- `frontend/src/pages/Dashboard.tsx` (modified — import + one JSX line)

## Review

### Correctness

**✓ localStorage semantics are correct.**
`last_visit_at` is read once on mount via `useState` initializer — snapshot semantics, not reactive. Only `dismiss()` writes the new timestamp. No `beforeunload` listener, no `useNavigate`/`useLocation` side effects. SPA navigation does not affect the stored value.

**✓ First visit mode.**
When `localStorage.getItem('sdlc_last_visit_at')` returns `null`, `since` falls back to 7 days ago. The banner header renders "Recent project activity" (via `isFirstVisit === true`). After dismissing from first-visit mode, `last_visit_at` is set and subsequent loads show returning-user mode.

**✓ 404 graceful degradation.**
`useChangelog` catches both 404 status and non-ok responses, silently returning empty results. The banner renders `null` when `events.length === 0`, so no error state is ever shown to the user.

**✓ Event sort order.**
`sortEvents()` partitions events into `run_failed` (first, desc timestamp) and rest (desc timestamp). This matches the spec requirement exactly.

**✓ Expand behavior.**
`VISIBLE_COUNT = 7`. The "See N more" button shows when `!expanded && sorted.length > VISIBLE_COUNT`. Clicking expands the full list. No collapse button after expanding — intentional per spec.

**✓ Dismiss is immediate / optimistic.**
`dismissed` state is set to `true` synchronously in `dismiss()`. The banner renders `null` on the same tick, no flickering or waiting for a re-fetch.

**✓ Links are correct.**
`run_failed` → `/runs`. `feature_merged` → `/features/<slug>`. All other kinds render non-linked rows.

**✓ TypeScript clean.**
`tsc --noEmit` shows zero errors from the new files. The duplicate `import type` that was accidentally placed mid-file was caught and corrected before commit.

### Design / Architecture

**✓ Follows project SSE pattern.**
Uses `useSSE(refresh)` — the generic `onUpdate` callback — for re-fetching on changelog changes. This matches how `useProjectState` works (the same generic update stream). Clean.

**✓ No `useCallback` on `refresh` creating a dependency cycle.**
`refresh` is wrapped in `useCallback` with `[since]` as its dependency. `since` is derived from `lastVisitAt` which is set once on mount and never changes (except after `dismiss()`, which sets `dismissed = true` making the component invisible anyway). So `refresh` identity is stable for the component lifetime. `useEffect(() => { refresh() }, [refresh])` correctly runs once on mount. The `useSSE(refresh)` call re-subscribes on `refresh` identity changes, but since `subscribe` is stable in `SseContext`, no extra subscriptions occur.

**✓ No memory leaks.**
`useSSE` unsubscribes on unmount (returns cleanup function). No dangling fetch promises (the component may unmount mid-fetch but the setState calls are no-ops on unmounted components in React 18).

**✓ Placement in Dashboard.**
Added just after the Vision/Architecture missing banner, before the "Project Overview" section. This follows the design spec and is the correct position for a "what changed" signal.

### Code Quality

**✓ No inline styles, all Tailwind classes.**

**✓ Responsive design.**
Slug and separator are hidden on mobile (`hidden sm:inline`) to keep the row readable on small screens. Icon and kind badge are always visible.

**✓ Accessibility.**
Dismiss button has `aria-label="Dismiss changelog banner"`. `Link` components use proper `to` prop routing (React Router `<Link>`, not raw `<a href>`).

### Findings

**Finding 1 (minor): `hiddenCount` can be negative when `sorted.length <= VISIBLE_COUNT`.**
```typescript
const hiddenCount = sorted.length - VISIBLE_COUNT
```
When `sorted.length` is 3, `hiddenCount` is -4. This is fine because the "See more" button is guarded by `!expanded && hiddenCount > 0`, so it never renders. But the variable name implies it's always non-negative — slightly misleading.

*Action*: Accept as-is. The guard prevents incorrect rendering. A cosmetic fix can go in a follow-up.

**Finding 2 (minor): `total` from API vs `events.length` for display.**
The banner header shows `{total}` (from API response's total field) while the expand logic uses `sorted.length` (from the events array, capped at 50 by the `limit` parameter). These can differ if the API has >50 events since `last_visit_at`. In that case, "See X more" would show the remainder within the 50 limit, not the full count.

*Action*: Acceptable for v1. The `limit=50` is generous for dashboard use. Track as future improvement if needed.

**Finding 3 (minor): `ClipboardList` icon may not be in current lucide-react version.**
The Sidebar already imports `ScrollText` and other icons from lucide-react. `ClipboardList` is a standard lucide icon available since early versions. Confirmed no TS error.

*Action*: No action needed.

## Verdict: APPROVED

The implementation correctly fulfills all 10 acceptance criteria from the spec. No blocking issues found. Minor findings documented above are all acceptable as-is.
