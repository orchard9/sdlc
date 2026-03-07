import { useState } from 'react'
import { useProjectState } from '@/hooks/useProjectState'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonMilestone } from '@/components/shared/Skeleton'
import { Link } from 'react-router-dom'
import { MilestonePreparePanel } from '@/components/milestones/MilestonePreparePanel'
import { ChevronDown, ChevronRight } from 'lucide-react'
import type { FeatureSummary, MilestoneStatus } from '@/lib/types'

function MilestoneCard({ m, showPrepare }: {
  m: { slug: string; title: string; vision: string | null; status: string; features: string[] }
  showPrepare?: boolean
}) {
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
      {showPrepare && (
        <div className="mt-3 pt-3 border-t border-border/50">
          <MilestonePreparePanel milestoneSlug={m.slug} milestoneStatus={m.status as MilestoneStatus} />
        </div>
      )}
    </div>
  )
}

export function MilestonesPage() {
  const { state, loading } = useProjectState()
  const [archiveOpen, setArchiveOpen] = useState(false)

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

  const active = state.milestones.filter(m => m.status !== 'released')
  const released = state.milestones.filter(m => m.status === 'released')

  const assignedSlugs = new Set(state.milestones.flatMap(m => m.features))
  const releasedStandalone: FeatureSummary[] = state.features.filter(
    f => !assignedSlugs.has(f.slug) && !f.archived && f.phase === 'released'
  )

  const archiveCount = released.length + releasedStandalone.length

  return (
    <div className="max-w-5xl mx-auto p-4 sm:p-6">
      <h2 className="text-xl font-semibold mb-6">Milestones</h2>
      {active.length === 0 && archiveCount === 0 ? (
        <p className="text-sm text-muted-foreground">No milestones yet.</p>
      ) : (
        <>
          {active.length > 0 && (
            <div className="space-y-3 mb-8">
              {active.map(m => (
                <MilestoneCard key={m.slug} m={m} showPrepare />
              ))}
            </div>
          )}

          {archiveCount > 0 && (
            <div>
              <button
                onClick={() => setArchiveOpen(v => !v)}
                className="flex items-center gap-1.5 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors mb-3"
              >
                {archiveOpen ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
                Archive · {archiveCount} released
              </button>
              {archiveOpen && (
                <>
                  {released.length > 0 && (
                    <div className="space-y-3 mb-4">
                      {released.map(m => <MilestoneCard key={m.slug} m={m} />)}
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
          )}
        </>
      )}
    </div>
  )
}
