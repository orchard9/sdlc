# Spec: Git Section in Sidebar under Integrate with Master-Detail Layout

## Overview

Add a "Git" nav entry to the sidebar under the **integrate** group and create a `GitPage` shell component that uses the existing `WorkspaceShell` master-detail layout. This page serves as the container for the git file browser milestone — the left pane will host the file list (built by `git-file-browser-ui`) and the right pane will show file details/diffs.

## Requirements

### Sidebar Navigation

- Add a "Git" entry to the `integrate` nav group in `Sidebar.tsx`, positioned after "Network".
- Use the `GitBranch` icon from lucide-react (already imported in Sidebar).
- Route: `/git` with `exact: false` so sub-routes like `/git/:path` highlight correctly.

### Route Registration

- Register `/git` and `/git/*` routes in `App.tsx` pointing to a new `GitPage` component.
- The wildcard route allows deep-linking to specific files/directories.

### GitPage Shell Component

- Create `frontend/src/pages/GitPage.tsx`.
- Use `WorkspaceShell` for the master-detail layout:
  - **Left pane (list):** Placeholder content for now — a heading "Files" and empty state text. The `git-file-browser-ui` feature will populate this.
  - **Right pane (detail):** Placeholder content — empty state prompting to select a file. The git diff viewer milestone will populate this.
- `showDetail` should be driven by whether a file path is selected (from URL params or local state).
- Page should display the current branch name and git status summary at the top of the list pane, using the existing `/api/git/status` endpoint via `useGitStatus` hook.

### Responsive Behavior

- On mobile, show only the list or detail pane (handled by `WorkspaceShell`).
- Back navigation from detail to list on mobile.

## Out of Scope

- Actual file listing (handled by `git-files-api` + `git-file-browser-ui`).
- Diff viewing (handled by the git-diff-viewer milestone).
- Commit history (handled by the git-commit-history milestone).

## Acceptance Criteria

1. A "Git" link appears in the sidebar under the "integrate" group.
2. Clicking "Git" navigates to `/git` and renders the `GitPage` component.
3. The page uses `WorkspaceShell` with a left list pane and right detail pane.
4. The list pane header shows the current branch name and status from `/api/git/status`.
5. Both panes show appropriate empty-state placeholders.
6. The sidebar "Git" entry highlights correctly when on `/git` or `/git/*` routes.
7. The app compiles and runs without errors.
