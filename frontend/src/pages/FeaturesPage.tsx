import { useProjectState } from '@/hooks/useProjectState'
import { FeatureCard } from '@/components/features/FeatureCard'
import { Loader2 } from 'lucide-react'

export function FeaturesPage() {
  const { state, loading } = useProjectState()

  if (loading || !state) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  const active = state.features.filter(f => !f.archived)

  return (
    <div className="max-w-5xl mx-auto">
      <h2 className="text-xl font-semibold mb-6">Features</h2>
      {active.length === 0 ? (
        <p className="text-sm text-muted-foreground">No features yet.</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {active.map(f => <FeatureCard key={f.slug} feature={f} />)}
        </div>
      )}
    </div>
  )
}
