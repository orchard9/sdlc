import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { useProjectState } from '@/hooks/useProjectState'
import { useMilestoneUatRun } from '@/hooks/useMilestoneUatRun'
import { HumanUatModal } from '@/components/shared/HumanUatModal'
import { CheckCircle, Loader2, Play, ArrowRight } from 'lucide-react'
import type { UatRun, UatVerdict } from '@/lib/types'

const verdictStyles: Record<UatVerdict, { classes: string; label: string }> = {
  pass: { classes: 'bg-emerald-600/80 text-emerald-100', label: 'PASS' },
  pass_with_tasks: { classes: 'bg-amber-600/80 text-amber-100', label: 'PASS + TASKS' },
  failed: { classes: 'bg-red-600/80 text-red-100', label: 'FAILED' },
}

function VerdictBadge({ verdict }: { verdict: UatVerdict }) {
  const style = verdictStyles[verdict] ?? { classes: 'bg-neutral-600 text-neutral-200', label: verdict }
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-medium ${style.classes}`}>
      {style.label}
    </span>
  )
}

function formatDate(iso: string | null | undefined): string {
  if (!iso) return ''
  return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

interface ReleasedPanelProps {
  milestoneSlug: string
}

export function ReleasedPanel({ milestoneSlug }: ReleasedPanelProps) {
  const [uatRuns, setUatRuns] = useState<UatRun[]>([])
  const { state } = useProjectState()
  const { running, handleStart, handleFocus, modalOpen, setModalOpen } = useMilestoneUatRun(milestoneSlug)

  const loadRuns = useCallback(() => {
    api.listMilestoneUatRuns(milestoneSlug)
      .then(runs => setUatRuns(runs))
      .catch(() => {})
  }, [milestoneSlug])

  useEffect(() => { loadRuns() }, [loadRuns])

  // Refresh UAT runs on milestone UAT events
  const noop = useCallback(() => {}, [])
  useSSE(
    noop,
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    (event) => { if (event.slug === milestoneSlug) loadRuns() },
  )

  // Compute stats
  const milestone = state?.milestones.find(m => m.slug === milestoneSlug)
  const featureCount = milestone?.features.length ?? 0
  const runCount = uatRuns.length

  // Latest completed run (sorted by completion date descending)
  const latestRun = uatRuns
    .filter(r => r.completed_at)
    .sort((a, b) => new Date(b.completed_at!).getTime() - new Date(a.completed_at!).getTime())[0] ?? null

  // Find next active milestone
  const nextActiveMilestone = state?.milestones.find(
    m => m.status === 'active' && m.slug !== milestoneSlug
  ) ?? null

  return (
    <>
      <div className="bg-green-950/30 border border-green-500/30 rounded-lg p-4 mb-4">
        {/* Victory banner */}
        <div className="flex items-center gap-2.5 mb-3">
          <CheckCircle className="w-5 h-5 text-green-400 shrink-0" />
          <div>
            <div className="text-sm text-green-400 font-medium">Milestone Released</div>
            {milestone && (
              <div className="text-xs text-foreground/60">{milestone.title}</div>
            )}
          </div>
        </div>

        {/* Stats row */}
        <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground tabular-nums mb-3">
          <span>{featureCount} feature{featureCount !== 1 ? 's' : ''}</span>
          <span>{runCount} UAT run{runCount !== 1 ? 's' : ''}</span>
          {latestRun && (
            <>
              <span className="flex items-center gap-1.5">
                Latest: <VerdictBadge verdict={latestRun.verdict} />
              </span>
              <span>{formatDate(latestRun.completed_at)}</span>
            </>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-3 mb-3">
          {running ? (
            <button
              onClick={handleFocus}
              className="shrink-0 inline-flex items-center gap-1 px-2.5 py-1 rounded border border-border bg-muted text-muted-foreground text-[11px] hover:bg-muted/80 transition-colors"
            >
              <Loader2 className="w-3 h-3 animate-spin" />
              Running
            </button>
          ) : (
            <button
              onClick={handleStart}
              className="shrink-0 inline-flex items-center gap-1 px-2.5 py-1 rounded border border-green-500/30 bg-green-500/20 text-green-400 text-[11px] hover:bg-green-500/30 transition-colors"
            >
              <Play className="w-3 h-3" />
              Re-run UAT
            </button>
          )}
          {!running && (
            <button
              onClick={() => setModalOpen(true)}
              className="text-[11px] text-muted-foreground underline hover:text-foreground transition-colors"
            >
              Submit manually
            </button>
          )}
        </div>

        {/* Next milestone link */}
        {nextActiveMilestone && (
          <>
            <div className="border-t border-green-500/15 my-3" />
            <Link
              to={`/milestones/${nextActiveMilestone.slug}`}
              className="inline-flex items-center gap-1.5 text-xs text-green-400 hover:underline transition-colors"
            >
              Next milestone: {nextActiveMilestone.title}
              <ArrowRight className="w-3 h-3" />
            </Link>
          </>
        )}
      </div>

      <HumanUatModal
        open={modalOpen}
        onClose={() => setModalOpen(false)}
        mode="milestone"
        slug={milestoneSlug}
      />
    </>
  )
}
