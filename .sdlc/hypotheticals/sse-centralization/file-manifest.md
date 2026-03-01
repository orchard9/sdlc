# File Manifest

## Added

| File | Purpose |
|------|---------|
| `frontend/src/contexts/SseContext.tsx` | Single SSE connection owner. Holds fetch loop, subscriber Set, shared debounce timer, dispatch logic. Exports `SseProvider`, `useSseContext()`, `SseCallbacks` type. |

## Modified

| File | What Changes |
|------|-------------|
| `frontend/src/hooks/useSSE.ts` | Remove: `fetch` loop, `dispatch` fn, `AbortController`, `active` flag, reconnect logic. Keep: 6 callback refs, ref-sync effect. Add: import `useSseContext`, replace loop effect with `useEffect(() => subscribe({...}), [subscribe])` |
| `frontend/src/App.tsx` | Add import `SseProvider` from `@/contexts/SseContext`. Wrap `<AgentRunProvider>` with `<SseProvider>`. Two lines changed. |

## Removed

_(none)_

## Unchanged (notable)

| File | Why Untouched |
|------|--------------|
| `frontend/src/lib/types.ts` | All SSE event types (`PonderSseEvent`, `RunSseEvent`, `InvestigationSseEvent`, `DocsSseEvent`, `AdvisorySseEvent`) are unchanged — they're just interface definitions |
| `frontend/src/contexts/AgentRunContext.tsx` | Already calls `useSSE(noop, undefined, handleRunEvent)` — that call continues to work unchanged since `useSSE` API is preserved |
| All 18 `useSSE` call sites | API is identical — `useSSE(onUpdate, onPonderEvent?, ...)` still works exactly the same from the caller's perspective |
