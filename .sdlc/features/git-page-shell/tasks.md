# Tasks: Git Page Shell

## T1: Add Git entry to sidebar integrate group
Add `{ path: '/git', label: 'Git', icon: GitBranch, exact: false }` to the `integrate` nav group in `Sidebar.tsx`, after "Network".

## T2: Create GitPage component with WorkspaceShell layout
Create `frontend/src/pages/GitPage.tsx` using `WorkspaceShell` with:
- Left pane: branch header (severity dot + branch name + status text from `useGitStatus`) and empty-state placeholder for file list.
- Right pane: empty-state placeholder prompting to select a file, or selected file path display.
- `showDetail` driven by URL wildcard param.
- Mobile back button when detail is shown.

## T3: Register /git routes in App.tsx
Add `<Route path="/git" element={<GitPage />} />` and `<Route path="/git/*" element={<GitPage />} />` to App.tsx with the GitPage import.

## T4: Verify build compiles without errors
Run `cd frontend && npm run build` to confirm the frontend compiles cleanly with the new page and sidebar entry.
