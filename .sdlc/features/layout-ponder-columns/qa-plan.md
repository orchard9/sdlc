# QA Plan: Ponder Layout — CONTEXT and ARTIFACTS Resizable Panels, Mobile Chat/Files/Team Tabs

## Scope

All testing targets `frontend/src/pages/PonderPage.tsx` and
`frontend/src/components/ponder/DialoguePanel.tsx`. No backend changes are involved.

## Static Analysis

- [ ] `cd frontend && npx tsc --noEmit` — zero TypeScript errors.
- [ ] `cd frontend && npm run lint` — zero ESLint errors or warnings introduced by this feature.

## Desktop Layout Tests (viewport ≥ 768 px)

### Context Panel

- [ ] On first load with no localStorage state, the context panel is **expanded** (~200 px).
- [ ] The toggle button collapses the context panel to icon-only width (~32 px).
- [ ] The toggle button expands it again.
- [ ] After collapse/expand, refresh the page — the panel state is **persisted** (same open/closed state survives reload).
- [ ] In expanded state, the panel shows: entry slug (monospace), status badge, team row (if team members exist), orientation strip (if orientation exists).
- [ ] In collapsed state, only the toggle icon is visible — no slug, badge, or team content.
- [ ] `TeamRow` and `OrientationStrip` are **not** visible in the chat column when the context panel is present (desktop).

### Resizable Workspace Panel

- [ ] The drag handle is visible between the chat area and the workspace panel.
- [ ] Dragging the handle left reduces workspace width; dragging right increases it.
- [ ] Width is clamped: cannot go below 160 px, cannot exceed 50% of the detail pane width.
- [ ] After resizing, refresh the page — the workspace width is **persisted** (same width survives reload).
- [ ] With no stored width, the default is 256 px.
- [ ] The drag handle shows `cursor-col-resize` cursor on hover.
- [ ] No content flickers or jumps while dragging.

### Chat Functionality (desktop)

- [ ] Sending a message in the chat works as before.
- [ ] SSE events (ponder run started/completed) update the UI correctly.
- [ ] The commit button functions correctly.
- [ ] The status change modal opens and applies the change.

## Mobile Layout Tests (viewport < 768 px)

### Tab Bar

- [ ] Three tabs are visible at the bottom of the detail pane: Chat, Files, Team.
- [ ] Tapping **Chat** shows the `DialoguePanel` with `InputBar`.
- [ ] Tapping **Files** shows the `WorkspacePanel` (artifact list); `InputBar` is not visible.
- [ ] Tapping **Team** shows the team/orientation panel; `InputBar` is not visible.
- [ ] The active tab has a distinct visual highlight (`text-primary` / `bg-accent/50`).
- [ ] The **Files** tab shows a numeric badge when `entry.artifacts.length > 0`.
- [ ] The old Files toggle button in the entry header is **gone**.
- [ ] The old bottom-sheet overlay and slide-up sheet are **gone**.

### Mobile Chat Functionality

- [ ] On the Chat tab, `TeamRow` and `OrientationStrip` are still visible inside `DialoguePanel` (they are not suppressed on mobile).
- [ ] Sending a message works on mobile.
- [ ] The back button (ArrowLeft) in the header returns to the entry list.

### Mobile Files Tab

- [ ] Artifact list renders correctly in the Files tab.
- [ ] Expanding/collapsing an artifact and viewing its content works.
- [ ] The fullscreen modal for artifact content opens correctly.

### Mobile Team Tab

- [ ] If the ponder entry has team members, `TeamRow` renders in the Team tab.
- [ ] If orientation exists, `OrientationStrip` renders in the Team tab.
- [ ] If neither exists, the panel shows an appropriate empty state (or renders empty gracefully).

## Regression Tests (both desktop and mobile)

- [ ] Navigating between ponder entries resets `mobileTab` to `'chat'` (component key resets state).
- [ ] Advisory panel still opens and closes correctly.
- [ ] New idea form still creates entries and navigates to the new slug.
- [ ] Commit workflow (GitMerge button → commit run → SSE completion) still works end-to-end.
- [ ] No console errors in any tested scenario.

## localStorage Isolation

- [ ] Clearing `localStorage` and reloading: context panel defaults to expanded, workspace defaults to 256 px.
- [ ] Setting `ponder_context_open=false` in `localStorage` before load: panel loads collapsed.
- [ ] Setting `ponder_workspace_width=400` in `localStorage` before load: workspace loads at 400 px.
