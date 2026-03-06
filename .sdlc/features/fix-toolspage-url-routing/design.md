# Design: ToolsPage URL Routing

## Overview

This is a minimal routing refactor — no new components, no layout changes. We replace internal `useState` selection with React Router URL params, matching the pattern already used by ThreadsPage and SpikePage.

## Route Changes (App.tsx)

```tsx
// Before
<Route path="/tools" element={<ToolsPage />} />

// After
<Route path="/tools" element={<ToolsPage />} />
<Route path="/tools/:name" element={<ToolsPage />} />
```

Both routes render the same `ToolsPage` component. The `:name` param is optional — present when a tool is selected, absent for the list view.

## ToolsPage State Changes

### Removed
- `useState<string | null>(selectedName)` — no longer needed
- `selectedNameRef` — no longer needed

### Added
```tsx
import { useParams, useNavigate } from 'react-router-dom'

const { name } = useParams<{ name?: string }>()
const navigate = useNavigate()
```

### Derived State
```tsx
const selectedTool = tools.find(t => t.name === name) ?? null
```

### Navigation Actions

| Action | Before | After |
|--------|--------|-------|
| Select tool | `setSelectedName(tool.name)` | `navigate(\`/tools/${tool.name}\`)` |
| Back to list | `setSelectedName(null)` | `navigate('/tools')` |
| Auto-select first (desktop) | `setSelectedName(data[0].name)` | `navigate(\`/tools/${data[0].name}\`, { replace: true })` |
| Reload after create | `load(name)` with `selectAfterLoad` | `load()` then `navigate(\`/tools/${name}\`)` |

Auto-select uses `{ replace: true }` so the empty `/tools` entry is not left in browser history.

## Desktop Auto-Select Logic

When at `/tools` (no `:name` param), after tools load, if `window.innerWidth >= 768` (md breakpoint) and tools array is non-empty, navigate to `/tools/${tools[0].name}` with `replace: true`. On mobile (< 768px), stay on the list view — user taps to select.

## Edge Cases

- **Invalid tool name in URL**: If `name` param does not match any tool, show the empty state panel (same as current behavior when no tool selected on desktop). No redirect.
- **Tool deleted while viewing**: Same as above — `selectedTool` becomes null, empty state shown.
- **Create tool flow**: After `ToolBuildCompleted`, reload tools and navigate to `/tools/${newToolName}`.

## Files Modified

1. `frontend/src/App.tsx` — add `/tools/:name` route
2. `frontend/src/pages/ToolsPage.tsx` — replace useState with useParams/useNavigate
