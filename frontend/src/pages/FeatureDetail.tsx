import { useParams, Link } from 'react-router-dom'
import { useFeature } from '@/hooks/useFeature'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import { ArtifactViewer } from '@/components/features/ArtifactViewer'
import { SkeletonFeatureDetail } from '@/components/shared/Skeleton'
import { CopyButton } from '@/components/shared/CopyButton'
import { ArrowLeft, Play, Loader2, AlertTriangle } from 'lucide-react'

const ARTIFACT_TYPES = ['spec', 'design', 'tasks', 'qa_plan', 'review', 'audit', 'qa_results']

export function FeatureDetail() {
  const { slug } = useParams<{ slug: string }>()
  const { feature, classification, error, loading } = useFeature(slug ?? '')
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()

  if (!slug) return null

  if (loading) {
    return <div className="p-6"><SkeletonFeatureDetail /></div>
  }

  if (error) {
    // Extract artifact type from serde error like: artifacts[1].status: unknown variant `waived`...
    const artifactMatch = error.match(/artifacts\[\d+\]\.status[^`]*`([^`]+)`/)
    const corruptStatus = artifactMatch?.[1]

    return (
      <div className="max-w-4xl mx-auto p-6">
        <Link to="/" className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground mb-4 transition-colors">
          <ArrowLeft className="w-4 h-4" />
          Back
        </Link>
        <div className="bg-destructive/10 border border-destructive/30 rounded-xl p-5 space-y-4">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-destructive shrink-0 mt-0.5" />
            <div>
              <p className="text-sm font-semibold text-destructive mb-1">Feature data is corrupt</p>
              <p className="text-xs text-muted-foreground font-mono break-all">{error}</p>
            </div>
          </div>
          <div className="space-y-2">
            <p className="text-xs text-muted-foreground">
              The feature manifest contains an unrecognized value{corruptStatus ? ` ('${corruptStatus}')` : ''}.
              Use the <code className="font-mono bg-muted/60 px-1 rounded">sdlc_repair_artifact</code> MCP tool to reset it:
            </p>
            <div className="flex items-center gap-1">
              <code className="flex-1 text-xs font-mono bg-muted/60 border border-border/50 px-2 py-1.5 rounded text-muted-foreground break-all select-all">
                {`sdlc_repair_artifact({ "slug": "${slug}", "artifact_type": "<type>", "set_status": "missing" })`}
              </code>
              <CopyButton
                text={`sdlc_repair_artifact({ "slug": "${slug}", "artifact_type": "<type>", "set_status": "missing" })`}
                className="shrink-0 p-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
              />
            </div>
            <p className="text-xs text-muted-foreground">
              After repairing, call <code className="font-mono bg-muted/60 px-1 rounded">sdlc_get_directive</code> to re-enter the normal flow.
              Valid statuses: <span className="font-mono">missing, draft, approved, rejected, needs_fix, passed, failed, waived</span>
            </p>
          </div>
        </div>
      </div>
    )
  }

  if (!feature) {
    return <div className="p-6"><SkeletonFeatureDetail /></div>
  }

  const running = isRunning(slug)
  const activeRun = getRunForKey(slug)

  const handleRun = () => {
    startRun({
      key: slug,
      runType: 'feature',
      target: slug,
      label: slug,
      startUrl: `/api/run/${slug}`,
      stopUrl: `/api/run/${slug}/stop`,
    })
  }

  const handleFocus = () => {
    if (activeRun) focusRun(activeRun.id)
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <Link to="/" className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground mb-4 transition-colors">
        <ArrowLeft className="w-4 h-4" />
        Back
      </Link>

      <div className="flex items-start justify-between gap-4 mb-4">
        <div>
          <h2 data-testid="feature-title" className="text-xl font-semibold">{feature.title}</h2>
          <p className="text-sm text-muted-foreground font-mono">{feature.slug}</p>
          {feature.description && (
            <p className="text-sm text-muted-foreground mt-1">{feature.description}</p>
          )}
        </div>
        <StatusBadge status={feature.phase} testId="phase-badge" />
      </div>

      <PhaseProgressBar current={feature.phase} className="mb-6" />

      {/* Next action */}
      {classification && classification.action !== 'done' && (
        <div data-testid="next-action" className="bg-card border border-border rounded-xl p-4 mb-6">
          <div className="flex items-start justify-between gap-3 mb-3">
            <div>
              <p className="text-sm font-medium mb-1">
                Next: {classification.action === 'create_spec' ? 'view spec' : classification.action.replace(/_/g, ' ')}
              </p>
              <p className="text-xs text-muted-foreground">{classification.message}</p>
            </div>
            {running ? (
              <button
                onClick={handleFocus}
                className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-muted text-muted-foreground border border-border text-xs font-medium hover:bg-muted/80 transition-colors whitespace-nowrap"
              >
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Running...
              </button>
            ) : (
              <button
                onClick={handleRun}
                className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors whitespace-nowrap"
              >
                <Play className="w-3.5 h-3.5" />
                Run
              </button>
            )}
          </div>

          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1">
              <code className="text-xs font-mono bg-muted/60 border border-border/50 px-2 py-1 rounded text-muted-foreground select-all">
                /sdlc-run {slug}
              </code>
              <CopyButton text={`/sdlc-run ${slug}`} className="shrink-0 p-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors" />
            </div>
            <div className="flex items-center gap-1">
              <code className="text-xs font-mono bg-muted/60 border border-border/50 px-2 py-1 rounded text-muted-foreground select-all">
                /sdlc-next {slug}
              </code>
              <CopyButton text={`/sdlc-next ${slug}`} className="shrink-0 p-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors" />
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
      <section data-testid="artifact-list" className="mb-6">
        <h3 className="text-sm font-semibold mb-3">Artifacts</h3>
        <div className="space-y-3">
          {ARTIFACT_TYPES.map(type => {
            const artifact = feature.artifacts.find(a => a.artifact_type === type)
            if (!artifact) return null
            return (
              <div key={type} id={`artifact-${type}`}>
                <ArtifactViewer artifact={artifact} />
              </div>
            )
          })}
        </div>
      </section>

      {/* Tasks */}
      <section data-testid="task-list" className="mb-6">
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
