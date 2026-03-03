# Security Audit: Dashboard Horizon Zone

## Feature Summary

Implements Zone 3 (Horizon) on the Dashboard — a read-only display component that
shows upcoming milestones and active ponders. Two files changed:

- `frontend/src/components/dashboard/HorizonZone.tsx` (new implementation)
- `frontend/src/pages/Dashboard.tsx` (prop threading only)

## Security Surface

This is a pure frontend read/display feature. No new API endpoints. No new
authentication or authorization paths. No new data writes. No backend changes.

## Threat Analysis

### 1. Data Display — XSS Risk

**Finding**: Milestone titles, ponder titles, and tag strings are rendered as React
children (e.g., `{m.title}`, `{p.title}`, `{tag}`). React automatically escapes
string values inserted via JSX expressions — there is no use of `dangerouslySetInnerHTML`.

**Verdict**: No XSS risk. React's JSX escaping is the defense-in-depth layer here.

### 2. Clipboard API — `navigator.clipboard.writeText`

**Finding**: The `CopyButton` component writes `/sdlc-ponder <slug>` to the clipboard.
The `slug` value originates from the API response (`PonderSummary.slug`). The slug
is a server-controlled identifier — not user-supplied input in the browser.

**Potential concern**: If a malicious slug contained newlines or escape sequences, the
clipboard content could be misleading if pasted into a terminal. However:
- Slugs are kebab-case identifiers produced by the sdlc server from YAML files. The
  server enforces slug format (alphanumeric + hyphens) at creation time.
- The clipboard content is always prefixed with `/sdlc-ponder ` — a static string
  the user can visually verify on paste.
- Clipboard writes require user gesture (button click) and browser permission.

**Verdict**: No exploitable risk given the slug format invariant. Accepted.

### 3. Navigation Links — Open Redirect Risk

**Finding**: Links use `react-router-dom` `<Link>` with interpolated slugs:
`to={/milestones/${m.slug}}` and `to={/ponder/${p.slug}}`. These are client-side
SPA routes — they do not trigger full page navigation or HTTP redirects.

**Verdict**: No open redirect risk. SPA-internal routes cannot redirect to external
domains.

### 4. Network Fetch — `api.getRoadmap()`

**Finding**: `HorizonZone` calls `api.getRoadmap()` on mount. This uses the same
`fetch()` wrapper as all other API calls in the app, with identical authentication
headers and error handling. Errors are caught silently, which is safe (the component
degrades gracefully rather than exposing error details to the UI).

**Verdict**: No new security surface. Same auth/session model as existing calls.

### 5. Information Disclosure

**Finding**: The Horizon zone exposes milestone titles, ponder titles, and ponder tags
to anyone who can see the Dashboard. This is consistent with the rest of the Dashboard
(Zone 2 shows milestone titles too). No new data categories are exposed.

**Verdict**: No information disclosure regression.

## Findings Summary

| # | Finding | Severity | Action |
|---|---------|----------|--------|
| 1 | XSS via title/tag rendering | None (React escaping) | Accepted |
| 2 | Clipboard content from slug | None (slug format invariant) | Accepted |
| 3 | Open redirect via route links | None (SPA-internal) | Accepted |
| 4 | New API fetch surface | None (same auth model) | Accepted |
| 5 | Information disclosure | None (consistent with existing zones) | Accepted |

## Verdict

APPROVED — No security findings. The feature has a minimal, well-understood security
surface consisting entirely of read-only display and a clipboard write of server-
controlled data through existing authenticated fetch infrastructure.
