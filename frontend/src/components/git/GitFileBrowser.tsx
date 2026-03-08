import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { cn } from '@/lib/utils'
import { List, FolderTree, ChevronRight, ChevronDown, AlertCircle, RotateCcw } from 'lucide-react'
import { StatusBadge } from './StatusBadge'
import type { GitFile } from '@/hooks/useGitFiles'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type FilterType = 'modified' | 'all' | 'staged' | 'untracked'
export type ViewMode = 'flat' | 'tree'

interface GitFileBrowserProps {
  files: GitFile[]
  loading: boolean
  error: boolean
  onSelect: (file: GitFile) => void
  selectedPath?: string
  onRetry: () => void
}

interface TreeNodeData {
  name: string
  path: string
  isDirectory: boolean
  file?: GitFile
  children: TreeNodeData[]
  changedCount: number
}

// ---------------------------------------------------------------------------
// Filter logic
// ---------------------------------------------------------------------------

const FILTERS: { key: FilterType; label: string; shortcut: string }[] = [
  { key: 'modified', label: 'M', shortcut: 'm' },
  { key: 'all', label: 'All', shortcut: 'a' },
  { key: 'staged', label: 'S', shortcut: 's' },
  { key: 'untracked', label: 'U', shortcut: 'u' },
]

function applyFilter(files: GitFile[], filter: FilterType): GitFile[] {
  switch (filter) {
    case 'modified':
      return files.filter(f => f.status === 'M')
    case 'staged':
      return files.filter(f => f.staged)
    case 'untracked':
      return files.filter(f => f.status === '??')
    case 'all':
    default:
      return files
  }
}

// ---------------------------------------------------------------------------
// Tree building
// ---------------------------------------------------------------------------

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
          name: parts[i],
          path: partPath,
          isDirectory: false,
          file,
          children: [],
          changedCount: 0,
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

  function finalize(node: TreeNodeData): number {
    node.children.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    node.changedCount = node.children.reduce(
      (sum, c) => sum + (c.isDirectory ? finalize(c) : 1),
      0,
    )
    return node.changedCount
  }
  finalize(root)

  return root
}

