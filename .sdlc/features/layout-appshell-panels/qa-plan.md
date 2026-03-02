# QA Plan: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Scope

This QA plan covers the two independent layout features delivered in this milestone:

1. **Sidebar icon rail collapse** — `Sidebar.tsx` state, toggle, transition, tooltips, persistence.
2. **AgentPanel drag-to-resize** — `AgentPanel.tsx` resize handle, clamping, persistence.

## Test Environment

- Desktop viewport: 1280×800 (primary) and 1440×900.
- Mobile viewport: 390×844 (iPhone 14 simulation via DevTools).
- Browser: Chrome (primary), Firefox (regression).
- `sdlc ui` running (`npm run dev` in `frontend/` or `sdlc ui`).
- `localStorage` cleared before persistence tests.

## QA Checklist

### Sidebar — Toggle

- [ ] QA-1: Clicking the `ChevronsLeft` button collapses the sidebar. Width animates from 224 px to 56 px over ~200 ms.
- [ ] QA-2: Clicking the `ChevronsRight` button expands the sidebar. Width animates back to 224 px.
- [ ] QA-3: Toggle button is visible in both states (never hidden).
- [ ] QA-4: No layout overflow on `main` content area after collapse — content area fills the remaining width.
- [ ] QA-5: No horizontal scrollbar appears on the page after collapse or expand.

### Sidebar — Collapsed State UI

- [ ] QA-6: Group labels (`work`, `plan`, `setup`, etc.) are not visible when collapsed.
- [ ] QA-7: Nav item labels are not visible when collapsed.
- [ ] QA-8: All nav icons are visible and centered in the collapsed rail.
- [ ] QA-9: `Fix Right Away` and `Search` buttons show only icons (no label, no `kbd` shortcut) when collapsed.
- [ ] QA-10: Active nav item is still visually highlighted (accent background) when collapsed.

### Sidebar — Tooltips

- [ ] QA-11: Hovering each nav icon in collapsed state shows a tooltip on the right side with the correct label.
- [ ] QA-12: Hovering `Fix Right Away` icon shows "Fix Right Away" tooltip.
- [ ] QA-13: Hovering `Search` icon shows "Search" tooltip.
- [ ] QA-14: Tooltips do NOT appear in expanded state (only shown when collapsed).

### Sidebar — Navigation

- [ ] QA-15: Clicking every nav icon in collapsed state navigates to the correct route.
- [ ] QA-16: Active state updates correctly after navigation in collapsed mode.

### Sidebar — Persistence

- [ ] QA-17: Collapse the sidebar → reload the page → sidebar opens in collapsed state.
- [ ] QA-18: Expand from collapsed → reload → sidebar opens in expanded state.
- [ ] QA-19: `localStorage.getItem('sdlc:sidebar-collapsed')` is `"true"` when collapsed, `"false"` when expanded (verify in DevTools).

### Sidebar — Mobile (regression)

- [ ] QA-20: At mobile viewport (< 768 px), the sidebar is not visible by default (slide-in overlay behavior unchanged).
- [ ] QA-21: Tapping the menu hamburger button opens the mobile sidebar overlay as before.
- [ ] QA-22: Sidebar `collapsed` state does not affect the mobile overlay behavior.
- [ ] QA-23: `BottomTabBar` is visible and functional on mobile.

### AgentPanel — Resize Handle

- [ ] QA-24: Hovering the left edge of the AgentPanel shows `col-resize` cursor.
- [ ] QA-25: The resize handle has a subtle hover highlight (accent color).
- [ ] QA-26: Dragging left (widening panel) increases panel width smoothly.
- [ ] QA-27: Dragging right (narrowing panel) decreases panel width smoothly.
- [ ] QA-28: Panel width never goes below 200 px (min clamp enforced during drag).
- [ ] QA-29: Panel width never exceeds 520 px (max clamp enforced during drag).
- [ ] QA-30: Panel content (header, run cards) reflows correctly at all widths within the [200, 520] range.

### AgentPanel — Persistence

- [ ] QA-31: Resize to ~350 px → reload → panel reopens at ~350 px.
- [ ] QA-32: `localStorage.getItem('sdlc:agent-panel-width')` reflects the current width (verify in DevTools after drag).
- [ ] QA-33: If `localStorage` value is corrupt or out of range, panel defaults to 288 px (not NaN, not 0, not overflow).

### AgentPanel — Fullscreen Modal

- [ ] QA-34: Opening the fullscreen modal (`Maximize2` button) shows the full-screen overlay.
- [ ] QA-35: No resize handle is visible/active inside the fullscreen modal.
- [ ] QA-36: Closing the fullscreen modal returns to the normal panel at the last resized width.

### AgentPanel — Open/Close

- [ ] QA-37: Closing the AgentPanel (`PanelRightClose`) and re-opening (`PanelRightOpen` button) restores the last width.
- [ ] QA-38: The `PanelRightOpen` thin toggle strip is unaffected by the resize feature.

### AgentPanel — Mobile (regression)

- [ ] QA-39: At mobile viewport, the AgentPanel is not visible (it is `hidden` on small screens).
- [ ] QA-40: `AgentPanelFab` still appears and the slide-out drawer still works on mobile.

### TypeScript / Build

- [ ] QA-41: `cd frontend && npx tsc --noEmit` produces zero errors.
- [ ] QA-42: `cd frontend && npm run build` completes successfully.
- [ ] QA-43: No browser console errors related to the new components.

## Regression Scenarios

- [ ] QA-44: Dashboard page renders correctly at all sidebar states.
- [ ] QA-45: A detail page (e.g., `/features/:slug`) renders correctly with sidebar collapsed.
- [ ] QA-46: Search modal (`⌘K`) opens and closes correctly regardless of sidebar state.
- [ ] QA-47: Fix Right Away modal (`⌘⇧F`) opens and closes correctly regardless of sidebar state.
- [ ] QA-48: SSE live updates continue to arrive and update agent run cards in the AgentPanel during a resize.

## Pass Criteria

All 48 checks must pass. Any failure blocks merge.
