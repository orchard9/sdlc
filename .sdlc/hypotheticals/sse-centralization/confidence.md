# Confidence Verdict

## Decision: READY

## Summary

Centralize 18 independent SSE connections into one by extracting the connection loop from
`useSSE` into a new `SseContext` at the app root. The `useSSE` hook's public API is completely
unchanged — all 18 call sites are untouched. Scope is 3 files: 1 new context, 2 small edits.

## Evidence

- Architecture is complete and internally consistent — single connection, subscriber registry, shared debounce
- File manifest covers all three components (SseContext, useSSE, App.tsx) with no orphans or unknowns
- Perspective review found 0 issues; identified 2 safeguards to implement (Set snapshot-before-dispatch, stable `subscribe` ref)
- No unknowns that would invalidate the architecture — all relevant files read, all call sites catalogued
- Zero behavioral changes visible to callers; shared debounce is strictly better (fewer redundant fetches)
- No server changes, no data format changes, no new dependencies

## Risk Notes

- **Highest-risk file:** `frontend/src/hooks/useSSE.ts` — 18 callers; the ref-forwarding and cleanup path must be correct or SSE breaks silently across the app
- **Hardest problem:** Ensuring `subscribe` from `SseContext` has stable identity — must use `useCallback([], [])` or equivalent so `useSSE`'s effect doesn't re-subscribe on every render
- **Silent-failure risk:** If `subscribe` isn't stable, `useSSE` re-runs its effect on every parent render — unsubscribing and re-subscribing without visible errors, callbacks silently miss events

## Implementation Notes

Two concrete safeguards must be in `SseContext.tsx`:
1. Dispatch must snapshot the subscriber set before iterating: `for (const sub of Array.from(subscribersRef.current))`
2. `subscribe` must be memoized: `useCallback(() => { ... }, [])` — stable identity, never recreated

## Next Step

**Next:** `/sdlc-hypothetical-do sse-centralization`
