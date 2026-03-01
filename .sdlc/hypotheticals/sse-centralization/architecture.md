# Architecture: Centralize SSE connections

## What Changes at the System Level

Today, `useSSE` is a self-contained hook: each call opens its own long-lived streaming `fetch`
to `/api/events` and keeps it alive for the lifetime of the calling component. With 18 call
sites active simultaneously, the browser maintains up to 18 parallel streaming connections to
the same server endpoint — all receiving the same events and doing duplicate work.

The structural shift: the connection logic moves out of `useSSE` and into a new `SseContext`
that lives at the app root. `SseContext` owns exactly one connection and maintains a subscriber
registry. `useSSE` becomes a thin registration hook — it registers callbacks with the context on
mount and deregisters on unmount. **The public API of `useSSE` is unchanged.** All 18 call sites
continue to call `useSSE(onUpdate, ...)` with the exact same arguments. They just no longer open
connections.

## Central Components

### 1. SseContext (new)
Owns the single streaming connection and the subscriber registry. Contains the `fetch` loop,
the SSE line parser, the dispatch logic, and the debounce timer. Exposes one function:
`subscribe(callbacks) → unsubscribe`. This is the only new abstraction.

### 2. useSSE (modified)
Becomes a registration adapter. Calls `useSseContext().subscribe(...)` on mount, returns the
unsubscribe function as the cleanup. Retains the existing ref pattern so callers still don't
need to wrap callbacks in `useCallback`. No API change — same 6-parameter signature.

### 3. App.tsx (modified)
Wraps `AgentRunProvider` with `SseProvider`. SseProvider must be above AgentRunProvider because
AgentRunContext calls `useSSE`, which now requires the context.

## Interaction Model

```
App
└─ SseProvider              ← owns: 1 fetch connection, Set<SseSubscriber>
   └─ AgentRunProvider      ← calls useSSE → registers in SseProvider.subscribers
      └─ AppShell
         └─ (pages/hooks)   ← each calls useSSE → registers in SseProvider.subscribers

Server event → SseProvider.dispatch(type, data)
  → debounce timer fires → calls .onUpdate() on every subscriber
  → typed events fire immediately → calls .onRunEvent() etc. on matching subscribers
```

The 500ms `update` debounce moves into `SseContext` as a **shared** timer. Instead of 18
independent debounce clocks, one timer fires and all `onUpdate` subscribers are called together
on the same tick. Behavior is strictly better: fewer fetches, same responsiveness.

## What Does NOT Change

- The `useSSE` call signature — all 18 call sites are untouched
- All typed event interfaces (`PonderSseEvent`, `RunSseEvent`, etc.) in `types.ts`
- `AgentRunContext.tsx` — it already calls `useSSE`; that call continues to work via context
- All page and component files that call `useSSE` — zero changes needed
- The server-side SSE endpoint — this is purely a client refactor
- The ref-based callback pattern — `useSSE` retains refs so callers don't need `useCallback`
