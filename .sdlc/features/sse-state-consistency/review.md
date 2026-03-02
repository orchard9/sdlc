# Code Review: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## Summary

Five files changed, ~45 lines net new. All changes are frontend-only. No
backend modifications. TypeScript compiles with zero errors.

---

## File-by-file review

### `frontend/src/lib/types.ts`

Added `MilestoneUatSseEvent`:

```typescript
export interface MilestoneUatSseEvent {
  type: 'milestone_uat_completed'
  slug: string
}
```

- Shape matches exactly what the server emits (`events.rs` line 141–148).
- Type is exported so any future subscriber can import it without duplication.
- **No issues.**

---

### `frontend/src/contexts/SseContext.tsx`

Two changes:

1. `MilestoneUatSseEvent` added to imports and to `SseCallbacks`.
2. `milestone_uat` dispatch branch added after the `advisory` branch, following
   the identical pattern used for all other named channels.

The pre-existing `react-refresh/only-export-components` ESLint error on
`useSseContext` (line 150) is a baseline issue that predates this feature —
confirmed by running ESLint against the HEAD commit before any of my changes.
Not introduced here, not in scope to fix here.

**No issues introduced by this change.**

---

### `frontend/src/hooks/useSSE.ts`

Added `onMilestoneUatEvent` as the seventh (optional) parameter. Follows the
ref-based callback pattern used by all other six parameters exactly: ref
created, ref kept current in the unsynchronised effect, ref forwarded to
`subscribe`. All existing call sites pass ≤ 6 arguments and are unaffected.

**No issues.**

---

### `frontend/src/components/milestones/UatHistoryPanel.tsx`

Before: single `useEffect` with inline fetch, no SSE wiring.

After: `load` wrapped in `useCallback([milestoneSlug])` so it is stable across
renders, `useEffect(() => { load() }, [load])` replaces the inline fetch, and
`useSSE` subscribes to `onMilestoneUatEvent` only — no generic `update`
subscription. The slug guard (`if (event.slug === milestoneSlug)`) ensures
cross-milestone events are ignored.

One nuance: when `load` fires after the initial `loading=true` state, subsequent
SSE-triggered calls do not reset `loading` to `true` (the `finally` sets it to
`false` only). This is intentional — the spinner should only show on initial
mount, not on every background refresh. The existing run list stays visible
during a background re-fetch.

**No issues.**

---

### `frontend/src/pages/SettingsPage.tsx`

Changed `refresh` from:

```typescript
const refresh = useCallback(() => {
  api.getConfig()
    .then(setConfig)
    .catch(err => setError(err.message))
}, [])
```

to:

```typescript
const refresh = useCallback(() => {
  setError(null)
  api.getConfig()
    .then(data => { setConfig(data); setError(null) })
    .catch(err => setError(err.message))
}, [])
```

The pre-call `setError(null)` clears stale errors immediately so the user does
not see a lingering error while a valid request is in-flight. The in-`.then`
`setError(null)` is belt-and-suspenders (redundant but harmless).

**No issues.**

---

## Findings

| ID | Severity | Finding | Action |
|---|---|---|---|
| R1 | Pre-existing | `react-refresh/only-export-components` on `SseContext.tsx:useSseContext` | Accept — baseline issue, not introduced here |

No blockers. No new issues introduced.

## Verdict

APPROVE — implementation is clean, minimal, and follows existing patterns exactly.
