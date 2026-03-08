# Tasks: File Browser Component

## Task 1: Create useGitFiles hook
Create a `useGitFiles` custom hook in `frontend/src/hooks/useGitFiles.ts` that fetches `GET /api/git/files` and returns `{ files, loading, error, refetch }`. Follow the same polling, visibility-pause, and focus-refetch pattern as `useGitStatus`.

## Task 2: Create StatusBadge component
Create `frontend/src/components/git/StatusBadge.tsx` — a small inline component that renders a single-letter git status badge with appropriate color coding (M=amber, A=green, D=red, R/C=blue, ??=muted).

## Task 3: Create GitFileBrowser component with flat view
Create `frontend/src/components/git/GitFileBrowser.tsx` — the main file browser component. Implement the flat view mode first: panel header with filter buttons, view toggle, file count badge, and a scrollable list of `FlatFileRow` entries. Include filter state management (modified/all/staged/untracked) with modified as default.

## Task 4: Add tree view mode
Add tree view to `GitFileBrowser`. Implement `buildTree` utility to construct a hierarchical `TreeNodeData` structure from flat files. Render recursive `TreeNode` components with expandable directories, indentation, and aggregate change counts. Wire the `f` shortcut and view toggle button to switch between flat and tree.

## Task 5: Add keyboard navigation
Implement `useKeyboardNav` hook and wire it into `GitFileBrowser`. Support j/k/arrows for cursor movement, Enter for selection, f for view toggle, m/a/s/u for filter shortcuts. In tree view, support left/right arrows for collapse/expand. Ensure keyboard events only fire when the panel is focused.

## Task 6: Add loading, empty, and error states
Add skeleton loading rows, empty state (centered message when no files match filter), and error state (message with retry button) to `GitFileBrowser`.

## Task 7: Integrate with Git page shell
Wire `GitFileBrowser` into the Git page's left panel. Connect `onSelect` callback to update the detail panel's selected file. Create the `useGitFiles` hook call at the page level and pass data down. Persist view mode preference to localStorage.
