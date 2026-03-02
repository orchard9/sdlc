# Tasks: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## T1 — Add `MilestoneUatSseEvent` type to `types.ts`

**File:** `frontend/src/lib/types.ts`

Add after the `AdvisorySseEvent` interface (around line 469):

```typescript
export interface MilestoneUatSseEvent {
  type: 'milestone_uat_completed'
  slug: string
}
```

**Acceptance:** TypeScript recognises the type; no compile errors.

---

## T2 — Handle `milestone_uat` channel in `SseContext.tsx`

**File:** `frontend/src/contexts/SseContext.tsx`

1. Import `MilestoneUatSseEvent` from `@/lib/types`.
2. Add `onMilestoneUatEvent?: (event: MilestoneUatSseEvent) => void` to `SseCallbacks`.
3. Add dispatch branch after the `advisory` branch:
   ```typescript
   } else if (type === 'milestone_uat') {
     try {
       const event = JSON.parse(data) as MilestoneUatSseEvent
       for (const sub of subs) sub.onMilestoneUatEvent?.(event)
     } catch { /* malformed */ }
   }
   ```

**Acceptance:** Channel events reach registered subscribers; no existing tests broken.

---

## T3 — Add `onMilestoneUatEvent` parameter to `useSSE.ts`

**File:** `frontend/src/hooks/useSSE.ts`

1. Import `MilestoneUatSseEvent`.
2. Add seventh parameter `onMilestoneUatEvent?: (event: MilestoneUatSseEvent) => void`.
3. Add `onMilestoneUatRef` — same ref pattern as all other callbacks.
4. Wire through to `subscribe({ ..., onMilestoneUatEvent: (e) => onMilestoneUatRef.current?.(e) })`.

**Acceptance:** Parameter is optional; all existing `useSSE(...)` call sites compile unchanged.

---

## T4 — Wire `UatHistoryPanel.tsx` to SSE

**File:** `frontend/src/components/milestones/UatHistoryPanel.tsx`

1. Wrap the fetch in a `useCallback` named `load`.
2. Replace the inline `useEffect` fetch with `useEffect(() => { load() }, [load])`.
3. Add `useSSE` call:
   ```typescript
   useSSE(
     () => {},          // no generic update needed
     undefined, undefined, undefined, undefined, undefined,
     (event) => { if (event.slug === milestoneSlug) load() },
   )
   ```
4. Import `useCallback` and `useSSE`.

**Acceptance:** When `milestone_uat_completed` fires for this milestone, the run
list refreshes automatically. Events for other milestones are ignored.

---

## T5 — Fix `SettingsPage.tsx` error-state handling on refresh

**File:** `frontend/src/pages/SettingsPage.tsx`

In the `refresh` callback, add `setError(null)` before the API call and clear
error on success:

```typescript
const refresh = useCallback(() => {
  setError(null)
  api.getConfig()
    .then(data => { setConfig(data); setError(null) })
    .catch(err => setError(err.message))
}, [])
```

**Acceptance:** A previously displayed error is cleared as soon as a successful
SSE-triggered refresh completes. TypeScript compiles.

---

## Sequencing

T1 must land before T2, T3 (type dependency). T2 and T3 can be done in
parallel after T1. T4 depends on T3. T5 is fully independent.

Recommended order: T1 → T2 + T3 (parallel) → T4 → T5.
