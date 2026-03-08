# Spec: Responsive Diff Viewer with @git-diff-view/react and Blue/Amber Palette

## Overview

Add a diff viewer UI component to the Ponder frontend that renders file diffs fetched from the `GET /api/git/diff` endpoint (provided by the `git-diff-api` feature). The viewer uses the `@git-diff-view/react` library for rendering unified and split diff views, styled with a colorblind-safe blue/amber palette that integrates with the existing dark theme.

## Problem

Users currently have no way to view file diffs in the Ponder web UI. The Git status chip shows aggregate counts (dirty, staged, untracked) but cannot drill into individual file changes. Developers and agents need to inspect diffs without leaving the browser.

## Solution

### Diff Viewer Component

A `DiffViewer` React component that:

1. **Accepts a file path** (and optional ref/commit) as props
2. **Fetches diff content** from `GET /api/git/diff?path=<file>&ref=<ref>` (relative URL, same-origin)
3. **Renders the diff** using `@git-diff-view/react` with:
   - Unified view (default) and split view toggle
   - Line numbers on both sides
   - Syntax highlighting via the library's built-in highlighter
4. **Colorblind-safe palette**: additions in blue (`#3b82f6` / blue-500), deletions in amber (`#f59e0b` / amber-500) — avoids red/green which is inaccessible to ~8% of males with color vision deficiency
5. **Responsive layout**: full-width, horizontal scroll for long lines, no wrapping by default with a wrap toggle
6. **Loading and error states**: skeleton loader while fetching, clear error message on failure
7. **Empty state**: "No changes" message when diff is empty

### Integration Points

- The component is a standalone reusable module at `frontend/src/components/DiffViewer.tsx`
- It does NOT add a new route or page — it is a building block for future Git page features
- Consumes the `GET /api/git/diff` endpoint from the `git-diff-api` feature
- Uses existing Tailwind/shadcn design tokens for borders, backgrounds, and text colors

### Styling

- Background: matches existing card backgrounds (`bg-card` / `bg-muted`)
- Gutter/line numbers: `text-muted-foreground` on `bg-muted`
- Added lines: blue-tinted background (`bg-blue-500/10`), blue left-border accent
- Removed lines: amber-tinted background (`bg-amber-500/10`), amber left-border accent
- Header (file path, stats): compact bar with file icon, path, and +/- counts
- Font: monospace (`font-mono`), consistent with existing code display

### Props Interface

```typescript
interface DiffViewerProps {
  filePath: string
  oldRef?: string   // defaults to HEAD
  newRef?: string   // defaults to working tree
  defaultView?: 'unified' | 'split'
}
```

## Out of Scope

- Git page shell / dedicated route (separate feature)
- File browser or tree navigation (separate feature)
- Commit history integration (separate milestone)
- Inline commenting or review annotations
- Multi-file diff views (this component handles one file at a time)

## Acceptance Criteria

1. `DiffViewer` component renders a valid unified diff with correct line numbers
2. Toggle between unified and split view works
3. Blue/amber palette is applied — no red/green for additions/deletions
4. Component handles loading, error, and empty-diff states gracefully
5. Horizontal scrolling works for lines exceeding viewport width
6. Component is responsive and usable at viewport widths down to 768px
7. `@git-diff-view/react` is installed as a production dependency
