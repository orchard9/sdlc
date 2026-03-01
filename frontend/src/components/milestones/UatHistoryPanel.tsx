import { useEffect, useState } from 'react'
import { api } from '@/api/client'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { UatRun, UatVerdict } from '@/lib/types'

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

export function UatHistoryPanel({ milestoneSlug }: UatHistoryPanelProps) {
  const [runs, setRuns] = useState<UatRun[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.listMilestoneUatRuns(milestoneSlug)
      .then(data => setRuns(sortRunsDescending(data)))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [milestoneSlug])

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

  return (
    <div data-testid="uat-history-panel" className="space-y-2">
      {runs.map(run => (
        <div key={run.id} className="bg-card border border-border rounded-xl p-4 flex flex-wrap items-center gap-3">
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
      ))}
    </div>
  )
}
