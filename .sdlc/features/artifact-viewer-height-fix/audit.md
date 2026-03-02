# Security Audit: Remove Artifact Height Cap

## Scope

Single CSS class removal in a frontend React component (`ArtifactViewer.tsx`). No backend changes, no new data paths, no new API calls.

## Security Surface Analysis

### Data flow

This change has zero effect on data flow. `ArtifactViewer` receives an `Artifact` prop (already fetched from the server) and renders its `content` field via `MarkdownContent`. Neither the fetch path nor the rendering path changes.

### XSS / content injection

`MarkdownContent` is already responsible for safe rendering of markdown. Removing a height cap does not alter how the markdown is sanitized or rendered. The same content that was rendered before (just clipped) is now rendered at full height. No new injection surface.

### CSS injection

The className string `"p-4"` is a static literal — it cannot be influenced by user input. No CSS injection risk.

### Clickjacking / UI redress

Removing a height cap does not change the page's iframe embedding posture, CSP, or any layout that could be exploited for UI redress. The outer container layout (`max-w-4xl`) is unchanged.

### Denial of service (client-side)

In theory a very large artifact could cause the browser to render more DOM nodes and scroll more content. This is not a new risk — the content was already fetched and parsed by `MarkdownContent`; only the visible clip changed. Users with legitimate access to their own project artifacts can already see this content in fullscreen mode.

### Authentication / authorization

No change. The component is rendered only for authenticated users who have already loaded the feature detail page.

## Findings

None. This is a CSS-only layout change with no meaningful security surface.

## Verdict

**APPROVED.** No security concerns.
