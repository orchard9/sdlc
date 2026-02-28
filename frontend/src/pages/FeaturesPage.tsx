import { useState } from 'react'
import { useProjectState } from '@/hooks/useProjectState'
import { FeatureCard } from '@/components/features/FeatureCard'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { Phase } from '@/lib/types'

export function FeaturesPage() {
  const { state, loading } = useProjectState()
  const [activePhase, setActivePhase] = useState<Phase | null>(null)

  if (loading || !state) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  const active = state.features.filter(f => !f.archived)

  // Derive the ordered set of phases that actually have features
  const presentPhases = Array.from(
    new Set(active.map(f => f.phase))
  )

  const filtered = activePhase
    ? active.filter(f => f.phase === activePhase)
    : active

  return (
    <div className="max-w-5xl mx-auto p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold">Features</h2>
        <span className="text-xs text-muted-foreground">{filtered.length} of {active.length}</span>
      </div>

      {presentPhases.length > 1 && (
        <div className="flex flex-wrap gap-1.5 mb-5">
          <button
            onClick={() => setActivePhase(null)}
            className={cn(
              'px-2.5 py-0.5 text-xs rounded-md font-medium transition-colors',
              activePhase === null
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:text-foreground'
            )}
          >
            All
          </button>
          {presentPhases.map(phase => (
            <button
              key={phase}
              onClick={() => setActivePhase(prev => prev === phase ? null : phase)}
              className={cn(
                'px-2.5 py-0.5 text-xs rounded-md font-medium transition-colors',
                activePhase === phase
                  ? 'bg-accent text-accent-foreground'
                  : 'bg-muted text-muted-foreground hover:text-foreground'
              )}
            >
              {phase}
            </button>
          ))}
        </div>
      )}

      {filtered.length === 0 ? (
        <p className="text-sm text-muted-foreground">No features yet.</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {filtered.map(f => <FeatureCard key={f.slug} feature={f} />)}
        </div>
      )}
    </div>
  )
}
