# Design: Diff Viewer Component

## Component Architecture

```
DiffViewer (main component)
  +-- DiffHeader        (file path, +/- stats, view toggle)
  +-- DiffContent       (wraps @git-diff-view/react DiffView)
  +-- DiffSkeleton      (loading state)
  +-- DiffError         (error state)
  +-- DiffEmpty         (no-changes state)
```

All sub-components are internal to the `DiffViewer.tsx` file — no separate files until complexity demands it.

## Data Flow

```
Props { filePath, oldRef?, newRef? }
        |
        v
  fetch(`/api/git/diff?path=${filePath}&old=${oldRef}&new=${newRef}`)
        |
        v
  Parse response: { diff_text: string, additions: number, deletions: number, is_binary: boolean }
        |
        v
  Feed diff_text into @git-diff-view/react's DiffView component
        |
        v
  Render with custom theme (blue/amber palette)
```

## Hook: useDiff

```typescript
function useDiff(filePath: string, oldRef?: string, newRef?: string) {
  // Returns { diff, loading, error }
  // Fetches from /api/git/diff with query params
  // Re-fetches when props change
}
```

Internal to DiffViewer.tsx — not exported as a shared hook unless other components need it.

## @git-diff-view/react Integration

The library provides a `DiffView` component that accepts:
- `diffFile` — parsed diff file object (created from raw diff text via the library's parser)
- `renderWidgetLine` — optional custom line renderer
- `highlight` — syntax highlighting toggle

We use the library's `DiffFile.createFile` to parse the raw unified diff text from the API into the structure the component expects.

## Theme / Palette

CSS custom properties scoped to `.diff-viewer` container:

| Token | Value | Usage |
|---|---|---|
| `--diff-add-bg` | `rgba(59, 130, 246, 0.08)` | Added line background (blue-500 @ 8%) |
| `--diff-add-border` | `#3b82f6` | Added line left accent (blue-500) |
| `--diff-add-text` | `#93c5fd` | Added text highlight (blue-300) |
| `--diff-del-bg` | `rgba(245, 158, 11, 0.08)` | Deleted line background (amber-500 @ 8%) |
| `--diff-del-border` | `#f59e0b` | Deleted line left accent (amber-500) |
| `--diff-del-text` | `#fcd34d` | Deleted text highlight (amber-300) |
| `--diff-gutter-bg` | `var(--muted)` | Line number gutter background |
| `--diff-gutter-text` | `var(--muted-foreground)` | Line number text |

These override the library's default red/green theme via CSS specificity on the `.diff-viewer` wrapper.

## View Toggle

A segmented control in the DiffHeader:
- **Unified** (default) — single column, interleaved adds/deletes
- **Split** — side-by-side old/new columns

State stored in component-local `useState`. No URL persistence needed.

## Responsive Behavior

- `>= 1024px`: split view available, full side-by-side rendering
- `768px – 1023px`: split view force-collapses to unified (too narrow for side-by-side)
- `< 768px`: unified only, header stacks vertically

Horizontal scroll via `overflow-x: auto` on the diff content container. No line wrapping by default; optional wrap toggle in the header.

## Error States

| State | Display |
|---|---|
| Loading | Skeleton: 8 animated bars mimicking code lines |
| Network error | Card with warning icon + "Failed to load diff" + retry button |
| Binary file | Card with file icon + "Binary file — diff not available" |
| Empty diff | Card with check icon + "No changes" |

## File Structure

```
frontend/src/components/DiffViewer.tsx    — component + sub-components + useDiff hook
frontend/src/components/DiffViewer.css    — theme overrides for @git-diff-view/react
```

## Mockup

[Mockup](mockup.html)
