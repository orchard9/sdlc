import { useCallback, useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { useProjectState } from '@/hooks/useProjectState'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { FeatureCard } from '@/components/features/FeatureCard'
import { ArrowLeft, ArrowUp, ArrowDown, Loader2 } from 'lucide-react'
import type { MilestoneDetail as MilestoneDetailType } from '@/lib/types'

export function MilestoneDetail() {
  const { slug } = useParams<{ slug: string }>()
  const [milestone, setMilestone] = useState<MilestoneDetailType | null>(null)
  const [loading, setLoading] = useState(true)
  const [reordering, setReordering] = useState(false)
  const { state } = useProjectState()

  const load = useCallback(() => {
    if (!slug) return
    api.getMilestone(slug)
      .then(m => setMilestone(m))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [slug])

  useEffect(() => {
    setLoading(true)
    load()
  }, [load])

  useSSE(load)

  if (!slug) return null

  const handleMove = async (featureSlug: string, direction: 'up' | 'down') => {
    if (!milestone || reordering) return
    const idx = milestone.features.indexOf(featureSlug)
    if (idx === -1) return
    const newFeatures = [...milestone.features]
    const swapIdx = direction === 'up' ? idx - 1 : idx + 1
    if (swapIdx < 0 || swapIdx >= newFeatures.length) return
    ;[newFeatures[idx], newFeatures[swapIdx]] = [newFeatures[swapIdx], newFeatures[idx]]

    // Optimistic update
    setMilestone(m => m ? { ...m, features: newFeatures } : m)
    setReordering(true)
    try {
      const updated = await api.reorderMilestoneFeatures(slug, newFeatures)
      setMilestone(updated)
    } catch {
      // Revert on error — milestone.features is intentionally the pre-move snapshot
      // captured in closure at the start of this call, not the current state.
      setMilestone(m => m ? { ...m, features: milestone.features } : m)
    } finally {
      setReordering(false)
    }
  }

  if (loading || !milestone) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  const featureBySlug = new Map((state?.features ?? []).map(f => [f.slug, f]))

  return (
    <div className="max-w-4xl mx-auto">
      <Link
        to="/milestones"
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground mb-4 transition-colors"
      >
        <ArrowLeft className="w-4 h-4" />
        Back
      </Link>

      <div className="flex items-start justify-between gap-4 mb-6">
        <div>
          <h2 className="text-xl font-semibold">{milestone.title}</h2>
          <p className="text-sm text-muted-foreground font-mono">{milestone.slug}</p>
          {milestone.description && (
            <p className="text-sm text-muted-foreground mt-1">{milestone.description}</p>
          )}
          {milestone.vision && (
            <p className="text-sm text-foreground/80 mt-2 max-w-2xl">{milestone.vision}</p>
          )}
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge status={milestone.status} />
          <span className="text-xs text-muted-foreground">
            {milestone.features.length} feature{milestone.features.length !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      <section>
        <h3 className="text-sm font-semibold mb-3">Features</h3>
        {milestone.features.length === 0 ? (
          <p className="text-xs text-muted-foreground">No features in this milestone.</p>
        ) : (
          <div className="space-y-2">
            {milestone.features.map((fs, idx) => {
              const feature = featureBySlug.get(fs)
              const isFirst = idx === 0
              const isLast = idx === milestone.features.length - 1
              return (
                <div key={fs} className="flex items-start gap-2">
                  <div className="flex flex-col items-center gap-0.5 pt-3 shrink-0">
                    <span className="text-xs font-mono font-semibold text-foreground/50 w-5 text-center tabular-nums">
                      {idx + 1}
                    </span>
                    <button
                      onClick={() => handleMove(fs, 'up')}
                      disabled={isFirst || reordering}
                      className="p-0.5 rounded hover:bg-accent disabled:opacity-30 disabled:cursor-default transition-colors"
                      aria-label={`Move ${fs} up`}
                    >
                      <ArrowUp className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => handleMove(fs, 'down')}
                      disabled={isLast || reordering}
                      className="p-0.5 rounded hover:bg-accent disabled:opacity-30 disabled:cursor-default transition-colors"
                      aria-label={`Move ${fs} down`}
                    >
                      <ArrowDown className="w-3.5 h-3.5" />
                    </button>
                  </div>
                  <div className="flex-1 min-w-0">
                    {feature ? (
                      <FeatureCard feature={feature} />
                    ) : (
                      <div className="bg-card border border-border rounded-xl p-4">
                        <span className="text-sm font-medium font-mono text-muted-foreground">{fs}</span>
                      </div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </section>
    </div>
  )
}
