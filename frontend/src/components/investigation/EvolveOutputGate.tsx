import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { cn } from '@/lib/utils'
import type { InvestigationDetail } from '@/lib/types'

interface Props {
  investigation: InvestigationDetail
}

export function EvolveOutputGate({ investigation }: Props) {
  const navigate = useNavigate()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const featureSlug = 'evolve-' + investigation.slug.slice(0, 36)

  // Read roadmap.md for context
  const roadmapArtifact = investigation.artifacts.find(a => a.filename === 'roadmap.md')
  const roadmapExcerpt = roadmapArtifact?.content?.slice(0, 300) ?? ''

  // Already-outputted state
  if (investigation.output_type === 'task' && investigation.output_ref) {
    return (
      <div className="px-3 py-3 space-y-2.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
            Output
          </span>
          <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            feature created
          </span>
        </div>
        <p className="text-xs text-muted-foreground/70 leading-snug">
          Evolution feature created from this analysis.
        </p>
        <button
          onClick={() => navigate(`/features/${investigation.output_ref}`)}
          className="text-xs text-primary hover:text-primary/80 transition-colors underline underline-offset-2"
        >
          View feature →
        </button>
      </div>
    )
  }

  const handleCreateFeature = async () => {
    setLoading(true)
    setError(null)
    try {
      await api.createFeature({
        slug: featureSlug,
        title: 'Evolve: ' + investigation.title,
        description: roadmapExcerpt || undefined,
      })
      await api.updateInvestigation(investigation.slug, {
        output_type: 'task',
        output_ref: featureSlug,
      })
      navigate(`/features/${featureSlug}`)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to create feature'
      if (msg.toLowerCase().includes('already') || msg.toLowerCase().includes('conflict') || msg.includes('409')) {
        try {
          await api.updateInvestigation(investigation.slug, {
            output_type: 'task',
            output_ref: featureSlug,
          })
          navigate(`/features/${featureSlug}`)
        } catch {
          setError('Feature already exists. Navigate to it manually.')
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
      <div>
        <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
          Output
        </span>
        <p className="mt-1 text-xs text-foreground/80 leading-snug">
          Evolution roadmap complete. Create a feature to execute the work.
        </p>
      </div>

      {investigation.scope && (
        <p className="text-xs text-muted-foreground/50 font-mono truncate">
          scope: {investigation.scope}
        </p>
      )}

      <div className="flex items-center gap-2">
        <button
          onClick={handleCreateFeature}
          disabled={loading}
          className={cn(
            'px-3 py-1.5 text-xs font-medium rounded-lg transition-colors whitespace-nowrap',
            'bg-primary text-primary-foreground hover:bg-primary/90',
            'disabled:opacity-50 disabled:cursor-not-allowed',
          )}
        >
          {loading ? 'Creating...' : 'Create Evolution Feature'}
        </button>
      </div>

      {error && (
        <p className="text-xs text-destructive leading-snug">{error}</p>
      )}
    </div>
  )
}
