# Security Audit: Hub UI

## Scope

Frontend-only changes: `HubPage.tsx`, `useHubSSE.ts`, additions to `App.tsx`, `types.ts`, `api/client.ts`. No Rust server code was modified.

## Findings

### F1: `window.open(url, '_blank')` — Open Redirect / Tab Hijacking

**Severity:** Low
**Area:** `HubPage.tsx` — `ProjectCard` click handler

The card click opens the project URL using `window.open(url, '_blank')`. The URL comes from `GET /api/hub/projects`, which is server-controlled data. In hub mode, the hub registry only receives URLs via `POST /api/hub/heartbeat`, which requires authentication (same auth middleware as all other endpoints).

- Attacker would need valid auth credentials to register a malicious URL.
- The URL is not user-input — it arrives from the server registry.
- `_blank` opens in a new tab; the opener relationship could expose `window.opener` in older browsers.

**Resolution:** Accept — the threat model requires authenticated access to register projects. Adding `rel="noopener noreferrer"` is not applicable to `window.open()` directly, but the risk is negligible given auth gating. No action needed.

### F2: Hub Mode Detection — Information Disclosure

**Severity:** Informational
**Area:** `App.tsx` — `useHubMode` hook

The hub mode detection probe (`GET /api/hub/projects`) returns different responses based on server mode. This is by design — the frontend must know which mode it is in. No sensitive information is leaked in the 503 response body (`{ error: "not running in hub mode" }`).

**Resolution:** Accept — this is intended behavior.

### F3: SSE Connection — Reconnect Loop

**Severity:** Low
**Area:** `useHubSSE.ts` — reconnect logic

The hook retries every 3 seconds on connection failure. This matches the pattern in `SseContext.tsx`. There is no exponential backoff, but the server endpoint is local (hub mode runs on the same host), so flood risk is minimal.

**Resolution:** Accept — matches existing pattern in `SseContext.tsx`. Consistent behavior is more important than backoff here.

### F4: No XSS vectors

Project data rendered in the page:
- `project.name` — rendered as text via React (auto-escaped).
- `project.url` — rendered as text inside a `<div>` (auto-escaped), and used in `window.open()` — URL is server-controlled.
- `project.active_milestone` — rendered as text (auto-escaped).
- `project.feature_count` — number, formatted as text.

No `dangerouslySetInnerHTML` used anywhere in the new code.

**Resolution:** No action needed.

## Summary

No blocking findings. All four items are either accepted by design or informational. The implementation is consistent with existing patterns in the codebase.
