---
session: 2
timestamp: 2026-03-02T06:30:00Z
orientation:
  current: "Architecture confirmed correct — fetch on mount + passive SSE is consistently implemented, infinite reload risk eliminated. Two small consistency gaps remain: UatHistoryPanel has no SSE subscription, SettingsPage has a duplicate mount fetch."
  next: "Create a feature with two tasks: add useSSE(refresh) to UatHistoryPanel, consolidate SettingsPage to call refresh() from its mount effect"
  commit: "Gaps patched — every data-fetching component in the frontend follows the same pattern: fetch on mount, update via SSE passively, no reload loops"
---

**jordan · Owner**
1. follow the code to find this answer
2. query for current state on reloads and remounts, use SSE to keep it passively up to date
3. it was refreshing the browser, i believe we have fixed it
4. clean, consistent, and properly implemented with no tech debt and no chance of infinite reloads

---

## Session 2: Code Investigation — Confirming the Fix

I followed the code top to bottom to answer Jordan's four points. Here's what the implementation actually is.

---

### The architecture: how it works

**`SseContext.tsx`** owns a single `EventSource`-equivalent connection (POST `/api/events`, fetch API). It lives at the app root inside `SseProvider`. The connection is opened once on provider mount and never restarted due to subscriber changes. A `subscribersRef` (a `Set` in a `useRef`) holds all callbacks — stored in a ref so subscribe/unsubscribe never trigger re-renders.

**`useSSE.ts`** is a thin registration hook. Callers pass callbacks; the hook registers them with the context. Critically, all callbacks are held in refs inside `useSSE` itself, so the SSE subscription effect has a single dependency — `subscribe` — which is a `useCallback([])` from the context and is therefore permanently stable. **The subscription effect runs exactly once per component instance.** No re-subscriptions. No reload risk from identity churn.

**Shared debounce**: all `onUpdate` subscribers fire on a single 500ms timer that resets on each incoming event. If a file write triggers three rapid SSE events, components still only re-fetch once.

This is the correct pattern. ⑊ Decided: The SSE architecture has zero infinite-reload risk.

---

### Jordan's four points, answered by the code

**Point 1 — "follow the code"**

Done. Traced: `SseContext` → `useSSE` → `useFeature` / `useProjectState` / `AgentRunContext`. The complete flow for any data-bearing component:

1. Component mounts
2. `useEffect` fires with stable `refresh` callback — fetches from REST API
3. `useSSE(refresh)` registers the same callback as an SSE `onUpdate` subscriber
4. When server writes a file, SSE `update` event fires, debounces 500ms, calls `refresh()` on all subscribers
5. Component re-renders with fresh data from server

Component state is ephemeral (reset on unmount). Context-level state (`AgentRunContext.runs`) persists for the app lifetime.

**Point 2 — "query for current state on reloads and remounts, use SSE to keep it passively up to date"**

This is exactly the implemented pattern, and it's the right pattern. The interesting architectural detail: components do *not* try to derive state from SSE events directly. They use SSE as a "you should refresh" signal and then hit the REST API for authoritative state. This means:

- No event replay concerns — components always get current server state, not a snapshot from when they subscribed
- No stale state if SSE was disconnected during a transition — the next `update` event triggers a fresh fetch
- State is always consistent with server truth

⑊ Decided: Query on mount + SSE as passive refresh trigger is the correct pattern and it is implemented correctly in the majority of the codebase.

**Point 3 — "it was refreshing the browser, I believe we have fixed it"**

Confirmed. There is no mechanism in the current code that could cause infinite reloads. The previous risk was almost certainly one of:

- A `useEffect` with a non-stable callback in its dependency array, causing it to re-run every render
- A `useCallback` that recreated on every render (e.g., capturing state as a dep), causing `useSSE`'s effect to re-register and potentially re-trigger

