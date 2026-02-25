import { useProjectState } from '@/hooks/useProjectState'
import { FeatureCard } from '@/components/features/FeatureCard'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonCard } from '@/components/shared/Skeleton'
import { RefreshCw } from 'lucide-react'

export function Dashboard() {
  const { state, error, loading, refresh } = useProjectState()

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-destructive text-sm">{error}</p>
          <button
            onClick={refresh}
            className="mt-3 text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto">
        <div className="flex items-center justify-between mb-6">
          <div className="space-y-2">
            <Skeleton width="w-40" className="h-6" />
            <Skeleton width="w-48" className="h-3" />
          </div>
        </div>
        <div className="mb-8">
          <div className="flex items-center gap-2 mb-3">
            <Skeleton width="w-32" />
            <Skeleton width="w-14" className="h-5 rounded-md" />
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <SkeletonCard />
            <SkeletonCard />
            <SkeletonCard />
          </div>
        </div>
        <div className="mb-8">
          <div className="flex items-center gap-2 mb-3">
            <Skeleton width="w-24" />
            <Skeleton width="w-14" className="h-5 rounded-md" />
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <SkeletonCard />
            <SkeletonCard />
          </div>
        </div>
      </div>
    )
  }

  // Group features by milestone. Features not in any milestone go to "Ungrouped".
  const milestoneFeatureMap = new Map<string, string[]>()
  for (const m of state.milestones) {
    milestoneFeatureMap.set(m.slug, m.features)
  }
  const assignedSlugs = new Set(state.milestones.flatMap(m => m.features))
  const ungrouped = state.features.filter(f => !assignedSlugs.has(f.slug) && !f.archived)
  const featureBySlug = new Map(state.features.map(f => [f.slug, f]))

  return (
    <div className="max-w-5xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold">{state.project}</h2>
          <p className="text-sm text-muted-foreground mt-0.5">
            {state.features.length} features &middot; {state.milestones.length} milestones
          </p>
        </div>
        <button
          onClick={refresh}
          className="p-2 rounded-lg hover:bg-accent transition-colors"
          title="Refresh"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {state.milestones.map(milestone => {
        const features = milestone.features
          .map(s => featureBySlug.get(s))
          .filter((f): f is NonNullable<typeof f> => f != null && !f.archived)
        if (features.length === 0) return null

        return (
          <section key={milestone.slug} className="mb-8">
            <div className="flex items-center gap-2 mb-3">
              <h3 className="text-sm font-semibold">{milestone.title}</h3>
              <StatusBadge status={milestone.status} />
              <span className="text-xs text-muted-foreground ml-auto">
                {features.length} features
              </span>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {features.map(f => <FeatureCard key={f.slug} feature={f} />)}
            </div>
          </section>
        )
      })}

      {ungrouped.length > 0 && (
        <section className="mb-8">
          <h3 className="text-sm font-semibold text-muted-foreground mb-3">Ungrouped Features</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {ungrouped.map(f => <FeatureCard key={f.slug} feature={f} />)}
          </div>
        </section>
      )}

      {state.features.length === 0 && (
        <div className="text-center py-16">
          <p className="text-muted-foreground text-sm">No features yet.</p>
          <p className="text-xs text-muted-foreground mt-1">
            Run the Setup Wizard or use <code className="text-primary">sdlc feature create</code>
          </p>
        </div>
      )}
    </div>
  )
}
