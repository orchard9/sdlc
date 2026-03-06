# Spike UI Page — Security Audit

## Scope

Frontend-only feature: `SpikePage.tsx`, type additions in `types.ts`, API method additions in `client.ts`, and routing/sidebar changes. No new backend endpoints are introduced by this feature — it consumes existing REST endpoints (`/api/spikes`, `/api/spikes/:slug`, `/api/spikes/:slug/promote`).

## Threat Surface

| Area | Assessment |
|------|-----------|
| Data display | Read-only render of spike data from server. No user-controlled HTML injection — all values are rendered as React text nodes, not `dangerouslySetInnerHTML`. |
| Promote action | `POST /api/spikes/:slug/promote` — slug comes from the URL param, passed through `encodeURIComponent` in the API client. No CSRF risk beyond what the existing auth middleware handles. |
| Navigation after promote | `navigate(/ponder/${result.ponder_slug})` — the ponder_slug comes from the server response, not user input. React Router's `navigate` does not execute arbitrary URLs. |
| Clipboard copy | Uses `navigator.clipboard.writeText(text)` where `text` is the CLI command string built from `spike.slug` (server-provided). No XSS risk. |
| External links (knowledge, ponder) | All links use React Router `<Link to="...">` with server-provided slugs. No `javascript:` or `data:` URIs possible through this pattern. |
| Error messages | Errors from API calls (`err instanceof Error ? err.message : 'string'`) are displayed as React text nodes, not HTML. |

## Findings

### Finding 1 — `ponder_slug` and `knowledge_slug` from server used in routes (Accepted)

The `navigate(/ponder/${result.ponder_slug})` call after promote uses a server-provided slug. An attacker who can control the server response could theoretically inject a malicious path segment. However:
- The server is trusted (same-origin, behind the existing tunnel auth middleware).
- React Router's `navigate` does not interpret path separators as shell commands or execute arbitrary code.
- Worst case is navigation to an unexpected route — not a security vulnerability.

**Action**: Accept. No fix needed.

### Finding 2 — No rate limiting on "Promote to Ponder" button (Tracked)

The promote button is disabled during the in-flight request (`disabled={promoting}`), preventing double-submission. However, there is no backend rate limit on the promote endpoint. This is a backend concern, not a frontend security issue.

**Action**: Accept for this feature. If the backend introduces a rate-limit header, the frontend error handler already displays the server error message.

## Verdict

No exploitable security issues. The feature is frontend-only, renders server data safely as React text nodes, uses `encodeURIComponent` on URL params, and has no user-controlled HTML rendering. The existing auth middleware (tunnel token/cookie) protects all `/api/spikes/*` endpoints.

APPROVED.
