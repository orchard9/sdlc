import { useState, useEffect, useRef, useCallback, type ReactNode } from 'react'
import { FileText, Monitor, Image, ChevronDown, X, Maximize2, ChevronLeft, ChevronRight } from 'lucide-react'
import { ArtifactContent } from '@/components/shared/ArtifactContent'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import { cn, formatBytes } from '@/lib/utils'
import type { PonderArtifact } from '@/lib/types'

const IMAGE_EXTS = new Set(['.png', '.jpg', '.jpeg', '.gif', '.webp'])

function isImageArtifact(filename: string): boolean {
  const dot = filename.lastIndexOf('.')
  if (dot === -1) return false
  return IMAGE_EXTS.has(filename.slice(dot).toLowerCase())
}

function relativeDate(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const diff = Date.now() - d.getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days === 1) return 'yesterday'
  if (days < 7) return `${days}d ago`
  return d.toLocaleDateString()
}


interface Props {
  artifacts: PonderArtifact[]
  onClose?: () => void
  phasePanel?: ReactNode
  /** Base URL for serving binary media files, e.g. `/api/roadmap/my-slug/media`.
   *  When provided, image artifacts are rendered as `<img>` thumbnails instead
   *  of text content blocks. */
  mediaBaseUrl?: string
}

export function WorkspacePanel({ artifacts, onClose, phasePanel, mediaBaseUrl }: Props) {
  const [activeIndex, setActiveIndex] = useState<number | null>(null)
  const [fullscreen, setFullscreen] = useState(false)
  const touchStartX = useRef<number | null>(null)

  const canPrev = activeIndex !== null && activeIndex > 0
  const canNext = activeIndex !== null && activeIndex < artifacts.length - 1

  const navigate = useCallback((dir: -1 | 1) => {
    setActiveIndex(i => {
      if (i === null) return null
      const next = i + dir
      if (next < 0 || next >= artifacts.length) return i
      return next
    })
  }, [artifacts.length])

  // Arrow key navigation when an artifact is active
  useEffect(() => {
    if (activeIndex === null) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'ArrowLeft') { e.preventDefault(); navigate(-1) }
      else if (e.key === 'ArrowRight') { e.preventDefault(); navigate(1) }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [activeIndex, navigate])

  const activeArtifact = activeIndex !== null ? artifacts[activeIndex] : null

  const swipeHandlers = {
    onTouchStart: (e: React.TouchEvent) => { touchStartX.current = e.touches[0].clientX },
    onTouchEnd: (e: React.TouchEvent) => {
      if (touchStartX.current === null) return
      const diff = touchStartX.current - e.changedTouches[0].clientX
      if (Math.abs(diff) > 48) navigate(diff > 0 ? 1 : -1)
      touchStartX.current = null
    },
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      {/* Header */}
      {onClose && (
        <div className="shrink-0 flex items-center justify-between px-4 py-3 border-b border-border/50">
          <span className="text-sm font-semibold">Workspace</span>
          <button
            onClick={onClose}
            className="p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Phase-aware panel slot — provided by caller */}
      {phasePanel && (
        <div className="shrink-0 border-b border-border/40">
          {phasePanel}
        </div>
      )}

      {/* Scrollable artifact list */}
      <div className="flex-1 overflow-y-auto min-h-0 px-3 py-3">
        {artifacts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center gap-2 py-12">
            <p className="text-sm text-muted-foreground/60">No artifacts yet.</p>
            <p className="text-xs text-muted-foreground/40 max-w-xs leading-relaxed">
              Agents write artifacts here — problem statements, research notes, sketches — as they explore ideas.
            </p>
          </div>
        ) : (
          <div className="space-y-0.5">
            {artifacts.map((artifact, i) => (
              <div
                key={artifact.filename}
                className={cn(
                  'flex items-center w-full rounded-lg transition-colors',
                  activeIndex === i ? 'bg-accent/60' : 'hover:bg-accent/40',
                )}
              >
                <button
                  onClick={() => setActiveIndex(activeIndex === i ? null : i)}
                  className="flex-1 flex items-center gap-3 px-3 py-2.5 text-left min-w-0"
                >
                  {/\.(html|htm)$/i.test(artifact.filename)
                    ? <Monitor className={cn('w-3.5 h-3.5 shrink-0 transition-colors', activeIndex === i ? 'text-primary' : 'text-muted-foreground/50')} />
                    : isImageArtifact(artifact.filename)
                    ? <Image className={cn('w-3.5 h-3.5 shrink-0 transition-colors', activeIndex === i ? 'text-primary' : 'text-muted-foreground/50')} />
                    : <FileText className={cn('w-3.5 h-3.5 shrink-0 transition-colors', activeIndex === i ? 'text-primary' : 'text-muted-foreground/50')} />
                  }
                  <span className="flex-1 text-sm font-mono truncate">{artifact.filename}</span>
                  {/\.(html|htm)$/i.test(artifact.filename) && (
                    <span className="shrink-0 text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded font-mono">
                      preview
                    </span>
                  )}
                  <span className="text-xs text-muted-foreground/40 shrink-0 tabular-nums">
                    {formatBytes(artifact.size_bytes)}
                  </span>
                  <span className="text-xs text-muted-foreground/30 shrink-0 hidden sm:block">
                    {relativeDate(artifact.modified_at)}
                  </span>
                  <ChevronDown className={cn('w-3.5 h-3.5 text-muted-foreground/40 shrink-0 transition-transform', activeIndex === i && 'rotate-180')} />
                </button>
                {artifact.content && (
                  <button
                    onClick={() => { setActiveIndex(i); setFullscreen(true) }}
                    className="p-2 shrink-0 text-muted-foreground/40 hover:text-foreground transition-colors"
                    title="Fullscreen"
                  >
                    <Maximize2 className="w-3.5 h-3.5" />
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Expanded content panel — anchored at bottom, above pagination */}
      {activeArtifact && (
        <div className="shrink-0 border-t border-border/50" {...swipeHandlers}>
          {/* Prev / counter / next */}
          <div className="flex items-center justify-between px-3 py-1.5 border-b border-border/30">
            <button
              onClick={() => navigate(-1)}
              disabled={!canPrev}
              className="p-1 rounded text-muted-foreground/40 hover:text-foreground disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              title="Previous"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
            </button>
            <span className="text-xs text-muted-foreground/40 tabular-nums font-mono">
              {activeIndex! + 1} / {artifacts.length}
            </span>
            <button
              onClick={() => navigate(1)}
              disabled={!canNext}
              className="p-1 rounded text-muted-foreground/40 hover:text-foreground disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              title="Next"
            >
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
          {/* Scrollable content */}
          <div className="overflow-auto max-h-96 px-3 py-2">
            {mediaBaseUrl && isImageArtifact(activeArtifact.filename) ? (
              <a
                href={`${mediaBaseUrl}/${activeArtifact.filename}`}
                target="_blank"
                rel="noopener noreferrer"
                className="block"
              >
                <img
                  src={`${mediaBaseUrl}/${activeArtifact.filename}`}
                  alt={activeArtifact.filename}
                  className="max-h-80 w-auto object-contain rounded border border-border"
                />
              </a>
            ) : activeArtifact.content ? (
              <ArtifactContent filename={activeArtifact.filename} content={activeArtifact.content} />
            ) : (
              <p className="text-xs text-muted-foreground/50 italic">No content</p>
            )}
          </div>
        </div>
      )}

      {/* Pagination bar — one segment per artifact */}
      {artifacts.length > 0 && (
        <div className="shrink-0 flex gap-1 px-4 py-2.5 border-t border-border/30">
          {artifacts.map((_, i) => (
            <button
              key={i}
              onClick={() => setActiveIndex(activeIndex === i ? null : i)}
              className={cn(
                'h-1 flex-1 rounded-full transition-colors',
                i === activeIndex ? 'bg-primary' : 'bg-muted-foreground/20 hover:bg-muted-foreground/40',
              )}
            />
          ))}
        </div>
      )}

      {/* Fullscreen modal with in-modal pagination */}
      {activeArtifact?.content && (
        <FullscreenModal
          open={fullscreen}
          onClose={() => setFullscreen(false)}
          title={activeArtifact.filename}
        >
          {/* Pagination bar */}
          <div className="flex gap-1.5 mb-5">
            {artifacts.map((_, i) => (
              <button
                key={i}
                onClick={() => setActiveIndex(i)}
                className={cn(
                  'h-1.5 flex-1 rounded-full transition-colors',
                  i === activeIndex ? 'bg-primary' : 'bg-muted-foreground/20 hover:bg-muted-foreground/40',
                )}
              />
            ))}
          </div>
          {/* Prev / next row */}
          <div className="flex items-center justify-between mb-5">
            <button
              onClick={() => navigate(-1)}
              disabled={!canPrev}
              className="flex items-center gap-1.5 px-2 py-1 text-xs text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors rounded-lg hover:bg-accent/50"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
              {canPrev ? artifacts[activeIndex! - 1].filename : ''}
            </button>
            <span className="text-xs text-muted-foreground/50 tabular-nums font-mono">
              {activeIndex! + 1} / {artifacts.length}
            </span>
            <button
              onClick={() => navigate(1)}
              disabled={!canNext}
              className="flex items-center gap-1.5 px-2 py-1 text-xs text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors rounded-lg hover:bg-accent/50"
            >
              {canNext ? artifacts[activeIndex! + 1].filename : ''}
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
          <ArtifactContent filename={activeArtifact.filename} content={activeArtifact.content} fullscreen />
        </FullscreenModal>
      )}
    </div>
  )
}
