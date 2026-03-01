# Component Breakdown: SSE Centralization

## SseContext (new file)

**Current state:** Does not exist. Each `useSSE` call owns its own connection loop.

**Required change:** Create `frontend/src/contexts/SseContext.tsx` with:
- `SseCallbacks` interface: `{ onUpdate?, onPonderEvent?, onRunEvent?, onInvestigationEvent?, onDocsEvent?, onAdvisoryEvent? }`
- A subscriber `Set<SseCallbackRefs>` stored in a `useRef` (stable, not state — avoids renders on subscribe/unsubscribe)
- The full `fetch` loop body from the current `useSSE` (reconnect-while-active loop, line parser, `dispatch`)
- Shared debounce timer — one `setTimeout` for `update` events; on fire, calls `.onUpdate()` on every registered subscriber
- Typed events dispatched immediately: iterate subscribers, call the matching typed handler if present
- `subscribe(cbs: SseCallbacks): () => void` — adds to registry, returns unsubscribe function; memoized with `useCallback` so `useSSE`'s effect deps stay stable
- `SseProvider` component and `useSseContext()` hook (throws if used outside provider)

**Why this scope:** The connection loop is the only thing moving. No logic changes — exact same reconnect behavior, same parser, same event types.

**Downstream effects:** All 18 `useSSE` callers implicitly benefit. `App.tsx` must add `<SseProvider>`. `useSSE` must import from this context.

---

## useSSE (modified)

**Current state:** Self-contained hook. Contains: 6 callback params, 6 `useRef` slots, a `useEffect` that syncs refs on every render, and a `useEffect([])` that runs the `fetch` loop. Returns nothing.

**Required change:** Strip out the `fetch` loop and `dispatch` logic entirely. Keep the 6 callback refs (same API, same caller behavior). Replace the loop effect with a single `useEffect`:
```typescript
useEffect(() => {
  return subscribe({
    onUpdate: () => onUpdateRef.current(),
    onPonderEvent: (e) => onPonderRef.current?.(e),
    onRunEvent: (e) => onRunRef.current?.(e),
    onInvestigationEvent: (e) => onInvestigationRef.current?.(e),
    onDocsEvent: (e) => onDocsRef.current?.(e),
    onAdvisoryEvent: (e) => onAdvisoryRef.current?.(e),
  })
}, [subscribe])
```
`subscribe` is stable (memoized in SseContext), so this effect runs once on mount and the return value (unsubscribe) fires on unmount. Net result: same hook API, zero connection.

**Why this scope:** The ref pattern is worth keeping — it means callers still don't need `useCallback`. Removing it would be a breaking change for all 18 call sites.

**Downstream effects:** None — callers are unaffected. `SseContext.tsx` must be imported.

---

## App.tsx (modified)

**Current state:**
```tsx
<BrowserRouter>
  <AgentRunProvider>
    <AppShell>...</AppShell>
  </AgentRunProvider>
</BrowserRouter>
```

**Required change:** Add `SseProvider` wrapping `AgentRunProvider`:
```tsx
<BrowserRouter>
  <SseProvider>
    <AgentRunProvider>
      <AppShell>...</AppShell>
    </AgentRunProvider>
  </SseProvider>
</BrowserRouter>
```
`SseProvider` must be above `AgentRunProvider` because `AgentRunContext` calls `useSSE` which now calls `useSseContext()`.

**Why this scope:** One line added, one import added. The provider order matters — this is the only structural constraint.

**Downstream effects:** None — `AgentRunContext` continues calling `useSSE` unchanged, it just goes through context now.
