# Design: File Browser Component

## Component Architecture

```
GitPage (shell)
  +-- GitFileBrowser (this feature)
  |     +-- FileBrowserHeader
  |     |     +-- ViewToggle (flat/tree)
  |     |     +-- FilterButtons (modified/all/staged/untracked)
  |     |     +-- FileCount badge
  |     +-- FileList
  |     |     +-- FlatFileRow (flat mode)
  |     |     +-- TreeNode (tree mode)
  |     |           +-- DirectoryRow (expandable)
  |     |           +-- TreeFileRow
  |     +-- EmptyState
  |     +-- LoadingState (Skeleton rows)
  +-- DetailPanel (right, separate feature)
```

## Component Breakdown

### `GitFileBrowser`

Top-level component. Accepts:

```typescript
interface GitFileBrowserProps {
  files: GitFile[]
  loading: boolean
  error: boolean
  onSelect: (file: GitFile) => void
  selectedPath?: string
  onRetry: () => void
}
```

Manages internal state:
- `viewMode: 'flat' | 'tree'` — persisted to localStorage key `git-file-browser-view`
- `filter: 'modified' | 'all' | 'staged' | 'untracked'` — defaults to `'modified'`
- `cursorIndex: number` — keyboard navigation position
- `expandedDirs: Set<string>` — which directories are expanded in tree view

### `FileBrowserHeader`

Renders the panel header bar with:
- Left: "Files" label + count badge showing number of files matching current filter
- Center/Right: Filter pill buttons (M / All / S / U) — compact single-letter labels with tooltips
- Far right: View toggle icon button (list icon for flat, tree icon for tree)

### `FileList`

Renders the scrollable file list. Delegates to flat or tree rendering based on `viewMode`.

The component ref is focused on mount and handles `onKeyDown` for keyboard navigation.

### `FlatFileRow`

```typescript
interface FlatFileRowProps {
  file: GitFile
  selected: boolean
  cursored: boolean
  onClick: () => void
}
```

Layout: `[StatusBadge] [filepath] [staged-dot?]`

- StatusBadge: 16px wide fixed column, single letter, color-coded
- Filepath: truncated from left with `direction: rtl; text-overflow: ellipsis` so the filename (rightmost part) is always visible
- Staged indicator: small emerald dot if `file.staged`

### `TreeNode` (recursive)

```typescript
interface TreeNodeData {
  name: string           // Directory or file name segment
  path: string           // Full path
  isDirectory: boolean
  file?: GitFile         // Present for leaf nodes
  children: TreeNodeData[]
  changedCount: number   // Aggregate count for directories
}
```

- Directories render with a chevron (right when collapsed, down when expanded), name, and a muted count badge.
- Files render identically to FlatFileRow but with indentation (`paddingLeft: depth * 16px`).
- Indent uses `pl-` classes, depth capped at reasonable levels (files deeper than ~8 levels show truncated path).

### `StatusBadge`

Inline component rendering the single-letter git status with appropriate color:

```typescript
function statusColor(status: string): string {
  switch (status) {
    case 'M': return 'text-amber-500'
    case 'A': return 'text-emerald-500'
    case 'D': return 'text-red-500'
    case 'R': case 'C': return 'text-blue-500'
    case '??': return 'text-muted-foreground'
    default: return 'text-muted-foreground'
  }
}
```

## Tree Building Algorithm

Client-side tree construction from flat file list:

```typescript
function buildTree(files: GitFile[]): TreeNodeData {
  const root: TreeNodeData = { name: '', path: '', isDirectory: true, children: [], changedCount: 0 }

  for (const file of files) {
    const parts = file.path.split('/')
    let current = root

    for (let i = 0; i < parts.length; i++) {
      const isLast = i === parts.length - 1
      const partPath = parts.slice(0, i + 1).join('/')

      if (isLast) {
        current.children.push({
          name: parts[i], path: partPath, isDirectory: false,
          file, children: [], changedCount: 0,
        })
      } else {
        let dir = current.children.find(c => c.isDirectory && c.name === parts[i])
        if (!dir) {
          dir = { name: parts[i], path: partPath, isDirectory: true, children: [], changedCount: 0 }
          current.children.push(dir)
        }
        current = dir
      }
    }
  }

  // Recursive sort + count aggregation
  function finalize(node: TreeNodeData): number {
    node.children.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    node.changedCount = node.children.reduce((sum, c) => sum + (c.isDirectory ? finalize(c) : 1), 0)
    return node.changedCount
  }
  finalize(root)

  return root
}
```

Wrapped in `useMemo(files, filter)` to avoid recomputation on every render.

## Keyboard Navigation

Implemented via a `useKeyboardNav` custom hook:

```typescript
function useKeyboardNav(opts: {
  itemCount: number
  onSelect: (index: number) => void
  onToggleView: () => void
  onSetFilter: (f: FilterType) => void
}): {
  cursorIndex: number
  setCursorIndex: (i: number) => void
  handleKeyDown: (e: React.KeyboardEvent) => void
}
```

The hook maps keys to actions and manages cursor position. It prevents default on handled keys to avoid scrolling conflicts.

In tree view, the flattened visible list (respecting collapsed directories) is used for cursor indexing. Expanding/collapsing a directory recalculates the visible list.

## Data Flow

```
useGitFiles() hook
  -> fetch('/api/git/files')
  -> returns { files, loading, error, refetch }

GitPage
  -> useGitFiles()
  -> passes files/loading/error to GitFileBrowser
  -> GitFileBrowser calls onSelect(file)
  -> GitPage updates selectedFile state
  -> Detail panel renders selected file info
```

The `useGitFiles` hook follows the same pattern as `useGitStatus`: poll on interval (10s), pause when tab hidden, refetch on focus.

## Styling

All styling via Tailwind utility classes, consistent with existing components:

- Panel background: `bg-card`
- Borders: `border-border`
- Selected row: `bg-accent text-accent-foreground`
- Cursor (keyboard focus, not selected): `bg-accent/50`
- Hover: `hover:bg-accent/30`
- Text: `text-sm` for file paths, `text-xs` for badges and counts
- Font: `font-mono` for file paths

## Responsive Behavior

- Panel width is fixed at 320px within the master-detail layout.
- On narrow viewports (< 768px), the file browser takes full width and the detail panel is hidden until a file is selected (handled by the shell feature).

## [Mockup](mockup.html)

See the interactive mockup for visual reference of all states.