/** Flatten tree into a visible list respecting expanded state. */
function flattenTree(root: TreeNodeData, expanded: Set<string>): { node: TreeNodeData; depth: number }[] {
  const result: { node: TreeNodeData; depth: number }[] = []

  function walk(node: TreeNodeData, depth: number) {
    for (const child of node.children) {
      result.push({ node: child, depth })
      if (child.isDirectory && expanded.has(child.path)) {
        walk(child, depth + 1)
      }
    }
  }
  walk(root, 0)
  return result
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

const VIEW_MODE_KEY = 'git-file-browser-view'

export function GitFileBrowser({ files, loading, error, onSelect, selectedPath, onRetry }: GitFileBrowserProps) {
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    try { return (localStorage.getItem(VIEW_MODE_KEY) as ViewMode) || 'flat' } catch { return 'flat' }
  })
  const [filter, setFilter] = useState<FilterType>('modified')
  const [cursorIndex, setCursorIndex] = useState(0)
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set())
  const listRef = useRef<HTMLDivElement>(null)

  // Persist view mode
  useEffect(() => {
    try { localStorage.setItem(VIEW_MODE_KEY, viewMode) } catch { /* noop */ }
  }, [viewMode])

  // Filter files
  const filtered = useMemo(() => {
    const result = applyFilter(files, filter)
    return result.sort((a, b) => a.path.localeCompare(b.path))
  }, [files, filter])

  // Build tree
  const tree = useMemo(() => buildTree(filtered), [filtered])
  const flatTree = useMemo(() => flattenTree(tree, expandedDirs), [tree, expandedDirs])

  // Items for cursor navigation
  const itemCount = viewMode === 'flat' ? filtered.length : flatTree.length

  // Clamp cursor when item count changes
  useEffect(() => {
    setCursorIndex(prev => Math.min(prev, Math.max(0, itemCount - 1)))
  }, [itemCount])

  // Reset cursor on filter change
  const handleFilterChange = useCallback((f: FilterType) => {
    setFilter(f)
    setCursorIndex(0)
  }, [])

  const toggleViewMode = useCallback(() => {
    setViewMode(prev => prev === 'flat' ? 'tree' : 'flat')
    setCursorIndex(0)
  }, [])

  const toggleDir = useCallback((path: string) => {
    setExpandedDirs(prev => {
      const next = new Set(prev)
      if (next.has(path)) next.delete(path)
      else next.add(path)
      return next
    })
  }, [])

  // Get the file at the current cursor position
  const getFileAtCursor = useCallback((): GitFile | undefined => {
    if (viewMode === 'flat') {
      return filtered[cursorIndex]
    }
    const entry = flatTree[cursorIndex]
    return entry?.node.file
  }, [viewMode, filtered, flatTree, cursorIndex])

  // Keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    // Ignore if typing in an input
    const tag = (e.target as HTMLElement).tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault()
        setCursorIndex(prev => Math.min(prev + 1, itemCount - 1))
        break
      case 'k':
      case 'ArrowUp':
        e.preventDefault()
        setCursorIndex(prev => Math.max(prev - 1, 0))
        break
      case 'Enter': {
        e.preventDefault()
        const file = getFileAtCursor()
        if (file) onSelect(file)
        break
      }
      case 'f':
        e.preventDefault()
        toggleViewMode()
        break
      case 'm':
        e.preventDefault()
        handleFilterChange('modified')
        break
      case 'a':
        e.preventDefault()
        handleFilterChange('all')
        break
      case 's':
        e.preventDefault()
        handleFilterChange('staged')
        break
      case 'u':
        e.preventDefault()
        handleFilterChange('untracked')
        break
      case 'ArrowRight':
        if (viewMode === 'tree') {
          const entry = flatTree[cursorIndex]
          if (entry?.node.isDirectory && !expandedDirs.has(entry.node.path)) {
            e.preventDefault()
            toggleDir(entry.node.path)
          } else if (entry?.node.isDirectory && expandedDirs.has(entry.node.path)) {
            // Move to first child
            e.preventDefault()
            setCursorIndex(prev => Math.min(prev + 1, itemCount - 1))
          }
        }
        break
      case 'ArrowLeft':
        if (viewMode === 'tree') {
          const entry = flatTree[cursorIndex]
          if (entry?.node.isDirectory && expandedDirs.has(entry.node.path)) {
            e.preventDefault()
            toggleDir(entry.node.path)
          }
        }
        break
    }
  }, [itemCount, getFileAtCursor, onSelect, toggleViewMode, handleFilterChange, viewMode, flatTree, cursorIndex, expandedDirs, toggleDir])

  // Scroll cursor into view
  useEffect(() => {
    const container = listRef.current
    if (!container) return
    const row = container.querySelector(`[data-index="${cursorIndex}"]`)
    if (row) row.scrollIntoView({ block: 'nearest' })
  }, [cursorIndex])

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <div
      className="flex flex-col h-full bg-card"
      tabIndex={0}
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2.5 border-b border-border shrink-0">
        <span className="text-sm font-semibold text-foreground">Files</span>
        <span className="text-[11px] bg-muted text-muted-foreground px-1.5 py-0.5 rounded-full">
          {loading ? '--' : filtered.length}
        </span>
        <div className="flex gap-1 ml-auto">
          {FILTERS.map(f => (
            <button
              key={f.key}
              onClick={() => handleFilterChange(f.key)}
              title={`${f.key} (${f.shortcut})`}
              className={cn(
                'text-[11px] px-2 py-0.5 rounded border transition-colors',
                filter === f.key
                  ? 'bg-accent text-accent-foreground border-border'
                  : 'text-muted-foreground border-transparent hover:text-foreground hover:border-border',
              )}
            >
              {f.label}
            </button>
          ))}
        </div>
        <button
          onClick={toggleViewMode}
          title={`Switch to ${viewMode === 'flat' ? 'tree' : 'flat'} view (f)`}
          className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
        >
          {viewMode === 'flat'
            ? <List className="w-3.5 h-3.5" />
            : <FolderTree className="w-3.5 h-3.5" />
          }
        </button>
      </div>

      {/* Content */}
      <div ref={listRef} className="flex-1 overflow-y-auto overflow-x-hidden">
        {loading ? (
          <LoadingState />
        ) : error ? (
          <ErrorState onRetry={onRetry} />
        ) : filtered.length === 0 ? (
          <EmptyState filter={filter} />
        ) : viewMode === 'flat' ? (
          filtered.map((file, i) => (
            <FlatFileRow
              key={file.path}
              file={file}
              index={i}
              selected={file.path === selectedPath}
              cursored={i === cursorIndex}
              onClick={() => onSelect(file)}
            />
          ))
        ) : (
          flatTree.map((entry, i) => (
            entry.node.isDirectory ? (
              <DirectoryRow
                key={entry.node.path}
                node={entry.node}
                depth={entry.depth}
                index={i}
                expanded={expandedDirs.has(entry.node.path)}
                cursored={i === cursorIndex}
                onToggle={() => toggleDir(entry.node.path)}
              />
            ) : (
              <TreeFileRow
                key={entry.node.path}
                node={entry.node}
                depth={entry.depth}
                index={i}
                selected={entry.node.file?.path === selectedPath}
                cursored={i === cursorIndex}
                onClick={() => entry.node.file && onSelect(entry.node.file)}
              />
            )
          ))
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function FlatFileRow({ file, index, selected, cursored, onClick }: {
  file: GitFile; index: number; selected: boolean; cursored: boolean; onClick: () => void
}) {
  return (
    <div
      data-index={index}
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 px-3 py-1.5 cursor-pointer text-sm transition-colors',
        selected ? 'bg-accent text-accent-foreground' : 'text-foreground',
        cursored && !selected && 'bg-accent/50',
        !selected && !cursored && 'hover:bg-accent/30',
      )}
    >
      <StatusBadge status={file.status} />
      <span className="flex-1 font-mono text-xs truncate" title={file.path}>{file.path}</span>
      {file.staged && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0" title="Staged" />}
    </div>
  )
}

function DirectoryRow({ node, depth, index, expanded, cursored, onToggle }: {
  node: TreeNodeData; depth: number; index: number; expanded: boolean; cursored: boolean; onToggle: () => void
}) {
  return (
    <div
      data-index={index}
      onClick={onToggle}
      className={cn(
        'flex items-center gap-1.5 py-1.5 cursor-pointer text-sm text-muted-foreground hover:text-foreground hover:bg-accent/30 transition-colors',
        cursored && 'bg-accent/50',
      )}
      style={{ paddingLeft: `${12 + depth * 16}px`, paddingRight: '12px' }}
    >
      {expanded
        ? <ChevronDown className="w-3 h-3 shrink-0" />
        : <ChevronRight className="w-3 h-3 shrink-0" />
      }
      <span className="text-xs font-medium">{node.name}/</span>
      <span className="ml-auto text-[11px] text-muted-foreground/60">{node.changedCount}</span>
    </div>
  )
}

function TreeFileRow({ node, depth, index, selected, cursored, onClick }: {
  node: TreeNodeData; depth: number; index: number; selected: boolean; cursored: boolean; onClick: () => void
}) {
  return (
    <div
      data-index={index}
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 py-1.5 cursor-pointer text-sm transition-colors',
        selected ? 'bg-accent text-accent-foreground' : 'text-foreground',
        cursored && !selected && 'bg-accent/50',
        !selected && !cursored && 'hover:bg-accent/30',
      )}
      style={{ paddingLeft: `${12 + (depth + 1) * 16}px`, paddingRight: '12px' }}
    >
      <StatusBadge status={node.file?.status ?? ''} />
      <span className="flex-1 font-mono text-xs truncate" title={node.path}>{node.name}</span>
      {node.file?.staged && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0" title="Staged" />}
    </div>
  )
}

