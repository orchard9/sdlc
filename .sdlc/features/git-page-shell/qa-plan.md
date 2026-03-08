# QA Plan: Git Page Shell

## Test Cases

### TC1: Sidebar navigation entry exists
- Navigate to the app.
- Verify "Git" appears in the sidebar under the "integrate" group, after "Network".
- Verify it uses the GitBranch icon.

### TC2: Sidebar link navigates to /git
- Click the "Git" sidebar entry.
- Verify the URL changes to `/git`.
- Verify the GitPage component renders.

### TC3: Sidebar highlights on /git routes
- Navigate to `/git` — verify "Git" entry is highlighted.
- Navigate to `/git/src/main.rs` — verify "Git" entry remains highlighted.
- Navigate to `/` — verify "Git" entry is not highlighted.

### TC4: WorkspaceShell layout renders correctly
- On `/git`, verify the two-pane layout is visible on desktop.
- Left pane shows branch header with severity dot and branch name.
- Right pane shows empty state placeholder.

### TC5: Git status displayed in list pane header
- Verify the branch name from `/api/git/status` is displayed.
- Verify the severity dot reflects the current repo status (green/yellow/red).
- Verify the status summary text is shown.

### TC6: Empty states render appropriately
- Left pane shows "File browser will appear here" or similar placeholder.
- Right pane shows "Select a file to view details" or similar placeholder.

### TC7: Build succeeds
- Run `npm run build` in the frontend directory.
- Verify no compilation errors.