Neither of these exists in the current code. `subscribe` is `useCallback([])` — permanently stable. `useSSE`'s effect dependency is `[subscribe]` — runs once. Callbacks are held in refs — identity doesn't matter. The 500ms debounce prevents any rapid-fire refresh loops even if multiple SSE events arrive simultaneously.

⑊ Decided: The browser refresh loop is fixed. The implementation is structurally sound.

**Point 4 — "clean, consistent, and properly implemented with no tech debt and no chance of infinite reloads"**

Mostly yes. Two small inconsistencies remain.

---

### Priya Nair (Distributed Systems) weighs in

*On "passive SSE + REST fetch" vs. "SSE as source of truth":*

> The current design is the right one for this system. You're using SSE as a change notification channel, not a state carrier. The REST API is authoritative. This means you never have to worry about event ordering, replay, or missed events causing stale state — a component that mounts after an event fires will still see the correct state on its initial fetch. The only downside is a potential extra round-trip, which is trivially acceptable here.
>
> The alternative — deriving state from SSE events directly — creates a distributed state problem: what is the state if you missed an event? You'd need to implement client-side event replay or state accumulation, and then you have a local cache that can diverge from the server. Don't do that.

⑊ Decided: Passive notification pattern (SSE triggers refresh, REST is truth) is architecturally correct for this system.

---

### Two gaps: not critical, but inconsistent

**Gap 1: `UatHistoryPanel.tsx` has no SSE subscription**

```tsx
// Current — line 43-48
useEffect(() => {
  api.listMilestoneUatRuns(milestoneSlug)
    .then(data => setRuns(sortRunsDescending(data)))
    .catch(() => {})
    .finally(() => setLoading(false))
}, [milestoneSlug])
// No useSSE call
```

This component fetches UAT history on mount but never re-fetches when a UAT run completes. A user watching a UAT run finish won't see the history panel update — they'd have to navigate away and back. This is the original UAT state bug from session 1, still present in the history panel specifically.

Fix: extract a `useCallback` for the fetch, add `useSSE(load)`.

**Gap 2: `SettingsPage.tsx` has a duplicate mount fetch**

```tsx
// Current — lines 12-25
const refresh = useCallback(() => {
  api.getConfig()
    .then(setConfig)
    .catch(err => setError(err.message))
  // NOTE: does not set loading = false
}, [])

useEffect(() => {
  api.getConfig()        // ← duplicate fetch
    .then(setConfig)
    .catch(err => setError(err.message))
    .finally(() => setLoading(false))  // loading handled here separately
}, [])

useSSE(refresh)
```

Two fetches fire on mount. This is a minor inefficiency and inconsistency with the rest of the codebase pattern. Fix: consolidate — have `refresh` manage its own loading state, call `refresh()` from the mount effect.

---

### Aria Chen (Agent Ergonomics) on trust implications

> The original bug report from session 1 was fundamentally about trust erosion. "Spawning agent" appearing twice, UAT button resetting, blocked state not clearing — each individually is ignorable, but together they make users uncertain whether the system is working. The fix now implemented (authoritative server state on every mount) addresses this correctly: the UI can never show stale state longer than one mount/unmount cycle. That's a real trust improvement.
>
> The UatHistoryPanel gap is worth patching specifically because UAT completion is a *visible, user-initiated action*. Users watch it. If they see the run complete in the activity feed but the history panel doesn't update, they'll wonder if it was recorded.

---

### Assessment: ready to converge

The core question from session 1 — "is SSE/state sync authoritative on remount?" — is answered: **yes, for all components except UatHistoryPanel**. The infinite reload regression is fixed. The architecture is correct.

Remaining work is two targeted patches:
1. Add `useSSE(load)` to `UatHistoryPanel` — makes history panel live-update when runs complete
2. Consolidate `SettingsPage` mount fetch — removes duplicate, aligns with project pattern

These are small, well-scoped, and don't require architectural changes. This ponder has done its job.

⑊ Decided: Ponder is converging. Create a feature to patch both gaps, then this is done.