function LoadingState() {
  return (
    <div className="space-y-0.5 p-1">
      {[70, 85, 60, 75, 50].map((w, i) => (
        <div key={i} className="flex items-center gap-2 px-3 py-1.5">
          <div className="w-4 h-4 rounded bg-muted animate-pulse" />
          <div className="h-3.5 rounded bg-muted animate-pulse" style={{ width: `${w}%` }} />
        </div>
      ))}
    </div>
  )
}

function ErrorState({ onRetry }: { onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 px-4 text-center">
      <AlertCircle className="w-6 h-6 text-muted-foreground mb-2" />
      <p className="text-sm text-muted-foreground mb-3">Failed to load files</p>
      <button
        onClick={onRetry}
        className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground border border-border rounded px-2.5 py-1 transition-colors"
      >
        <RotateCcw className="w-3 h-3" />
        Retry
      </button>
    </div>
  )
}

function EmptyState({ filter }: { filter: FilterType }) {
  const messages: Record<FilterType, string> = {
    modified: 'No modified files',
    all: 'No changed files',
    staged: 'No staged files',
    untracked: 'No untracked files',
  }
  return (
    <div className="flex flex-col items-center justify-center py-10 px-4 text-center">
      <p className="text-sm text-muted-foreground">{messages[filter]}</p>
    </div>
  )
}
