import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { CommandBlock } from '@/components/shared/CommandBlock'
import type { MilestoneSummary, FeatureSummary, UatRun } from '@/lib/types'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { api } from '@/api/client'

// ---------------------------------------------------------------------------
// Progress helpers
// ---------------------------------------------------------------------------

function progressFraction(features: FeatureSummary[]): { done: number; total: number } {
  const total = features.filter(f => !f.archived).length
  const done = features.filter(f => !f.archived && f.next_action === 'done').length
  return { done, total }
}

function dotColor(features: FeatureSummary[]): string {
  const active = features.filter(f => !f.archived)
  if (active.length === 0) return 'bg-muted-foreground'
  if (active.every(f => f.next_action === 'done')) return 'bg-green-400'
  if (active.some(f => f.blocked)) return 'bg-amber-400'
  return 'bg-primary'
}

// ---------------------------------------------------------------------------
// ProgressBar
// ---------------------------------------------------------------------------

function ProgressBar({ done, total }: { done: number; total: number }) {
  const pct = total > 0 ? (done / total) * 100 : 0
  return (
    <div className="flex items-center gap-2 shrink-0">
      <div className="w-20 h-1.5 bg-muted rounded-full overflow-hidden">
        <div
          className="h-full bg-primary rounded-full transition-all duration-300"
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs text-muted-foreground tabular-nums">
        {done} / {total}
      </span>
    </div>
  )
}

// ---------------------------------------------------------------------------
// MilestoneDigestRow
// ---------------------------------------------------------------------------

interface MilestoneDigestRowProps {
  milestone: MilestoneSummary
  features: FeatureSummary[]
}

export function MilestoneDigestRow({ milestone, features }: MilestoneDigestRowProps) {
  const [expanded, setExpanded] = useState(false)
  const [latestRun, setLatestRun] = useState<UatRun | null>(null)
  const { isRunning } = useAgentRuns()

  useEffect(() => {
    api.getLatestMilestoneUatRun(milestone.slug)
      .then(run => setLatestRun(run))
      .catch(() => {})
  }, [milestone.slug])

  const { done, total } = progressFraction(features)
  const dot = dotColor(features)

  const nextFeature = features.find(f => !f.archived && f.next_action !== 'done')
  const cmd = nextFeature ? `/sdlc-run ${nextFeature.slug}` : null

  return (
    <div className="bg-card border border-border rounded-xl overflow-hidden">
      {/* Collapsed header */}
      <div className="px-4 py-3">
        <div className="flex items-center gap-3">
          {/* Status dot */}
          <span className={`w-2 h-2 rounded-full shrink-0 ${dot}`} />

          {/* Title */}
          <Link
            to={`/milestones/${milestone.slug}`}
            className="text-sm font-semibold hover:text-primary transition-colors flex-1 min-w-0 truncate"
          >
            {milestone.title}
          </Link>

          {/* Status badge */}
          <StatusBadge status={milestone.status} />

          {/* Hero thumbnail — latest UAT run screenshot, if available */}
          {latestRun?.screenshots?.[0] && (
            <Link to={`/milestones/${milestone.slug}`} className="shrink-0">
              <img
                src={api.uatArtifactUrl(milestone.slug, latestRun.id, latestRun.screenshots[0])}
                alt="Latest UAT screenshot"
                loading="lazy"
                className="h-8 w-14 rounded object-cover border border-border"
              />
            </Link>
          )}

          {/* Progress */}
          <ProgressBar done={done} total={total} />

          {/* Expand chevron */}
          <button
            onClick={() => setExpanded(v => !v)}
            className="p-0.5 text-muted-foreground hover:text-foreground transition-colors shrink-0"
            aria-label={expanded ? 'Collapse features' : 'Expand features'}
          >
            {expanded
              ? <ChevronDown className="w-4 h-4" />
              : <ChevronRight className="w-4 h-4" />
            }
          </button>
        </div>

        {/* Next command row */}
        {cmd && (
          <div className={`mt-2 pl-5 ${nextFeature && isRunning(nextFeature.slug) ? 'opacity-50' : ''}`}>
            <p className="text-xs text-muted-foreground mb-1">
              Next: <span className="font-mono">{nextFeature?.next_action}</span>
              {' · '}
              <span className="font-mono">{nextFeature?.slug}</span>
            </p>
            <CommandBlock cmd={cmd} />
          </div>
        )}
      </div>

      {/* Expanded feature list */}
      {expanded && (
        <div className="border-t border-border/50 divide-y divide-border/30">
          {features.filter(f => !f.archived).map(f => (
            <div key={f.slug} className="flex items-center gap-3 px-4 py-2">
              <Link
                to={`/features/${f.slug}`}
                className="text-xs font-mono text-muted-foreground hover:text-primary transition-colors flex-1 min-w-0 truncate"
              >
                {f.slug}
              </Link>
              <StatusBadge status={f.phase} />
              <span className="text-xs text-muted-foreground shrink-0 font-mono">
                {f.next_action}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
