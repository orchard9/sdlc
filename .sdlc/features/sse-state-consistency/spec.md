# Spec: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## Problem

Two frontend components have stale-data bugs caused by incomplete SSE wiring:

### Gap 1: `UatHistoryPanel` never refreshes after a UAT run completes

`UatHistoryPanel` fetches the milestone's UAT run list once on mount (inside a
`useEffect`) and never re-fetches. When the user starts a UAT run, the server
eventually emits a `MilestoneUatCompleted` event on the `milestone_uat` SSE
channel. However:

1. `SseContext.tsx` does not handle the `milestone_uat` channel at all — the
   dispatch block covers `update`, `ponder`, `run`, `investigation`, `docs`, and
   `advisory`, but the `milestone_uat` branch is missing. The event is silently
   dropped.
2. Even if the channel were handled, `UatHistoryPanel` has no `useSSE`
   subscription, so it would never receive the signal.

Result: the UAT history list remains empty (or stale) until the user navigates
away and returns.

### Gap 2: `SettingsPage` fires `api.getConfig()` on every generic `update` event

`SettingsPage` calls `useSSE(refresh)` where `refresh = useCallback(() =>
api.getConfig()…, [])`. Every time any file change fires the generic `update`
SSE event (state.yaml, roadmap/, investigations/, escalations/, tools/), the
config endpoint is re-queried even though `.sdlc/config.yaml` changes are rare
and the page shows a diff that doesn't benefit from sub-second latency.

Additionally, on SSE-triggered refreshes, the error state is never cleared
before the new fetch begins, so a previously visible error persists even if the
fresh call succeeds (race: error shows, fetch resolves, `setConfig(data)` is
called but `setError(null)` is not called in the refresh path).

## Goals

1. Add a `milestone_uat` SSE channel handler to `SseContext` and expose a
   corresponding `onMilestoneUatEvent` callback through `useSSE`.
2. Wire `UatHistoryPanel` to `useSSE` so it re-fetches the UAT run list
   whenever a `milestone_uat_completed` event arrives for the correct milestone.
3. Fix `SettingsPage` refresh path to clear error state before the new request
   and avoid unnecessary re-fetches on unrelated `update` events (make it
   subscribe only to `update` events, but clear the error on success — no
   change to event subscription, just fix the error state handling in `refresh`).

## Non-goals

- Changing when or how the server emits `MilestoneUatCompleted`.
- Adding new SSE channels beyond `milestone_uat`.
- Real-time progress streaming inside `UatHistoryPanel` (the panel shows
  completed runs only).

## Affected Files

| File | Change |
|---|---|
| `frontend/src/lib/types.ts` | Add `MilestoneUatSseEvent` type |
| `frontend/src/contexts/SseContext.tsx` | Handle `milestone_uat` channel; add `onMilestoneUatEvent` to `SseCallbacks` |
| `frontend/src/hooks/useSSE.ts` | Add `onMilestoneUatEvent` parameter |
| `frontend/src/components/milestones/UatHistoryPanel.tsx` | Add `useSSE` subscription; re-fetch on matching event |
| `frontend/src/pages/SettingsPage.tsx` | Fix `refresh` to clear error state on success |

## Acceptance Criteria

1. After a milestone UAT run completes, `UatHistoryPanel` automatically shows
   the new run without requiring a page reload or navigation.
2. If no `milestone_uat_completed` event arrives for the current milestone slug,
   the panel does not re-fetch unnecessarily.
3. On `SettingsPage`, a successful SSE-triggered config refresh clears any
   previously displayed error.
4. No existing SSE subscriptions are broken.
5. TypeScript compiles without errors.
