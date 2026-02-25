import { useParams, Link } from 'react-router-dom'
import { useEffect } from 'react'
import { useFeature } from '@/hooks/useFeature'
import { useRunStream } from '@/hooks/useRunStream'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import { ArtifactViewer } from '@/components/features/ArtifactViewer'
import { RunOutput } from '@/components/pipeline/RunOutput'
import { SkeletonFeatureDetail } from '@/components/shared/Skeleton'
import { api } from '@/api/client'
import { ArrowLeft, Play, Loader2 } from 'lucide-react'

const ARTIFACT_TYPES = ['spec', 'design', 'tasks', 'qa_plan', 'review', 'audit', 'qa_results']

export function FeatureDetail() {
  const { slug } = useParams<{ slug: string }>()
  const { feature, classification, loading, refresh } = useFeature(slug ?? '')
  const runStream = useRunStream({ onComplete: refresh })

  // Keyboard shortcuts: Enter = approve first draft artifact, Escape = blur/deselect
  useEffect(() => {
    if (!slug || !feature) return

    const handler = (e: KeyboardEvent) => {
      // Skip if user is typing in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

      if (e.key === 'Enter') {
        const draftArtifact = feature.artifacts.find(
          a => a.status === 'draft' || a.status === 'needs_fix',
        )
        if (draftArtifact) {
          e.preventDefault()
          api.approveArtifact(slug, draftArtifact.artifact_type, 'keyboard').then(() => refresh())
        }
      }

      if (e.key === 'Escape') {
        e.preventDefault()
        // Blur any focused element to deselect
        if (document.activeElement instanceof HTMLElement) {
          document.activeElement.blur()
        }
      }
    }

    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [slug, feature, refresh])

  if (!slug) return null

  if (loading || !feature) {
    return <SkeletonFeatureDetail />
  }

  const handleRun = async () => {
    const result = await api.runFeature(slug)
    if (result.run_id) {
      runStream.start(result.run_id)
    }
  }

  return (
    <div className="max-w-4xl mx-auto">
      <Link to="/" className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground mb-4 transition-colors">
        <ArrowLeft className="w-4 h-4" />
        Back
      </Link>

      <div className="flex items-start justify-between gap-4 mb-4">
        <div>
          <h2 className="text-xl font-semibold">{feature.title}</h2>
          <p className="text-sm text-muted-foreground font-mono">{feature.slug}</p>
          {feature.description && (
            <p className="text-sm text-muted-foreground mt-1">{feature.description}</p>
          )}
        </div>
        <StatusBadge status={feature.phase} />
      </div>

      <PhaseProgressBar current={feature.phase} className="mb-6" />

      {/* Next action */}
      {classification && classification.action !== 'done' && (
        <div className="bg-card border border-border rounded-xl p-4 mb-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Next: {classification.action.replace(/_/g, ' ')}</p>
              <p className="text-xs text-muted-foreground mt-0.5">{classification.message}</p>
            </div>
            <button
              onClick={handleRun}
              disabled={runStream.running}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
            >
              {runStream.running ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
              Run
            </button>
          </div>
        </div>
      )}

      {/* Run output */}
      {(runStream.lines.length > 0 || runStream.running) && (
        <RunOutput
          lines={runStream.lines}
          running={runStream.running}
          exitCode={runStream.exitCode}
          className="mb-6"
        />
      )}

      {/* Artifacts */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold mb-3">Artifacts</h3>
        <div className="space-y-3">
          {ARTIFACT_TYPES.map(type => (
            <ArtifactViewer
              key={type}
              slug={slug}
              artifactType={type}
              onStatusChange={refresh}
            />
          ))}
        </div>
      </section>

      {/* Tasks */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold mb-3">Tasks</h3>
        {feature.tasks.length === 0 ? (
          <p className="text-xs text-muted-foreground">No tasks yet</p>
        ) : (
          <div className="space-y-1.5">
            {feature.tasks.map(task => (
              <div key={task.id} className="flex items-center gap-2 px-3 py-2 bg-card border border-border rounded-lg">
                <StatusBadge status={task.status} />
                <span className="text-sm">{task.title}</span>
                <span className="text-xs text-muted-foreground ml-auto font-mono">{task.id}</span>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Comments */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold mb-3">Comments</h3>
        {feature.comments.length === 0 ? (
          <p className="text-xs text-muted-foreground">No comments</p>
        ) : (
          <div className="space-y-2">
            {feature.comments.map(comment => (
              <div key={comment.id} className="px-3 py-2 bg-card border border-border rounded-lg">
                <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
                  <span className="font-mono">{comment.id}</span>
                  {comment.flag && <StatusBadge status={comment.flag} />}
                  {comment.author && <span>{comment.author}</span>}
                </div>
                <p className="text-sm">{comment.body}</p>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
