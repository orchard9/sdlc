import { useProjectState } from '@/hooks/useProjectState'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonMilestone } from '@/components/shared/Skeleton'
import { Link } from 'react-router-dom'

export function MilestonesPage() {
  const { state, loading } = useProjectState()

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto">
        <Skeleton width="w-32" className="h-6 mb-6" />
        <div className="space-y-3">
          <SkeletonMilestone />
          <SkeletonMilestone />
          <SkeletonMilestone />
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-5xl mx-auto">
      <h2 className="text-xl font-semibold mb-6">Milestones</h2>
      {state.milestones.length === 0 ? (
        <p className="text-sm text-muted-foreground">No milestones yet.</p>
      ) : (
        <div className="space-y-3">
          {state.milestones.map(m => (
            <div key={m.slug} className="bg-card border border-border rounded-xl p-4">
              <div className="flex items-center gap-2 mb-2">
                <Link to={`/milestones/${m.slug}`} className="text-sm font-semibold hover:text-accent-foreground transition-colors">
                  {m.title}
                </Link>
                <StatusBadge status={m.status} />
              </div>
              <p className="text-xs text-muted-foreground">{m.features.length} features</p>
              {m.features.length > 0 && (
                <div className="flex flex-wrap gap-1.5 mt-2">
                  {m.features.map((fs, idx) => (
                    <Link
                      key={fs}
                      to={`/features/${fs}`}
                      className="text-xs bg-muted px-2 py-0.5 rounded hover:bg-accent transition-colors flex items-center gap-1"
                    >
                      <span className="font-mono text-muted-foreground/60 tabular-nums">{idx + 1}.</span>
                      {fs}
                    </Link>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
