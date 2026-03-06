# Tasks: ToolsPage URL Routing

## T1: Add /tools/:name route to App.tsx
Add `<Route path="/tools/:name" element={<ToolsPage />} />` below the existing `/tools` route in `App.tsx`.

## T2: Replace useState with useParams/useNavigate in ToolsPage
- Import `useParams` and `useNavigate` from `react-router-dom`
- Remove `selectedName` useState and `selectedNameRef`
- Read `name` from `useParams<{ name?: string }>()`
- Derive `selectedTool` from `tools.find(t => t.name === name)`
- Replace `setSelectedName(tool.name)` with `navigate(\`/tools/${tool.name}\`)`
- Replace `setSelectedName(null)` with `navigate('/tools')`
- Update auto-select-first-tool logic to use `navigate(\`/tools/${tools[0].name}\`, { replace: true })` on desktop only

## T3: Update tool creation / reload flow
- After `ToolBuildCompleted` SSE event, reload tools list and navigate to the newly created tool's URL
- Ensure `selectAfterLoad` pattern is replaced with post-load navigation
