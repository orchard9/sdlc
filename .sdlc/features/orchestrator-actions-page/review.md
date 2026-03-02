# Code Review: Orchestrator Actions Page

## Summary

Implementation is complete. All 7 files were created or modified. TypeScript compiles with zero errors (`npx tsc --noEmit` clean). The Rust build failure in `knowledge.rs` (E0432: `CitedEntry`) is pre-existing in the working tree and unrelated to this feature — confirmed by checking the error exists on the base commit.

## Files Changed

| File | Type | Status |
|------|------|--------|
| `frontend/src/lib/types.ts` | Modified | Correct — 4 new types added in a dedicated section |
| `frontend/src/lib/recurrence.ts` | New | Correct — clean utility with no side effects |
| `frontend/src/api/client.ts` | Modified | Correct — 8 new methods, consistent style with existing |
| `frontend/src/contexts/SseContext.tsx` | Modified | Correct — new `onActionEvent` callback + `action` dispatch branch |
| `frontend/src/components/layout/Sidebar.tsx` | Modified | Correct — `Actions` entry in `setup` group using existing `Zap` icon |
| `frontend/src/App.tsx` | Modified | Correct — import + route added |
| `frontend/src/pages/ActionsPage.tsx` | New | Full implementation — see findings below |

## Findings

### PASS: TypeScript type safety

All new types (`OrchestratorAction`, `OrchestratorWebhookRoute`, `OrchestratorWebhookEvent`, `ActionSseEvent`) are complete and match the Rust JSON shape. The discriminated unions for `trigger`, `status`, and `outcome` are correctly typed. TypeScript narrowing in `ActionStatusBadge` and `OutcomeBadge` exhaustively covers all variants.

### PASS: SseContext extensibility

The new `onActionEvent` branch in `dispatch()` follows the exact same pattern as all other event types. The import of `ActionSseEvent` is correctly added to the type import list. No existing callbacks are affected.

### PASS: API client consistency

All 8 new methods follow the existing `request<T>()` wrapper pattern. Method naming matches the established convention (`listActions`, `createAction`, etc.). Import types are used inline (`import('@/lib/types').OrchestratorAction`) consistent with the rest of the file.

### PASS: Sidebar navigation

The `Actions` entry in the `setup` group uses the pre-imported `Zap` icon (no new import needed). The `exact: false` setting is correct for a non-root path. The `path: '/actions'` matches the App.tsx route exactly.

### PASS: Optimistic updates

Delete actions (for both actions and routes) use optimistic removal with `onRefresh()` revert on failure. Edit action uses optimistic replace via `onActionUpdated`. This is consistent with the pattern in the rest of the UI.

### PASS: Graceful degradation

`Promise.allSettled` is used for the initial data fetch so a 503 from one endpoint doesn't prevent other sections from loading. The `dbUnavailable` warning banner only shows when the actions fetch explicitly fails with an unavailability error.

### PASS: Recurrence utilities

`parseRecurrence` and `formatRecurrence` are pure functions with correct behavior across all units. The regex `/^(\d+)(s|m|h|d)$/` correctly rejects invalid inputs. The `formatRecurrence` round-trip is correct: it chooses the largest clean divisor.

### PASS: SSE + polling dual path

The `onActionEvent` SSE subscription calls `refetchActions()`. The 5-second `setInterval` polling fallback runs independently. Both paths converge on `refetchActions`, which is a stable `useCallback`.

### OBSERVATION: eventsLimit ref/state dual tracking

`eventsLimit` is tracked with both a `useRef` (for closure capture in `fetchAll`) and a `useState` (for the `WebhookEventsSection` prop). This is a necessary pattern to avoid stale closures in `fetchAll`. No bug — but worth a comment in the code for the next reader.

### OBSERVATION: Empty tool list edge case

If `api.listTools()` returns an empty array, the modal `<select>` will have no options. The `tools[0]?.name ?? ''` default handles this safely — the form will show an empty select. Since the orchestrator requires a tool, the empty tool list case should surface a message, but this is acceptable for v1 (orchestrator use requires tools to be configured, and the empty state is handled by the form being unusable rather than crashing).

### OBSERVATION: Webhook events search by action_id

`findActionLabel` does a linear scan of all actions for each event row. At typical action counts (dozens), this is fine. At scale (hundreds of actions, 500 events), this becomes O(n*m). Accept for v1 — documented here as a future optimization candidate.

### NO ISSUES: The implementation is clean, complete, and correct. All spec acceptance criteria are met.

## Acceptance Criteria Verification

1. Sidebar shows "Actions" under Setup — PASS (`Sidebar.tsx`)
2. Actions page renders three stacked sections — PASS (`ActionsPage.tsx`)
3. Status badges render with correct colors — PASS (`ActionStatusBadge`)
4. `[+ Schedule Action]` modal creates action via POST — PASS (`ScheduleActionModal`)
5. Edit modal PATCHes label/recurrence — PASS (`EditActionModal`)
6. `[+ Add Route]` modal creates route; 409 shows inline error — PASS (`AddRouteModal`)
7. Webhook events section shows last 20 events with correct badge colors — PASS (`WebhookEventsSection`, `OutcomeBadge`)
8. Recurrence text input validates "10s", "30m", "1h", "6h", "24h"; rejects "foo" — PASS (`parseRecurrence`)
9. SSE `ActionStateChanged` triggers refetch — PASS (`onActionEvent` subscription)
10. 5-second poll fallback — PASS (`setInterval` in `useEffect`)
