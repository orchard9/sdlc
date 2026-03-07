# Spec: Dynamic Browser Tab Title

## Overview

Set the browser tab title (`document.title`) dynamically based on the current page context using the pattern: **PROJECT · FOCUS · Ponder**.

## Current State

- `index.html` has a static `<title>sdlc</title>` that never changes during navigation.
- `AppShell` already has `projectName` (from config) and `titleFromPath(pathname)` which maps routes to human-readable labels (e.g. `/milestones` -> "Milestones", `/ponder` -> "Ponder").
- No `document.title` manipulation exists anywhere in the frontend.

## Requirements

1. **Title format**: `{projectName} · {pageFocus} · Ponder` where:
   - `projectName` = project name from config (e.g. "sdlc")
   - `pageFocus` = the page label from `titleFromPath()` (e.g. "Milestones", "Features", "Dashboard")
   - "Ponder" = constant brand suffix
2. **Update on navigation**: Title must update whenever `location.pathname` changes.
3. **Detail pages**: For detail routes like `/features/:slug` or `/milestones/:slug`, the focus segment should include the slug formatted as a readable name (e.g. "my-feature" -> "my-feature · Features").
4. **Fallback**: If projectName is not yet loaded, use "Ponder" as the project segment.
5. **Hub mode**: Hub page should set title to "Ponder Hub".

## Scope

- Single `useEffect` in `AppShell` that sets `document.title` based on `projectName`, `location.pathname`, and `titleFromPath()`.
- No new dependencies. No new components. No new API calls.
- Approximately 10-15 lines of code.

## Out of Scope

- Favicon changes per page
- Notification counts in title
- SEO meta tags (this is a SPA)
