# Audit: ToolsPage URL Routing

## Security Surface

This feature is a frontend-only routing refactor. No backend changes, no new API calls, no new data flows.

## Findings

### F1: URL parameter injection — No risk
The `name` param from `useParams` is used only as:
1. A lookup key against the in-memory `tools` array (`tools.find(t => t.name === name)`)
2. A CSS class conditional (selected highlight)

It is never interpolated into API calls, HTML attributes, or backend queries. If `name` does not match any tool, `selectedTool` is null and the empty state is shown. No XSS or injection vector.

### F2: No new API surface
No new fetch calls, no new endpoints. The existing `api.listTools()` call is unchanged.

### F3: No sensitive data in URLs
Tool names are non-secret identifiers (e.g., "claude-credentials", "deploy-checker"). Exposing them in the URL bar is consistent with how feature slugs, ponder slugs, and thread slugs are already exposed.

## Verdict

No security concerns. This is a pure UI routing refactor with no meaningful security surface.
