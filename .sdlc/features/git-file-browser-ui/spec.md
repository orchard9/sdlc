# Spec: File Browser Component with Tree/Flat Toggle, Filters, and Keyboard Nav

## Overview

A custom-built React file browser component for the Git page that displays workspace files with their git status. The component supports two view modes (tree and flat), filter presets for common workflows, and full keyboard navigation. It consumes data from the `GET /api/git/files` endpoint (provided by the `git-files-api` feature) and renders within the master-detail layout shell (provided by the `git-page-shell` feature).

## Goals

1. Give developers an at-a-glance view of all changed files in the workspace with git status badges.
2. Support both flat (full-path) and hierarchical (tree) views, togglable at runtime.
3. Provide filter presets so users can focus on modified, staged, or untracked files.
4. Enable efficient keyboard-driven navigation without requiring a mouse.
5. Integrate seamlessly with the existing design system (Tailwind, shadcn patterns, dark theme).

## Dependencies

- **git-files-api**: Provides `GET /api/git/files` returning `{ files: [{ path, status, staged, old_path? }] }`.
- **git-page-shell**: Provides the Git page route, sidebar entry, and master-detail layout where this component renders in the left panel.

## Data Model

The component consumes an array of `GitFile` objects:

```typescript
interface GitFile {
  path: string        // Relative path from workspace root
  status: string      // Git status code: M, A, D, R, C, ??, !!, etc.
  staged: boolean     // Whether the file is in the staging area
  old_path?: string   // Previous path for renamed files
}
```

## View Modes

### Flat View
- Each file displayed as a single row with full relative path.
- Sorted alphabetically by path.
- Default view when filter is "modified only" (fewer files, full paths are more useful).

### Tree View
- Files organized into a collapsible directory hierarchy.
- Directories are expandable/collapsible with click or arrow keys.
- Directory nodes show an aggregate count of changed files within.
- Files sorted alphabetically within each directory level.
- Tree is built client-side from the flat file list by splitting paths on `/`.

Toggle between views via a button in the panel header or the `f` keyboard shortcut.

## Filters

Four filter presets, selectable via buttons or keyboard shortcuts:

| Filter | Shortcut | Description |
|--------|----------|-------------|
| Modified | `m` | Files with status M (default filter) |
| All | `a` | All files with any non-clean status |
| Staged | `s` | Files where `staged === true` |
| Untracked | `u` | Files with status `??` |

The active filter is visually indicated. Changing the filter resets the selection cursor to the first item.

**Default filter: Modified.** This matches the most common developer intent — reviewing what has changed.

## Status Badges

Each file row displays a status badge indicating the git status:

| Status | Badge | Color |
|--------|-------|-------|
| M (modified) | `M` | amber/yellow |
| A (added) | `A` | green |
| D (deleted) | `D` | red |
| R (renamed) | `R` | blue — shows `old_path -> new_path` on hover |
| ?? (untracked) | `?` | gray/muted |
| C (copied) | `C` | blue |

Staged files get an additional visual indicator (e.g., a small dot or left-border accent) to distinguish them from unstaged changes.

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `j` / `ArrowDown` | Move cursor to next file |
| `k` / `ArrowUp` | Move cursor to previous file |
| `Enter` | Select file (triggers detail view in right panel) |
| `f` | Toggle flat/tree view |
| `m` | Filter: modified only |
| `a` | Filter: all |
| `s` | Filter: staged |
| `u` | Filter: untracked |

In tree view, additional keys:

| Key | Action |
|-----|--------|
| `ArrowRight` | Expand directory / move to first child |
| `ArrowLeft` | Collapse directory / move to parent |

Keyboard shortcuts only fire when the file browser panel has focus (not when typing in other inputs). The panel is focusable and captures keyboard events via `onKeyDown`.

## Selection

- Single selection model — one file selected at a time.
- Selected file is highlighted with the `bg-accent` style.
- Selection triggers an `onSelect(file: GitFile)` callback passed from the parent layout, which updates the right detail panel.
- On initial load or filter change, no file is selected (detail panel shows empty state).

## Panel Layout

- The file browser occupies the left panel of the master-detail layout.
- Panel width: 280-350px range, initially 320px.
- Panel header contains: view toggle button, filter buttons, file count badge.
- File list scrolls independently within the panel.
- Empty state when no files match the active filter: centered message like "No modified files" with a muted icon.

## Loading & Error States

- **Loading**: Skeleton rows (3-5 shimmer bars) while the API call is in-flight.
- **Error**: "Failed to load files" message with a retry button.
- **Empty repo** (no files at all): "No files in workspace" message.

## Performance Considerations

- The file list is rendered as a simple scrollable list (no virtualization needed — typical workspaces have <500 changed files).
- Tree structure is computed via `useMemo` from the flat file list, recomputed only when files or view mode changes.
- Filter application is a simple array `.filter()` — no complex indexing needed.

## Non-Goals

- No file content preview in this component (that is the detail panel's responsibility).
- No staging/unstaging from the UI (view-only for this milestone).
- No drag-and-drop.
- No multi-select.
- No file search/fuzzy-find within the browser (may be added later).

## Acceptance Criteria

1. File browser renders in the left panel of the Git page master-detail layout.
2. Flat view shows all files with full paths, sorted alphabetically.
3. Tree view shows files in a collapsible directory hierarchy.
4. Pressing `f` toggles between flat and tree view.
5. Four filter presets (Modified, All, Staged, Untracked) work correctly via buttons and keyboard shortcuts.
6. Modified is the default filter on page load.
7. Each file displays a color-coded status badge matching its git status.
8. Keyboard navigation (j/k/Enter/arrows) works when the panel has focus.
9. Selecting a file calls `onSelect` with the file data.
10. Loading state shows skeleton placeholders.
11. Empty state displays when no files match the active filter.
12. Component uses the existing design system (Tailwind classes, cn utility, consistent with sidebar/card patterns).
