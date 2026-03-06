# Spec: ToolsPage URL Routing

## Problem

The ToolsPage currently uses `useState<string | null>` (`selectedName`) to track which tool is selected. This means:

1. **No shareable URLs** — selecting a tool does not update the browser URL, so users cannot link to `/tools/my-tool-name` directly.
2. **No browser history** — back/forward buttons do not navigate between tool selections.
3. **Inconsistency** — ThreadsPage and SpikePage already use `useParams`/`useNavigate` with `/threads/:slug` and `/spikes/:slug` routes. ToolsPage is the odd one out.

## Solution

Replace `useState`-based selection with React Router `useParams` and `useNavigate`:

1. **Add route** — In `App.tsx`, add `<Route path="/tools/:name" element={<ToolsPage />} />` alongside the existing `/tools` route.
2. **Read param** — In `ToolsPage`, call `useParams<{ name?: string }>()` to get the selected tool name from the URL.
3. **Navigate on select** — Replace `setSelectedName(tool.name)` with `navigate(\`/tools/${tool.name}\`)`.
4. **Navigate on back** — Replace `setSelectedName(null)` (the `onBack` handler) with `navigate('/tools')`.
5. **Auto-select first tool** — When the route is `/tools` (no `:name` param) and tools are loaded, navigate to `/tools/${tools[0].name}` on desktop (preserving current auto-select behavior). On mobile, stay on the list view.
6. **Remove `selectedName` state** — The URL param replaces it entirely. `selectedTool` is derived from `tools.find(t => t.name === name)`.

## Scope

- `frontend/src/pages/ToolsPage.tsx` — replace useState with useParams/useNavigate
- `frontend/src/App.tsx` — add `/tools/:name` route

## Out of Scope

- No backend changes.
- No changes to ToolRunPanel, ToolCard, or other sub-components (they receive callbacks, not routing concerns).
- Mobile layout fixes are handled by sibling features in this milestone.

## Acceptance Criteria

- Navigating to `/tools/foo` selects tool "foo" and renders its detail panel.
- Clicking a tool in the sidebar navigates to `/tools/<name>`.
- Browser back button returns to previously selected tool (or tool list).
- `/tools` with no param shows the tool list (mobile) or auto-selects first tool (desktop).
- Existing ToolsPage functionality (run, create, quality check, AMA) is unaffected.
