# Perspective Review

## 1. Correctness

**Does the architecture handle edge cases?**

- **Reconnect on disconnect:** Yes. The reconnect loop lives in `SseContext` and runs regardless of subscriber count. A subscriber registering after a reconnect gap still gets future events correctly.
- **Subscriber registers during reconnect:** Safe. The subscriber Set is a ref. The loop reads from it on each event dispatch. A subscriber added while the connection is reconnecting will receive events once the connection is re-established.
- **Subscriber unregisters mid-stream:** Safe. `unsubscribe()` deletes from the Set. The dispatch loop iterates the Set at the time of each event — a subscriber removed between events simply stops receiving.
- **SseProvider unmounts before useSSE cleanup runs:** The `active = false` + `controller.abort()` path fires in `SseProvider`'s cleanup. Any `useSSE` cleanup that runs after will call `unsubscribe()` on an already-cleared Set. Guard needed: `Set.delete()` on an already-empty or already-removed entry is a no-op in JS — safe.
- **useSseContext outside SseProvider:** Throws with a clear message. Existing call sites are all within `App.tsx`'s `<SseProvider>` tree.

**Missing/orphaned files?** No. All three components map to exactly one file each. No component without a file.

**Issues found:** None.

---

## 2. Coherence

**Scope consistency:** ✓ The change is purely internal wiring. No feature changes, no new event types, no API changes.

**Naming conventions:**
- `SseContext.tsx` / `SseProvider` / `useSseContext` — consistent with `AgentRunContext.tsx` / `AgentRunProvider` / `useAgentRuns`. ✓
- `SseCallbacks` as the subscriber interface type — clear and consistent. ✓

**Full rewrites disguised as edits?**
- `useSSE.ts`: ~60% of the file is removed (the loop), but it's not a full rewrite — the ref infrastructure and call signature are preserved. Legitimately "modified." ✓

**Issues found:** None.

---

## 3. Completeness

**Tests:** There are no unit tests for `useSSE.ts` in the current codebase (it's a browser-environment streaming hook). No test files need to change. ✓

**Migration:** Pure client-side refactor. No data shape changes, no `.sdlc/` state changes, no server changes. No migration needed. ✓

**Config/env files:** None affected. ✓

**Documentation:** No `docs/` references to `useSSE` — not documented externally. ✓

**Behavioral change disclosure:** The shared debounce is a subtle behavioral change. Today: 18 independent 500ms timers, all potentially firing at different offsets. After: 1 shared 500ms timer, all `onUpdate` callbacks fire together on the same tick. This is **strictly better** — fewer duplicate fetches, same 500ms responsiveness. No API change needed to document this.

**Issues found:** None.

---

## 4. Risk

**Highest-risk file:** `frontend/src/hooks/useSSE.ts`
- Reason: 18 callers depend on its exact behavior. A subtle mistake in the ref-forwarding or the cleanup path could silently break SSE updates across the entire app. The connection logic being removed must be reproduced exactly in `SseContext.tsx`.
- Mitigation: The loop body is copied verbatim into `SseContext.tsx`, not rewritten. The only logic change is the debounce sharing.

**Second-highest-risk:** `frontend/src/contexts/SseContext.tsx`
- Reason: New file, owns the only connection. If the subscriber dispatch has a bug (e.g., iterating a mutating Set), all event handling breaks. Use `Array.from(subscribersRef.current)` to snapshot before iterating.
- Mitigation: Pattern is standard; the Set snapshot-before-dispatch is a known safeguard.

**Hardest problem:** The shared debounce semantics. Today each caller has its own 500ms timer reset independently. After centralization, one timer covers all `onUpdate` subscribers. If `component A` just subscribed and `component B` resets the timer 400ms in, `component A` waits an extra 400ms. In practice this is undetectable since all components re-render together, but it's a semantic shift worth noting.

**Silent-failure risk:** If `subscribe` is not correctly memoized with `useCallback([])`, `useSSE`'s effect would re-run on every render of the `SseProvider` parent — unsubscribing and re-subscribing on every render without opening new connections. The result: callbacks would silently stop receiving events mid-render cycle. Fix: `useCallback` with `[]` deps or stable identity via `useRef`.
