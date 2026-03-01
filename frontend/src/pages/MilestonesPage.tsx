import { useProjectState } from '@/hooks/useProjectState'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonMilestone } from '@/components/shared/Skeleton'
import { Link } from 'react-router-dom'
import type { FeatureSummary } from '@/lib/types'

function MilestoneCard({ m }: { m: { slug: string; title: string; vision: string | null; status: string; features: string[] } }) {
  return (
    <div className="bg-card border border-border rounded-xl p-4">
      <div className="flex items-center gap-2 mb-2">
        <Link to={`/milestones/${m.slug}`} data-testid="milestone-title" className="text-sm font-semibold hover:text-accent-foreground transition-colors">
          {m.title}
        </Link>
        <StatusBadge status={m.status} testId="milestone-status" />
      </div>
      {m.vision && (
        <p className="text-sm text-foreground/80 mb-1.5">{m.vision}</p>
      )}
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
  )
}

export function MilestonesPage({ filter }: { filter?: 'released' }) {
  const { state, loading } = useProjectState()

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto p-4 sm:p-6">
        <Skeleton width="w-32" className="h-6 mb-6" />
        <div className="space-y-3">
          <SkeletonMilestone />
          <SkeletonMilestone />
          <SkeletonMilestone />
        </div>
      </div>
    )
  }

  const milestones = filter === 'released'
    ? state.milestones.filter(m => m.status === 'released')
    : state.milestones.filter(m => m.status !== 'released')

  const title = filter === 'released' ? 'Archive' : 'Milestones'
  const emptyMsg = filter === 'released' ? 'No released milestones yet.' : 'No milestones yet.'

  const assignedSlugs = new Set(state.milestones.flatMap(m => m.features))
  const releasedStandalone: FeatureSummary[] = filter === 'released'
    ? state.features.filter(f => !assignedSlugs.has(f.slug) && !f.archived && f.phase === 'released')
    : []

  return (
    <div className="max-w-5xl mx-auto p-4 sm:p-6">
      <h2 className="text-xl font-semibold mb-6">{title}</h2>
      {milestones.length === 0 && releasedStandalone.length === 0 ? (
        <p className="text-sm text-muted-foreground">{emptyMsg}</p>
      ) : (
        <>
          {milestones.length > 0 && (
            <div className="space-y-3 mb-8">
              {milestones.map(m => <MilestoneCard key={m.slug} m={m} />)}
            </div>
          )}
          {releasedStandalone.length > 0 && (
            <section>
              <h3 className="text-sm font-semibold text-muted-foreground mb-3">
                Released Features ({releasedStandalone.length})
              </h3>
              <div className="space-y-1.5">
                {releasedStandalone.map(f => (
                  <div key={f.slug} className="flex items-center gap-2 px-3 py-2 bg-muted/30 border border-border/40 rounded-lg">
                    <Link to={`/features/${f.slug}`} className="text-sm font-medium hover:text-primary transition-colors">
                      {f.title}
                    </Link>
                    <StatusBadge status={f.phase} />
                    <span className="text-xs font-mono text-muted-foreground/50">{f.slug}</span>
                  </div>
                ))}
              </div>
            </section>
          )}
        </>
      )}
    </div>
  )
}
