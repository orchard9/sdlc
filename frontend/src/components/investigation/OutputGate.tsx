import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { cn } from '@/lib/utils'
import type { InvestigationDetail } from '@/lib/types'

interface Props {
  investigation: InvestigationDetail
}

function confidenceColor(confidence: number): string {
  if (confidence >= 70) return 'bg-emerald-500'
  if (confidence >= 40) return 'bg-amber-500'
  return 'bg-red-500'
}

function ConfidenceBar({ confidence }: { confidence: number }) {
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-xs">
        <span className="text-muted-foreground/60">confidence</span>
        <span className={cn(
          'tabular-nums font-medium',
          confidence >= 70
            ? 'text-emerald-400'
            : confidence >= 40
              ? 'text-amber-400'
              : 'text-red-400',
        )}>
          {confidence}%
        </span>
      </div>
      <div className="h-1 rounded-full bg-muted/60 overflow-hidden">
        <div
          className={cn('h-full rounded-full transition-all', confidenceColor(confidence))}
          style={{ width: `${Math.min(100, Math.max(0, confidence))}%` }}
        />
      </div>
      {confidence < 70 && (
        <p className="text-xs text-amber-400/80 leading-snug">
          Low confidence — consider investigating further before acting
        </p>
      )}
    </div>
  )
}

export function OutputGate({ investigation }: Props) {
  const navigate = useNavigate()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Derive feature slug from investigation slug
  const featureSlug = 'fix-' + investigation.slug.slice(0, 36)

  // Extract synthesis excerpt from artifacts
  const synthesisArtifact = investigation.artifacts.find(
    a => a.filename === 'synthesis.md',
  )
  const synthesisExcerpt = synthesisArtifact?.content?.slice(0, 300) ?? ''

  // Already-outputted state
  if (investigation.output_type === 'task' && investigation.output_ref) {
    return (
      <div className="px-3 py-3 space-y-2.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
            Output
          </span>
          <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            task created
          </span>
        </div>
        <p className="text-xs text-muted-foreground/70 leading-snug">
          Fix task created from this investigation.
        </p>
        <button
          onClick={() => navigate(`/features/${investigation.output_ref}`)}
          className="text-xs text-primary hover:text-primary/80 transition-colors underline underline-offset-2"
        >
          View task
        </button>
        {investigation.confidence !== null && investigation.confidence !== undefined && (
          <ConfidenceBar confidence={investigation.confidence} />
        )}
      </div>
    )
  }

  const handleCreateTask = async () => {
    setLoading(true)
    setError(null)
    try {
      // Create the feature (fix task)
      await api.createFeature({
        slug: featureSlug,
        title: 'Fix: ' + investigation.title,
        description: synthesisExcerpt || undefined,
      })

      // Record the output on the investigation
      await api.updateInvestigation(investigation.slug, {
        output_type: 'task',
        output_ref: featureSlug,
      })

      // Navigate to the created feature
      navigate(`/features/${featureSlug}`)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to create task'
      if (msg.toLowerCase().includes('already') || msg.toLowerCase().includes('conflict') || msg.includes('409')) {
        // Feature already exists — still record the ref if missing
        try {
          await api.updateInvestigation(investigation.slug, {
            output_type: 'task',
            output_ref: featureSlug,
          })
          navigate(`/features/${featureSlug}`)
        } catch {
          setError('Task already exists. Navigate to it manually.')
        }
      } else {
        setError(msg)
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="px-3 py-3 space-y-3">
      {/* Header */}
      <div>
        <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
          Output
        </span>
        <p className="mt-1 text-xs text-foreground/80 leading-snug">
          Root cause identified. Choose an output:
        </p>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        <button
          onClick={handleCreateTask}
          disabled={loading}
          className={cn(
            'px-3 py-1.5 text-xs font-medium rounded-lg transition-colors',
            'bg-primary text-primary-foreground hover:bg-primary/90',
            'disabled:opacity-50 disabled:cursor-not-allowed',
          )}
        >
          {loading ? 'Creating...' : 'Create Fix Task'}
        </button>

        {/* Guideline — disabled, coming soon */}
        <div className="relative group">
          <button
            disabled
            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-border/50 text-muted-foreground/50 cursor-not-allowed"
          >
            Start Guideline
          </button>
          <div className="absolute bottom-full left-0 mb-1.5 w-56 px-2 py-1.5 rounded-lg bg-popover border border-border/60 shadow-md text-xs text-muted-foreground leading-snug opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
            Coming soon — create a guideline workspace to author this pattern permanently
          </div>
        </div>
      </div>

      {/* Error */}
      {error && (
        <p className="text-xs text-destructive leading-snug">{error}</p>
      )}

      {/* Confidence */}
      {investigation.confidence !== null && investigation.confidence !== undefined && (
        <ConfidenceBar confidence={investigation.confidence} />
      )}

      {/* Output ref if set but wrong type — edge case */}
      {investigation.output_ref && investigation.output_type !== 'task' && (
        <p className="text-xs text-muted-foreground/50 font-mono truncate">
          ref: {investigation.output_ref}
        </p>
      )}
    </div>
  )
}
