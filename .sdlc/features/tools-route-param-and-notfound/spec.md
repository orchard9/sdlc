# Spec: Tools Routing — Rename Param and Not-Found State

## Problem

The `/tools` section has two small gaps in its routing implementation:

1. The route parameter in `App.tsx` is named `:name`, which is ambiguous (name is also used for the tool's display name). The intent is that this param is a stable identifier — renaming it to `:toolId` makes that intent clear.

2. When a user navigates directly to `/tools/<unknown-tool>` (e.g. a stale bookmark, a shared link for a deleted tool), the UI shows a silent empty state ("select a tool from the list") with no indication that the URL was invalid.

## Solution

Two targeted frontend changes — no backend modifications required.

### Change 1: Route param rename

**File:** `frontend/src/App.tsx`

- Change `<Route path="/tools/:name" element={<ToolsPage />} />` to `<Route path="/tools/:toolId" element={<ToolsPage />} />`

**File:** `frontend/src/pages/ToolsPage.tsx`

- Update `useParams<{ name?: string }>()` to `useParams<{ toolId?: string }>()`
- Rename the destructured variable `name` → `toolId` throughout the component
- Update all references: `tools.find(t => t.name === name)` → `tools.find(t => t.name === toolId)`, `navigate(`/tools/${tool.name}`)` stays (tool.name is the value, not the param), etc.
- Update `nameRef` (the ref tracking the current URL param) accordingly

### Change 2: Not-found state

When `toolId` is set (non-empty string from URL) but `selectedTool` is `null` after the tools list has loaded:

- Show an inline message in the right pane: `Tool '{toolId}' not found.`
- The left pane (tool list) remains visible
- No redirect — the user can select a tool from the list

## Behavior

| Scenario | Before | After |
|---|---|---|
| Navigate to `/tools/quality-check` | Opens tool (works) | Opens tool (unchanged) |
| Navigate to `/tools/nonexistent` | Silent empty right pane | "Tool 'nonexistent' not found." |
| Click tool in sidebar | URL updates to `/tools/<name>` | URL updates to `/tools/<name>` (unchanged) |
| Mobile back button | Shows list (works) | Shows list (unchanged) |

## Scope

- `frontend/src/App.tsx` — 1 line change
- `frontend/src/pages/ToolsPage.tsx` — ~10 lines changed
- No backend changes
- No new types or API calls

## Out of Scope

- Adding a rename endpoint for tools
- UUID-based tool IDs (tool names are immutable slugs — stable as IDs)
- Any changes to the ToolCard, ToolRunPanel, or other tool subcomponents
