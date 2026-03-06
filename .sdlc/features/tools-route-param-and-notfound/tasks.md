# Tasks: tools-route-param-and-notfound

## Task 1: Rename route param in App.tsx
- [ ] Change `/tools/:name` route to `/tools/:toolId` in `frontend/src/App.tsx`

## Task 2: Update ToolsPage to use toolId param and add not-found state
- [ ] Update `useParams<{ name?: string }>()` → `useParams<{ toolId?: string }>()`
- [ ] Rename `name` → `toolId` throughout the ToolsPage component (URL param variable only — not `tool.name` references)
- [ ] Update the `nameRef` ref to `toolIdRef`
- [ ] When `toolId` is set and `selectedTool` is null after load, render "Tool 'X' not found." in the right pane
