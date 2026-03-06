# Review: tools-route-param-and-notfound

## Changes Made

### `frontend/src/App.tsx`
- `/tools/:name` → `/tools/:toolId` (1 line)

### `frontend/src/pages/ToolsPage.tsx`
- `useParams<{ name?: string }>()` → `useParams<{ toolId?: string }>()`
- `nameRef` → `toolIdRef` (ref tracking the current URL param)
- `tools.find(t => t.name === name)` → `tools.find(t => t.name === toolId)`
- `tool.name === name` (sidebar selected check) → `tool.name === toolId`
- Added not-found branch: when `toolId` is truthy but `selectedTool` is null, renders "Tool '{toolId}' not found." with "Select a tool from the list" subtitle

## Verification

- No `name` variable references remain in `ToolsPage` (only `tool.name` property accesses, which are correct)
- All `tool.name` references in navigation (`navigate(`/tools/${tool.name}`)`) are unchanged — these refer to the tool's name property, not the URL param
- Not-found state only shows on desktop (right pane is `hidden md:flex` when no tool selected — the `toolId` check is nested inside that pane)
- Mobile UX unchanged: when `toolId` is set and `selectedTool` is null, the right pane shows not-found; when `toolId` is absent the right pane is hidden on mobile

## Findings

No issues found. Changes are minimal and correct.
