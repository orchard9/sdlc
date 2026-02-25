import { useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { api } from '@/api/client'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { RunOutput } from '@/components/pipeline/RunOutput'
import { useRunStream } from '@/hooks/useRunStream'
import { ArrowLeft, ArrowUp, ArrowDown, Loader2, Play } from 'lucide-react'
import type { MilestoneDetail as MilestoneDetailType, MilestoneFeatureReview } from '@/lib/types'

export function MilestoneDetail() {
  const { slug } = useParams<{ slug: string }>()
  const [milestone, setMilestone] = useState<MilestoneDetailType | null>(null)
  const [reviews, setReviews] = useState<MilestoneFeatureReview[]>([])
  const [loading, setLoading] = useState(true)
  const [reordering, setReordering] = useState(false)
  const runStream = useRunStream()

  useEffect(() => {
    if (!slug) return
    setLoading(true)
    Promise.all([api.getMilestone(slug), api.reviewMilestone(slug)])
      .then(([m, r]) => {
        setMilestone(m)
        setReviews(r.features)
      })
      .finally(() => setLoading(false))
  }, [slug])

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

  const handleRunMilestone = async () => {
    const { run_id } = await api.runMilestone(slug, 'auto')
    runStream.start(run_id)
  }

  if (loading || !milestone) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  const reviewBySlug = new Map(reviews.map(r => [r.feature, r]))

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
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge status={milestone.status} />
          <span className="text-xs text-muted-foreground">
            {milestone.features.length} feature{milestone.features.length !== 1 ? 's' : ''}
          </span>
          <button
            onClick={handleRunMilestone}
            disabled={runStream.running}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            {runStream.running ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
            {runStream.running ? 'Running...' : 'Run Milestone'}
          </button>
        </div>
      </div>

      {(runStream.lines.length > 0 || runStream.running) && (
        <RunOutput
          lines={runStream.lines}
          running={runStream.running}
          exitCode={runStream.exitCode}
          className="mb-6"
        />
      )}

      <section>
        <h3 className="text-sm font-semibold mb-3">Features</h3>
        {milestone.features.length === 0 ? (
          <p className="text-xs text-muted-foreground">No features in this milestone.</p>
        ) : (
          <div className="space-y-2">
            {milestone.features.map((fs, idx) => {
              const review = reviewBySlug.get(fs)
              const isFirst = idx === 0
              const isLast = idx === milestone.features.length - 1
              return (
                <div key={fs} className="flex items-stretch gap-2">
                  <div className="flex flex-col items-center gap-0.5 pt-1">
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
                  <Link
                    to={`/features/${fs}`}
                    className="flex-1 block bg-card border border-border rounded-xl p-4 hover:border-accent transition-colors"
                  >
                    <div className="flex items-center justify-between gap-2 mb-1">
                      <span className="text-sm font-medium">{fs}</span>
                      {review && <StatusBadge status={review.phase} />}
                    </div>
                    {review && (
                      <div className="text-xs text-muted-foreground">
                        <span className="font-medium">
                          Next: {review.action.replace(/_/g, ' ')}
                        </span>
                        <span className="mx-1.5">&middot;</span>
                        <span>{review.message}</span>
                      </div>
                    )}
                  </Link>
                </div>
              )
            })}
          </div>
        )}
      </section>
    </div>
  )
}
