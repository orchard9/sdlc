# Audit: fleet-management-ui

## Scope
Frontend-only changes: React components, TypeScript types, API client methods, SSE hook extension. No backend changes, no new endpoints, no authentication logic.

## Findings

### A1: PAT field uses type="password" (PASS)
The import form's PAT input uses `type="password"` which prevents shoulder-surfing and browser autocomplete from exposing credentials. The PAT is only sent in the POST body to the server — never stored client-side.

### A2: No client-side credential storage (PASS)
PAT values live only in React component state and are cleared on success. No localStorage, sessionStorage, or cookie storage of sensitive values.

### A3: API calls use relative URLs (PASS)
All API calls use relative paths (`/api/hub/fleet`, `/api/hub/provision`, etc.) which inherit the page's origin. No hardcoded domains or cross-origin requests. Authentication is handled by the auth gate (oauth2-proxy) at the infrastructure layer — this UI does not bypass it.

### A4: Import URL input — no client-side validation (LOW)
The import form accepts any URL string and sends it to the server. Server-side validation is the correct place for this (URL scheme whitelist, SSRF protection). The frontend should not attempt security validation.
**Action:** Accept — server-side validation is the correct boundary. Adding client-side URL validation would be defense-in-depth but not required for v1.

### A5: SSE event parsing uses try/catch (PASS)
Malformed SSE events are caught and silently discarded. No eval or innerHTML from SSE data — all data is parsed as typed JSON objects.

### A6: window.open for instance navigation (PASS)
Instance cards open URLs via `window.open(url, '_blank')`. The URLs come from server-provided data (fleet API response), not user input. `noopener` is the default for `window.open` in modern browsers.

### A7: No XSS vectors (PASS)
All dynamic content is rendered via React JSX (auto-escaped). No `dangerouslySetInnerHTML`, no template literals injected into DOM. Repo names and descriptions are rendered as text nodes.

## Verdict
No security issues found. This is a frontend-only change consuming authenticated API endpoints. The security boundary is correctly at the server/infrastructure layer (oauth2-proxy, fleet-auth-gate).
