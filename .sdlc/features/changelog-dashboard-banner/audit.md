# Security Audit: changelog-dashboard-banner

## Scope

This audit covers the three files introduced/modified by this feature:
- `frontend/src/hooks/useChangelog.ts`
- `frontend/src/components/layout/WhatChangedBanner.tsx`
- `frontend/src/pages/Dashboard.tsx` (minor wiring change)

## Attack Surface Analysis

### 1. localStorage Access

**Pattern**: `localStorage.getItem('sdlc_last_visit_at')` / `localStorage.setItem('sdlc_last_visit_at', ...)`

**Risk level**: LOW

**Analysis**:
- The key `sdlc_last_visit_at` is read and written only within the same origin (SPA). No cross-origin access.
- The value stored is always `new Date().toISOString()` — generated client-side, never echoed from server input. No injection vector.
- The value is later used as a URL query parameter: `new URLSearchParams({ since, limit: '50' })`. Since `Date.toISOString()` produces a well-formed ISO 8601 string, no malformed characters can be injected into the URL. If a malicious actor manually set the localStorage key to an arbitrary string (e.g. via browser DevTools), the worst outcome is a malformed `since` parameter to the API — the server must validate it independently (which is `changelog-api`'s responsibility).
- **Finding**: If a user manually corrupts `sdlc_last_visit_at` in localStorage to inject arbitrary query string content, the API request may behave unexpectedly. However, since this is a user manipulating their own browser storage against their own session on their own local SDLC instance, this is not a meaningful threat.

**Action**: No change needed. Out-of-scope threat for a local-only tool.

### 2. Fetched Data Rendering

**Pattern**: Event fields (`kind`, `slug`, `title`, `timestamp`) rendered in JSX.

**Risk level**: LOW

**Analysis**:
- React's JSX escapes all string values by default — no `dangerouslySetInnerHTML` is used anywhere in the component.
- `event.kind` is rendered as a string inside a `<span>`. Even if the API returned an unexpected kind value, it would render as literal text.
- `event.title` is rendered inside a `<span className="truncate">` — React handles HTML escaping. No XSS risk.
- `event.slug` is interpolated in a `Link to` prop: `` `/features/${event.slug}` ``. If `event.slug` contained `../` or other path traversal characters, React Router would navigate to an unintended route within the SPA. Since SDLC slugs are validated as URL-safe identifiers by the server, and this is a local tool, this is an acceptable risk.

**Action**: No change needed.

### 3. API Endpoint (`GET /api/changelog`)

**Pattern**: `fetch('/api/changelog?since=<ts>&limit=50')`

**Analysis**:
- Uses the same `fetch` + relative URL pattern as all other API calls in this codebase.
- No authentication tokens are handled here; auth is managed at the transport layer (the sdlc-server `auth.rs` middleware applies uniformly to all `/api/*` routes).
- Error responses (non-2xx, 404) are handled gracefully — no sensitive error information is displayed to the user.

**Action**: No change needed.

### 4. No Server-Side Changes

This feature is entirely frontend. No new Rust endpoints, no new YAML files, no new authentication paths. The security surface is limited to the frontend behavior described above.

## Summary

| Area | Risk | Finding | Action |
|---|---|---|---|
| localStorage read/write | Low | Value is always a client-generated ISO timestamp; no server input echoed | Accept |
| JSX rendering of API data | Low | React auto-escapes; no `dangerouslySetInnerHTML` | Accept |
| Fetch call pattern | Low | Matches existing codebase patterns; auth handled by middleware | Accept |
| Server-side surface | None | No backend changes | N/A |

## Verdict: PASS

No security issues of concern. This is a pure frontend display component with no meaningful security attack surface for a local development tool.
