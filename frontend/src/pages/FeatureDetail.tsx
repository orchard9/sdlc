import { useParams, Link } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { useFeature } from '@/hooks/useFeature'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import { ArtifactViewer } from '@/components/features/ArtifactViewer'
import { SkeletonFeatureDetail } from '@/components/shared/Skeleton'
import { api } from '@/api/client'
import { ArrowLeft, Copy, Check } from 'lucide-react'

const ARTIFACT_TYPES = ['spec', 'design', 'tasks', 'qa_plan', 'review', 'audit', 'qa_results']

export function FeatureDetail() {
  const { slug } = useParams<{ slug: string }>()
  const { feature, classification, loading, refresh } = useFeature(slug ?? '')

  const [copied, setCopied] = useState<'run' | 'next' | null>(null)

  const handleCopy = (text: string, which: 'run' | 'next') => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(which)
      setTimeout(() => setCopied(null), 2000)
    })
  }

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
          <p className="text-sm font-medium mb-1">
            Next: {classification.action === 'create_spec' ? 'view spec' : classification.action.replace(/_/g, ' ')}
          </p>
          <p className="text-xs text-muted-foreground mb-3">{classification.message}</p>
          <div className="flex flex-col gap-2 mb-3">
            <div className="flex items-center gap-2">
              <code className="flex-1 text-xs font-mono bg-muted/60 border border-border/50 px-3 py-2 rounded-lg text-muted-foreground select-all">
                /sdlc-run {slug}
              </code>
              <button
                onClick={() => handleCopy(`/sdlc-run ${slug}`, 'run')}
                className="shrink-0 p-2 rounded-lg border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                title="Copy command"
              >
                {copied === 'run' ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
            <div className="flex items-center gap-2">
              <code className="flex-1 text-xs font-mono bg-muted/60 border border-border/50 px-3 py-2 rounded-lg text-muted-foreground select-all">
                /sdlc-next {slug}
              </code>
              <button
                onClick={() => handleCopy(`/sdlc-next ${slug}`, 'next')}
                className="shrink-0 p-2 rounded-lg border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                title="Copy command"
              >
                {copied === 'next' ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>
        </div>
      )}

      {classification && classification.action === 'done' && (
        <div className="flex items-center gap-2 px-3 py-2 bg-green-500/10 border border-green-500/30 rounded-xl mb-6">
          <span className="text-xs font-medium text-green-400">Feature complete — no pending actions</span>
        </div>
      )}

      {/* Artifacts */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold mb-3">Artifacts</h3>
        <div className="space-y-3">
          {ARTIFACT_TYPES.map(type => {
            const artifact = feature.artifacts.find(a => a.artifact_type === type)
            if (!artifact) return null
            return (
              <div key={type} id={`artifact-${type}`}>
                <ArtifactViewer slug={slug} artifact={artifact} onStatusChange={refresh} />
              </div>
            )
          })}
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
