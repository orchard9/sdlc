# Review: ToolsPage URL Routing

## Files Changed

### `frontend/src/App.tsx`
- Added `<Route path="/tools/:name" element={<ToolsPage />} />` — consistent with the pattern used by `/ponder/:slug`, `/threads/:slug`, `/spikes/:slug`, etc.

### `frontend/src/pages/ToolsPage.tsx`
- Added `useParams` and `useNavigate` imports from `react-router-dom`
- Removed `selectedName` useState and replaced with `useParams<{ name?: string }>()`
- Tool selection: `navigate(\`/tools/${tool.name}\`)` instead of `setSelectedName(tool.name)`
- Back button: `navigate('/tools')` instead of `setSelectedName(null)`
- Auto-select first tool on desktop: `navigate(\`/tools/${data[0].name}\`, { replace: true })` — uses `replace` to avoid polluting browser history
- `load()` callback uses refs for `navigate` and `name` to keep stable dependency array (avoids re-fetching tools on every URL change)
- `selectedTool` derived from `tools.find(t => t.name === name)`

## Findings

1. **No issues found.** The change is minimal and follows the exact pattern used by other pages in the app.
2. **TypeScript compiles cleanly** — no errors after changes.
3. **Stable load callback** — `navigateRef` and `nameRef` prevent `load` from being recreated on every URL change, avoiding unnecessary API calls.
4. **Sub-components unaffected** — ToolRunPanel, ToolCard, CreateToolModal receive the same callback signatures.

## Verdict

Approved — clean, minimal refactor that brings ToolsPage in line with every other list/detail page in the app.
