import { useCallback, useEffect, useMemo, useState } from 'react'
import { DiffView, DiffModeEnum, DiffFile } from '@git-diff-view/react'
import '@git-diff-view/react/styles/diff-view.css'
import './DiffViewer.css'
import { cn } from '@/lib/utils'
import { FileCode, AlertTriangle, Check, HardDrive, WrapText, Columns2, AlignJustify } from 'lucide-react'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface DiffViewerProps {
  /** File path to display and fetch diff for */
  filePath: string
  /** Old ref (defaults to HEAD on server) */
  oldRef?: string
  /** New ref (defaults to working tree on server) */
  newRef?: string
  /** Initial view mode */
  defaultView?: 'unified' | 'split'
}

interface DiffResponse {
  diff_text: string
  additions: number
  deletions: number
  is_binary: boolean
  old_content?: string
  new_content?: string
}

// ---------------------------------------------------------------------------
// useDiff hook
// ---------------------------------------------------------------------------

function useDiff(filePath: string, oldRef?: string, newRef?: string) {
  const [diff, setDiff] = useState<DiffResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchDiff = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const params = new URLSearchParams({ path: filePath })
      if (oldRef) params.set('old', oldRef)
      if (newRef) params.set('new', newRef)
      const res = await fetch(`/api/git/diff?${params}`)
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`)
      const data: DiffResponse = await res.json()
      setDiff(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load diff')
    } finally {
      setLoading(false)
    }
  }, [filePath, oldRef, newRef])

  useEffect(() => {
    fetchDiff()
  }, [fetchDiff])

  return { diff, loading, error, refetch: fetchDiff }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function DiffSkeleton() {
  const widths = ['70%', '55%', '80%', '45%', '65%', '50%', '75%', '60%']
  return (
    <div className="p-3 space-y-1.5">
      {widths.map((w, i) => (
        <div key={i} className="flex gap-3">
          <div className="w-10 h-4 rounded bg-muted animate-pulse" />
          <div className="h-4 rounded bg-muted animate-pulse" style={{ width: w }} />
        </div>
      ))}
    </div>
  )
}

function DiffError({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
      <AlertTriangle className="w-8 h-8 opacity-50" />
      <span className="text-sm">{message}</span>
      <button
        onClick={onRetry}
        className="px-4 py-1.5 text-sm rounded-md border border-border bg-muted hover:bg-accent/50 text-foreground transition-colors"
      >
        Retry
      </button>
    </div>
  )
}

function DiffEmpty() {
  return (
    <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
      <Check className="w-8 h-8 opacity-50" />
      <span className="text-sm">No changes</span>
    </div>
  )
}

function DiffBinary() {
  return (
    <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
      <HardDrive className="w-8 h-8 opacity-50" />
      <span className="text-sm">Binary file — diff not available</span>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

function DiffHeader({
  filePath,
  additions,
  deletions,
  viewMode,
  onViewModeChange,
  wrap,
  onWrapChange,
}: {
  filePath: string
  additions?: number
  deletions?: number
  viewMode: 'unified' | 'split'
  onViewModeChange: (mode: 'unified' | 'split') => void
  wrap: boolean
  onWrapChange: (wrap: boolean) => void
}) {
  return (
    <div className="flex items-center gap-3 px-3 py-2 border-b border-border text-sm flex-wrap">
      <FileCode className="w-4 h-4 text-muted-foreground shrink-0" />
      <span className="font-mono text-xs text-foreground truncate flex-1 min-w-0">
        {filePath}
      </span>

      {additions !== undefined && deletions !== undefined && (
        <div className="flex gap-2 text-xs shrink-0">
          <span className="text-blue-400">+{additions}</span>
          <span className="text-amber-400">-{deletions}</span>
        </div>
      )}

      <button
        onClick={() => onWrapChange(!wrap)}
        className={cn(
          'p-1 rounded transition-colors',
          wrap ? 'text-foreground bg-muted' : 'text-muted-foreground hover:text-foreground',
        )}
        title={wrap ? 'Disable line wrap' : 'Enable line wrap'}
      >
        <WrapText className="w-3.5 h-3.5" />
      </button>

      <div className="flex bg-muted rounded overflow-hidden shrink-0">
        <button
          onClick={() => onViewModeChange('unified')}
          className={cn(
            'flex items-center gap-1 px-2 py-1 text-[11px] transition-colors',
            viewMode === 'unified'
              ? 'bg-border text-foreground'
              : 'text-muted-foreground hover:text-foreground',
          )}
        >
          <AlignJustify className="w-3 h-3" />
          Unified
        </button>
        <button
          onClick={() => onViewModeChange('split')}
          className={cn(
            'flex items-center gap-1 px-2 py-1 text-[11px] transition-colors',
            viewMode === 'split'
              ? 'bg-border text-foreground'
              : 'text-muted-foreground hover:text-foreground',
          )}
        >
          <Columns2 className="w-3 h-3" />
          Split
        </button>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Responsive hook
// ---------------------------------------------------------------------------

function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() =>
    typeof window !== 'undefined' ? window.matchMedia(query).matches : false,
  )

  useEffect(() => {
    const mql = window.matchMedia(query)
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches)
    mql.addEventListener('change', handler)
    setMatches(mql.matches)
    return () => mql.removeEventListener('change', handler)
  }, [query])

  return matches
}

// ---------------------------------------------------------------------------
// DiffViewer (main)
// ---------------------------------------------------------------------------

export function DiffViewer({
  filePath,
  oldRef,
  newRef,
  defaultView = 'unified',
}: DiffViewerProps) {
  const { diff, loading, error, refetch } = useDiff(filePath, oldRef, newRef)
  const [viewMode, setViewMode] = useState<'unified' | 'split'>(defaultView)
  const [wrap, setWrap] = useState(false)
  const isNarrow = useMediaQuery('(max-width: 1023px)')

  // Force unified on narrow viewports
  const effectiveMode = isNarrow ? 'unified' : viewMode
  const diffModeEnum =
    effectiveMode === 'split' ? DiffModeEnum.Split : DiffModeEnum.Unified

  // Build DiffFile instance from raw diff text
  const diffFile = useMemo(() => {
    if (!diff || diff.is_binary || !diff.diff_text) return null
    try {
      // Extract file extension for syntax highlighting
      const ext = filePath.split('.').pop() || ''
      const langMap: Record<string, string> = {
        ts: 'typescript', tsx: 'typescript', js: 'javascript', jsx: 'javascript',
        rs: 'rust', py: 'python', rb: 'ruby', go: 'go', java: 'java',
        css: 'css', html: 'html', json: 'json', yaml: 'yaml', yml: 'yaml',
        md: 'markdown', sh: 'bash', toml: 'toml', sql: 'sql', xml: 'xml',
      }
      const lang = langMap[ext] || ext

      // Parse the diff text to extract hunks
      // The API returns a full unified diff; we need to extract the hunk lines
      const lines = diff.diff_text.split('\n')
      const hunks: string[] = []
      let inHunk = false

      for (const line of lines) {
        if (line.startsWith('@@')) {
          inHunk = true
          hunks.push(line)
        } else if (inHunk) {
          if (line.startsWith('diff ') || line.startsWith('index ') ||
              line.startsWith('--- ') || line.startsWith('+++ ')) {
            inHunk = false
          } else {
            hunks.push(line)
          }
        }
      }

      if (hunks.length === 0) return null

      const instance = DiffFile.createInstance({
        oldFile: {
          fileName: filePath,
          fileLang: lang,
          content: diff.old_content || '',
        },
        newFile: {
          fileName: filePath,
          fileLang: lang,
          content: diff.new_content || '',
        },
        hunks,
      })

      instance.init()
      instance.buildSplitDiffLines()
      instance.buildUnifiedDiffLines()

      return instance
    } catch (err) {
      console.warn('Failed to parse diff:', err)
      return null
    }
  }, [diff, filePath])

  return (
    <div className="diff-viewer rounded-lg border border-border bg-card overflow-hidden">
      <DiffHeader
        filePath={filePath}
        additions={diff?.additions}
        deletions={diff?.deletions}
        viewMode={effectiveMode}
        onViewModeChange={setViewMode}
        wrap={wrap}
        onWrapChange={setWrap}
      />

      {loading && <DiffSkeleton />}

      {!loading && error && <DiffError message={error} onRetry={refetch} />}

      {!loading && !error && diff?.is_binary && <DiffBinary />}

      {!loading && !error && diff && !diff.is_binary && !diff.diff_text && <DiffEmpty />}

      {!loading && !error && diffFile && (
        <DiffView
          diffFile={diffFile}
          diffViewMode={diffModeEnum}
          diffViewWrap={wrap}
          diffViewTheme="dark"
          diffViewHighlight={true}
          diffViewFontSize={13}
        />
      )}
    </div>
  )
}
