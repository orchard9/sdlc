# Plan: Tools Page Routing Cleanup

## Problem

The `/tools` section has routing via `/tools/:name`, but two small gaps exist:
1. The route param is named `:name` instead of the more intentional `:toolId`
2. When a URL contains an unknown tool name (e.g. a stale bookmark), the UI silently shows the empty "select a tool" placeholder — no feedback to the user

## What to Build

### Milestone: `tools-routing-cleanup`
**Title:** Tools page routing cleanup — param naming and not-found state
**Vision:** A user who navigates directly to `/tools/some-tool` (via bookmark, shared link, or browser history) sees a clear "not found" message if the tool doesn't exist, rather than a silent empty pane.

### Feature: `tools-route-param-and-notfound`
**Title:** Tools routing: rename param to toolId and add not-found state
**Scope:** Frontend only — two targeted changes

**Tasks:**
1. In `frontend/src/App.tsx`: rename route from `/tools/:name` to `/tools/:toolId`
2. In `frontend/src/pages/ToolsPage.tsx`:
   - Update `useParams<{ toolId?: string }>()` and rename `name` → `toolId` throughout
   - When `toolId` is set but `selectedTool` is null after load, render "Tool 'X' not found." inline in the right pane (not a redirect — the list stays visible)

**Acceptance:**
- Navigating to `/tools/quality-check` opens that tool (existing behavior preserved)
- Navigating to `/tools/nonexistent-tool` shows "Tool 'nonexistent-tool' not found." in the right pane with the list still visible on the left
- Selecting a tool from the sidebar still updates the URL
- Mobile back navigation still works
