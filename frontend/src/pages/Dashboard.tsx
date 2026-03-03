import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { useProjectState } from '@/hooks/useProjectState'
import { Skeleton, SkeletonCard } from '@/components/shared/Skeleton'
import { api } from '@/api/client'
import { DashboardEmptyState } from '@/components/dashboard/DashboardEmptyState'
import { AttentionZone } from '@/components/dashboard/AttentionZone'
import { CurrentZone } from '@/components/dashboard/CurrentZone'
import { HorizonZone } from '@/components/dashboard/HorizonZone'
import { ArchiveZone } from '@/components/dashboard/ArchiveZone'
import { AlertTriangle } from 'lucide-react'

export function Dashboard() {
  const { state, error, loading } = useProjectState()
  const [missingVisionOrArch, setMissingVisionOrArch] = useState(false)
  const [hasVision, setHasVision] = useState(false)
  const [hasArch, setHasArch] = useState(false)

  useEffect(() => {
    Promise.all([
      api.getVision().catch(() => null),
      api.getArchitecture().catch(() => null),
    ]).then(([vision, arch]) => {
      const visionExists = !!vision?.exists
      const archExists = !!arch?.exists
      setHasVision(visionExists)
      setHasArch(archExists)
      setMissingVisionOrArch(!visionExists || !archExists)
    })
  }, [])

  if (error) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    )
  }

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto space-y-6 p-4 sm:p-6">
        <div className="space-y-2">
          <Skeleton width="w-48" className="h-7" />
          <Skeleton width="w-64" className="h-4" />
          <Skeleton width="w-80" className="h-3" />
        </div>
        <div className="bg-card border border-border rounded-xl p-4 space-y-3">
          <Skeleton width="w-24" className="h-4" />
          <Skeleton width="w-full" className="h-10" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          <SkeletonCard /><SkeletonCard /><SkeletonCard />
        </div>
      </div>
    )
  }

  const featureBySlug = new Map(state.features.map(f => [f.slug, f]))
  const activeMilestones = state.milestones.filter(m => m.status !== 'released')
  const releasedMilestones = state.milestones.filter(m => m.status === 'released')
  const assignedSlugs = new Set(state.milestones.flatMap(m => m.features))
  const ungrouped = state.features.filter(
    f => !assignedSlugs.has(f.slug) && !f.archived && f.phase !== 'released'
  )

  const hitlFeatures = state.features.filter(
    f => !f.archived && (f.next_action === 'wait_for_approval' || f.next_action === 'unblock_dependency')
  )
  const activeFeatures = state.features.filter(
    f => !f.archived && f.next_action !== 'done' && f.next_action !== 'wait_for_approval' && f.next_action !== 'unblock_dependency'
  )
  const doneCount = state.features.filter(f => !f.archived && f.next_action === 'done').length
  const blockedCount = state.blocked.length

  const featureTitleBySlug = new Map(state.features.map(f => [f.slug, f.title]))

  return (
    <div className="max-w-5xl mx-auto p-4 sm:p-6">

      {/* Vision/Architecture missing banner */}
      {missingVisionOrArch && (
        <div className="bg-amber-950/20 border border-amber-500/30 rounded-xl px-4 py-3 mb-6 flex items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
            <p className="text-sm text-amber-200/90">
              Vision or Architecture not defined — agents use these to make aligned decisions.
            </p>
          </div>
          <Link
            to="/setup"
            className="text-sm text-amber-400 hover:text-amber-300 transition-colors whitespace-nowrap shrink-0"
          >
            Go to Setup →
          </Link>
        </div>
      )}

      {/* Stats bar */}
      <div className="flex items-center gap-4 mb-6 text-xs text-muted-foreground">
        <span>{state.features.filter(f => !f.archived).length} features</span>
        <span>·</span>
        <span>{activeMilestones.length} milestones</span>
        {activeFeatures.length > 0 && (
          <>
            <span>·</span>
            <span className="text-primary">{activeFeatures.length} active</span>
          </>
        )}
        {blockedCount > 0 && (
          <>
            <span>·</span>
            <span className="text-amber-400">{blockedCount} blocked</span>
          </>
        )}
        {doneCount > 0 && (
          <>
            <span>·</span>
            <span className="text-green-400">{doneCount} done</span>
          </>
        )}
      </div>

      {/* Zone 1 — Attention */}
      <AttentionZone
        escalations={state.escalations ?? []}
        hitlFeatures={hitlFeatures}
        activeDirectives={state.active_directives}
        featureTitleBySlug={featureTitleBySlug}
      />

      {/* Zone 2 — Current */}
      <CurrentZone
        milestones={activeMilestones}
        featureBySlug={featureBySlug}
        ungrouped={ungrouped}
      />

      {/* Zone 3 — Horizon */}
      <HorizonZone
        milestones={activeMilestones}
        featureBySlug={featureBySlug}
      />

      {/* Zone 4 — Archive */}
      <ArchiveZone milestones={releasedMilestones} />

      {state.milestones.length === 0 && state.features.length === 0 && (
        <DashboardEmptyState hasVision={hasVision} hasArch={hasArch} />
      )}
    </div>
  )
}
