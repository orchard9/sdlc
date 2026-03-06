# Tasks: layout-dead-code-cleanup

## T1 — Delete BottomTabBar.tsx

The `BottomTabBar` component is defined in `frontend/src/components/layout/BottomTabBar.tsx` but is never imported or used anywhere in the codebase. Delete the file.

## T2 — Fix AgentPanelFab vestigial bottom offset

In `frontend/src/components/layout/AgentPanelFab.tsx` line 20, the FAB button has `bottom-[56px]` which was a hard-coded offset to float above the BottomTabBar (h-11 = 44px). Since BottomTabBar is no longer rendered, change this to `bottom-4` so the FAB sits at a standard 16px offset from the bottom of the viewport.
