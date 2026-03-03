import { Link } from 'react-router-dom'
import { MilestoneDigestRow } from './MilestoneDigestRow'
import { StatusBadge } from '@/components/shared/StatusBadge'
import type { MilestoneSummary, FeatureSummary } from '@/lib/types'

function CurrentZoneEmpty() {
  return (
    <div className="bg-card border border-border rounded-xl p-4 flex items-center justify-between gap-4">
      <p className="text-sm text-muted-foreground">
        No active work. Start a milestone or add a feature.
      </p>
      <div className="flex items-center gap-2 shrink-0">
        <Link
          to="/milestones"
          className="text-xs text-muted-foreground hover:text-foreground transition-colors px-2 py-1 rounded hover:bg-accent border border-border"
        >
          Milestones
        </Link>
        <Link
          to="/features?new=1"
          className="text-xs text-primary hover:text-primary/80 transition-colors px-2 py-1 rounded hover:bg-accent border border-primary/30"
        >
          + Feature
        </Link>
      </div>
    </div>
  )
}

interface CurrentZoneProps {
  milestones: MilestoneSummary[]
  featureBySlug: Map<string, FeatureSummary>
  ungrouped: FeatureSummary[]
}

export function CurrentZone({ milestones, featureBySlug, ungrouped }: CurrentZoneProps) {
  const hasContent = milestones.length > 0 || ungrouped.length > 0

  return (
    <div className="mb-8 space-y-3">
      {milestones.map(milestone => {
        const features = milestone.features
          .map(s => featureBySlug.get(s))
          .filter((f): f is FeatureSummary => f != null && !f.archived)
        if (features.length === 0) return null
        return (
          <MilestoneDigestRow
            key={milestone.slug}
            milestone={milestone}
            features={features}
          />
        )
      })}

      {ungrouped.length > 0 && (
        <div className="bg-card border border-border rounded-xl overflow-hidden">
          <div className="px-4 py-2 border-b border-border/50">
            <h3 className="text-xs font-semibold text-muted-foreground">Ungrouped</h3>
          </div>
          <div className="divide-y divide-border/30">
            {ungrouped.map(f => (
              <div key={f.slug} className="flex items-center gap-3 px-4 py-2">
                <Link
                  to={`/features/${f.slug}`}
                  className="text-sm font-medium hover:text-primary transition-colors flex-1 min-w-0 truncate"
                >
                  {f.title}
                </Link>
                <StatusBadge status={f.phase} />
                <span className="text-xs text-muted-foreground font-mono shrink-0">
                  {f.next_action}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {!hasContent && <CurrentZoneEmpty />}
    </div>
  )
}
