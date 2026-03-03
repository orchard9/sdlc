import { createPortal } from 'react-dom'
import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { UatRun, UatVerdict } from '@/lib/types'
import { useSSE } from '@/hooks/useSSE'

interface UatHistoryPanelProps {
  milestoneSlug: string
}

const verdictStyles: Record<UatVerdict, { classes: string; label: string }> = {
  pass: { classes: 'bg-emerald-600/80 text-emerald-100', label: 'PASS' },
  pass_with_tasks: { classes: 'bg-amber-600/80 text-amber-100', label: 'PASS + TASKS' },
  failed: { classes: 'bg-red-600/80 text-red-100', label: 'FAILED' },
}

function VerdictBadge({ verdict }: { verdict: UatVerdict }) {
  const { classes, label } = verdictStyles[verdict] ?? { classes: 'bg-neutral-600 text-neutral-200', label: verdict }
  return (
    <span className={cn('inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium', classes)}>
      {label}
    </span>
  )
}

function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

function sortRunsDescending(runs: UatRun[]): UatRun[] {
  return [...runs].sort((a, b) => {
    const ta = new Date(a.completed_at ?? a.started_at).getTime()
    const tb = new Date(b.completed_at ?? b.started_at).getTime()
    return tb - ta
  })
}

// ---------------------------------------------------------------------------
// ScreenshotLightbox — local component, rendered via portal to escape clipping
// ---------------------------------------------------------------------------

interface ScreenshotLightboxProps {
  screenshots: string[]
  milestoneSlug: string
  runId: string
  initialIndex: number
  onClose: () => void
}

function ScreenshotLightbox({ screenshots, milestoneSlug, runId, initialIndex, onClose }: ScreenshotLightboxProps) {
  const [index, setIndex] = useState(initialIndex)

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
      if (e.key === 'ArrowLeft') setIndex(i => Math.max(0, i - 1))
      if (e.key === 'ArrowRight') setIndex(i => Math.min(screenshots.length - 1, i + 1))
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [onClose, screenshots.length])

  return createPortal(
    <div
      className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="relative max-w-5xl max-h-[90vh] flex flex-col items-center"
        onClick={e => e.stopPropagation()}
      >
        <img
          src={api.uatArtifactUrl(milestoneSlug, runId, screenshots[index])}
          alt={`UAT screenshot ${index + 1} of ${screenshots.length}`}
          className="max-h-[80vh] max-w-full object-contain rounded"
        />
        <div className="flex items-center gap-4 mt-3">
          <button
            onClick={() => setIndex(i => Math.max(0, i - 1))}
            disabled={index === 0}
            className="px-3 py-1 rounded bg-white/10 text-white hover:bg-white/20 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            ◀
          </button>
          <span className="text-sm text-white">{index + 1} / {screenshots.length}</span>
          <button
            onClick={() => setIndex(i => Math.min(screenshots.length - 1, i + 1))}
            disabled={index === screenshots.length - 1}
            className="px-3 py-1 rounded bg-white/10 text-white hover:bg-white/20 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            ▶
          </button>
        </div>
        <button
          onClick={onClose}
          className="absolute top-0 right-0 -translate-y-8 translate-x-2 text-white/70 hover:text-white text-lg leading-none"
          aria-label="Close lightbox"
        >
          ✕
        </button>
      </div>
    </div>,
    document.body
  )
}

// ---------------------------------------------------------------------------
// UatHistoryPanel
// ---------------------------------------------------------------------------

export function UatHistoryPanel({ milestoneSlug }: UatHistoryPanelProps) {
  const [runs, setRuns] = useState<UatRun[]>([])
  const [loading, setLoading] = useState(true)
  const [lightbox, setLightbox] = useState<{ runId: string; index: number } | null>(null)

  const load = useCallback(() => {
    api.listMilestoneUatRuns(milestoneSlug)
      .then(data => setRuns(sortRunsDescending(data)))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [milestoneSlug])

  useEffect(() => { load() }, [load])

  useSSE(
    () => {},
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    (event) => { if (event.slug === milestoneSlug) load() },
  )

  if (loading) {
    return (
      <div data-testid="uat-history-panel" className="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="w-4 h-4 animate-spin" />
        Loading UAT history…
      </div>
    )
  }

  if (runs.length === 0) {
    return (
      <div data-testid="uat-history-panel" className="text-xs text-muted-foreground">
        No UAT runs yet.
      </div>
    )
  }

  // Find the run currently open in the lightbox (if any)
  const lightboxRun = lightbox ? runs.find(r => r.id === lightbox.runId) : null

  return (
    <div data-testid="uat-history-panel" className="space-y-2">
      {runs.map(run => (
        <div key={run.id} className="bg-card border border-border rounded-xl p-4">
          <div className="flex flex-wrap items-center gap-3">
            <VerdictBadge verdict={run.verdict} />
            <span className="text-sm text-muted-foreground">
              {formatDate(run.completed_at ?? run.started_at)}
            </span>
            <span className="text-sm">
              {run.tests_passed}/{run.tests_total} passed
            </span>
            {run.tasks_created.length > 0 && (
              <span className="text-xs text-muted-foreground">
                {run.tasks_created.length} task{run.tasks_created.length !== 1 ? 's' : ''} created
              </span>
            )}
          </div>

          {run.screenshots?.length > 0 && (
            <div className="flex gap-2 overflow-x-auto py-1 mt-2">
              {run.screenshots.map((filename, i) => (
                <img
                  key={filename}
                  src={api.uatArtifactUrl(run.milestone_slug, run.id, filename)}
                  alt={`UAT screenshot ${i + 1} of ${run.screenshots.length}`}
                  loading="lazy"
                  className="h-16 w-auto rounded cursor-pointer shrink-0 border border-border hover:border-primary transition-colors"
                  onClick={() => setLightbox({ runId: run.id, index: i })}
                />
              ))}
            </div>
          )}
        </div>
      ))}

      {lightbox && lightboxRun && lightboxRun.screenshots?.length > 0 && (
        <ScreenshotLightbox
          screenshots={lightboxRun.screenshots}
          milestoneSlug={lightboxRun.milestone_slug}
          runId={lightboxRun.id}
          initialIndex={lightbox.index}
          onClose={() => setLightbox(null)}
        />
      )}
    </div>
  )
}
