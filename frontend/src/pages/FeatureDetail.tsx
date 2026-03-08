import { useParams, Link, useNavigate } from 'react-router-dom'
import { useFeature } from '@/hooks/useFeature'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import { ArtifactViewer } from '@/components/features/ArtifactViewer'
import { SkeletonFeatureDetail } from '@/components/shared/Skeleton'
import { CopyButton } from '@/components/shared/CopyButton'
import { HumanUatModal } from '@/components/shared/HumanUatModal'
import { ArrowLeft, Play, Loader2, AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-react'
import { useState, useEffect } from 'react'
import { api } from '@/api/client'
import { BlockedPanel } from '@/components/features/BlockedPanel'
import { nextIterationSlug } from '@/lib/iterateSlug'

const ARTIFACT_TYPES = ['spec', 'design', 'tasks', 'qa_plan', 'review', 'audit', 'qa_results']

export function FeatureDetail() {
  const { slug } = useParams<{ slug: string }>()
  const navigate = useNavigate()
  const { feature, classification, error, loading } = useFeature(slug ?? '')
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()
  const [allSlugs, setAllSlugs] = useState<string[]>([])
  const [humanQaModalOpen, setHumanQaModalOpen] = useState(false)
  const [iterating, setIterating] = useState(false)
  const [ponderSlugs, setPonderSlugs] = useState<string[]>([])

  useEffect(() => {
    api.getFeatures().then(features => setAllSlugs(features.map((f: { slug: string }) => f.slug))).catch(() => {})
    api.getRoadmap(true).then(entries => setPonderSlugs(entries.map((e: { slug: string }) => e.slug))).catch(() => {})
  }, [])

  if (!slug) return null

  if (loading) {
    return <div className="p-6"><SkeletonFeatureDetail /></div>
  }

  if (error) {
    // Detect serde corruption: artifacts[1].status: unknown variant `waived`...
    const artifactMatch = error.match(/artifacts\[\d+\]\.status[^`]*`([^`]+)`/)
    const corruptStatus = artifactMatch?.[1]
    const isCorrupt = artifactMatch !== null

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
              <p className="text-sm font-semibold text-destructive mb-1">
                {isCorrupt ? 'Feature data is corrupt' : 'Failed to load feature'}
              </p>
              <p className="text-xs text-muted-foreground font-mono break-all">{error}</p>
            </div>
          </div>
          {isCorrupt && (
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
          )}
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

  const handleIterate = async () => {
    if (!feature) return
    setIterating(true)
    try {
      const newSlug = nextIterationSlug(feature.slug, ponderSlugs)
      await api.createPonderEntry({
        slug: newSlug,
        title: `Iterate: ${feature.title}`,
        brief: `Follow-up iteration on "${feature.title}". Original feature slug: ${feature.slug}.`,
      })
      navigate(`/ponder/${newSlug}`)
    } catch (err) {
      console.error('Failed to create iteration ponder:', err)
    } finally {
      setIterating(false)
    }
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <nav className="flex items-center gap-1.5 text-sm text-muted-foreground mb-4">
        {feature.milestone ? (
          <>
            <Link to="/milestones" className="hover:text-foreground transition-colors">Milestones</Link>
            <span>/</span>
            <Link to={`/milestones/${feature.milestone.slug}`} className="hover:text-foreground transition-colors truncate max-w-[200px]">
              {feature.milestone.title}
            </Link>
            <span>/</span>
            <span className="text-foreground truncate">{feature.title}</span>
          </>
        ) : (
          <>
            <Link to="/" className="hover:text-foreground transition-colors">Features</Link>
            <span>/</span>
            <span className="text-foreground truncate">{feature.title}</span>
          </>
        )}
      </nav>

      <div className="flex items-start justify-between gap-4 mb-4">
        <div>
          <h2 data-testid="feature-title" className="text-xl font-semibold">{feature.title}</h2>
          <p className="text-sm text-muted-foreground font-mono">{feature.slug}</p>
          {feature.description && (
            <p className="text-sm text-muted-foreground mt-1">{feature.description}</p>
          )}
        </div>
        <div className="flex items-center gap-2">
          {feature.archived && (
            <span className="text-xs px-2 py-0.5 rounded bg-muted text-muted-foreground border border-border">Archived</span>
          )}
          <StatusBadge status={feature.phase} testId="phase-badge" />
        </div>
      </div>

      <PhaseProgressBar current={feature.phase} className="mb-6" />

      {/* Blocked panel */}
      {feature.blocked && feature.blockers && (
        <BlockedPanel
          slug={slug}
          blockers={feature.blockers}
          allSlugs={allSlugs}
          isRunning={running}
          onRunWithDirection={(direction) => {
            startRun({
              key: slug,
              runType: 'feature',
              target: slug,
              label: slug,
              startUrl: `/api/run/${slug}`,
              stopUrl: `/api/run/${slug}/stop`,
              context: direction,
            })
          }}
        />
      )}

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
              <div className="flex items-center gap-2">
                {classification.action === 'run_qa' && (
                  <button
                    onClick={() => setHumanQaModalOpen(true)}
                    className="text-xs text-muted-foreground underline hover:text-foreground transition-colors whitespace-nowrap"
                  >
                    Submit manually
                  </button>
                )}
                <button
                  onClick={handleRun}
                  className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors whitespace-nowrap"
                >
                  <Play className="w-3.5 h-3.5" />
                  Run
                </button>
              </div>
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

      {classification && classification.action === 'done' && (() => {
        const releasedEntry = [...feature.phase_history].reverse().find(p => p.phase === 'released')
        const releasedAt = releasedEntry?.entered ? new Date(releasedEntry.entered) : null
        const createdAt = new Date(feature.created_at)
        const journeyDays = releasedAt ? Math.max(1, Math.round((releasedAt.getTime() - createdAt.getTime()) / (1000 * 60 * 60 * 24))) : null
        return (
          <div className="bg-green-500/10 border border-green-500/30 rounded-xl p-4 mb-6">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-green-400" />
                <span className="text-sm font-medium text-green-400">Released</span>
              </div>
              <button
                onClick={handleIterate}
                disabled={iterating}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-green-500/20 text-green-400 text-xs font-medium hover:bg-green-500/30 transition-colors disabled:opacity-50"
              >
                {iterating ? <Loader2 className="w-3 h-3 animate-spin" /> : <RefreshCw className="w-3 h-3" />}
                Iterate
              </button>
            </div>
            <div className="flex items-center gap-4 text-xs text-muted-foreground">
              {releasedAt && <span>Released {releasedAt.toLocaleDateString()}</span>}
              {journeyDays !== null && <span>{journeyDays}d journey</span>}
              {feature.milestone && (
                <Link to={`/milestones/${feature.milestone.slug}`} className="hover:text-foreground transition-colors">
                  {feature.milestone.title}
                </Link>
              )}
            </div>
          </div>
        )
      })()}

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

      <HumanUatModal
        open={humanQaModalOpen}
        onClose={() => setHumanQaModalOpen(false)}
        mode="feature"
        slug={slug}
      />
    </div>
  )
}
