# QA Plan: layout-dead-code-cleanup

## Scope

Two mechanical changes with no runtime behavior impact:
1. Deletion of `BottomTabBar.tsx` (dead file, never imported)
2. CSS class change in `AgentPanelFab.tsx` (`bottom-[56px]` → `bottom-4`)

## Checks

### 1. Build passes

```bash
cd frontend && npm run build
```

No TypeScript errors, no missing imports, no broken references.

### 2. No remaining references to BottomTabBar

```bash
grep -r "BottomTabBar" frontend/src/
```

Expected: zero results (the file is gone and was never imported).

### 3. AgentPanelFab renders at correct position on mobile

Visual inspection: on a mobile viewport (< 768px), the FAB button should float 16px from the bottom edge of the screen (not 56px offset that was for the now-absent BottomTabBar).
