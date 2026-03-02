---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Three distinct UI state bugs with a shared root cause — SSE/state not authoritative on remount"
  next: "Read the server SSE implementation and the frontend useSSE hook to understand what happens on component remount"
  commit: "Root cause identified, fix proposed, and all three bug variants addressed with a single architectural change or targeted patches"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

Three bug instances, likely one root cause:

1. **Spawning status regression**: Ponder panel shows "spawning agent" after collapse/expand even when agent is already running. State is in component memory, not rehydrated from server on remount.
2. **UAT button state after completion**: Button resets to "Run UAT" after completing instead of advancing. Jordan has a local fix — need to understand what it is and generalize it.
3. **Blocked state stale after resolution**: After a dependency is resolved externally, the UI still shows the feature as blocked until manually triggered.

### Why this might matter

Each of these individually is annoying. Together, they erode trust. A user who sees "spawning agent" twice for the same run thinks the run started twice. A user who sees "Run UAT" after UAT completed thinks it failed. These trust-erosion bugs compound: users lose confidence in the entire state model.

### Open questions

- What is the source of truth for "is an agent currently running" — server RunRecord status, or SSE stream state?
- On component remount, does the frontend query the server for current state, or rely on SSE events it may have missed?
- What was Jordan's local UAT fix? Does it apply to the other two cases?
- Should the frontend implement a "fetch-on-mount" pattern for all stateful components that don't currently do this?

### Suggested first exploration

Read `frontend/src/` for the useSSE hook and how the ponder/feature panels consume it. Specifically: what happens when a component that was previously mounted unmounts and remounts — does it re-subscribe to SSE? Does it fetch current state from the REST API?
