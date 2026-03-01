import { useEffect, useRef, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useProjectState } from '@/hooks/useProjectState'
import { FeatureCard } from '@/components/features/FeatureCard'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonCard } from '@/components/shared/Skeleton'
import { CommandBlock } from '@/components/shared/CommandBlock'
import { api } from '@/api/client'
import type { AgentDefinition, EscalationSummary, ProjectConfig } from '@/lib/types'
import { PreparePanel } from '@/components/features/PreparePanel'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { AlertTriangle, Clock, ChevronDown, ChevronRight, Key, HelpCircle, Target, FlaskConical, Zap, Check } from 'lucide-react'

// ---------------------------------------------------------------------------
// Escalation helpers
// ---------------------------------------------------------------------------

function EscalationIcon({ kind }: { kind: EscalationSummary['kind'] }) {
  switch (kind) {
    case 'secret_request': return <Key className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
    case 'question':       return <HelpCircle className="w-4 h-4 text-blue-400 shrink-0 mt-0.5" />
    case 'vision':         return <Target className="w-4 h-4 text-purple-400 shrink-0 mt-0.5" />
    case 'manual_test':    return <FlaskConical className="w-4 h-4 text-green-400 shrink-0 mt-0.5" />
  }
}

interface EscalationCardProps {
  item: EscalationSummary
  onResolved: () => void
}

function EscalationCard({ item, onResolved }: EscalationCardProps) {
  const [resolving, setResolving] = useState(false)
  const [note, setNote] = useState('')
  const [error, setError] = useState<string | null>(null)

  const submit = async () => {
    if (!note.trim()) return
    setError(null)
    try {
      await api.resolveEscalation(item.id, note.trim())
      onResolved()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to resolve')
    }
  }

  return (
    <div className="flex items-start gap-2.5">
      <EscalationIcon kind={item.kind} />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-xs font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {item.kind}
          </span>
          <span className="text-sm font-medium">{item.title}</span>
        </div>
        <p className={`text-xs text-muted-foreground mt-0.5 ${resolving ? '' : 'line-clamp-2'}`}>{item.context}</p>
        <div className="flex items-center gap-2 mt-1 flex-wrap">
          {item.source_feature && (
            <Link
              to={`/features/${item.source_feature}`}
              className="text-xs text-muted-foreground hover:text-primary transition-colors font-mono"
            >
              → {item.source_feature}
            </Link>
          )}
          {item.kind === 'secret_request' && (
            <Link
              to="/secrets"
              className="text-xs text-amber-400 hover:text-amber-300 transition-colors"
            >
              Go to Secrets →
            </Link>
          )}
          {!resolving && (
            <button
              onClick={() => setResolving(true)}
              className="text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              Resolve
            </button>
          )}
        </div>
        {resolving && (
          <div className="mt-2 space-y-1.5">
            <textarea
              value={note}
              onChange={e => setNote(e.target.value)}
              placeholder={item.kind === 'secret_request'
                ? 'Describe what you added and where (e.g. "Added STRIPE_KEY to production env")…'
                : item.kind === 'manual_test'
                ? 'Describe what you tested and what passed/failed…'
                : 'Your answer or resolution…'}
              rows={3}
              className="w-full text-xs px-2 py-1.5 bg-background border border-border rounded focus:outline-none focus:ring-1 focus:ring-ring resize-none"
              onKeyDown={e => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submit() }}
              // eslint-disable-next-line jsx-a11y/no-autofocus
              autoFocus
            />
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">⌘↵ to submit</span>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => { setResolving(false); setNote('') }}
                  className="text-xs text-muted-foreground hover:text-foreground transition-colors px-2 py-1 rounded hover:bg-accent"
                >
                  Cancel
                </button>
                <button
                  onClick={submit}
                  disabled={!note.trim()}
                  className="text-xs px-2 py-1 rounded bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40 flex items-center gap-1 whitespace-nowrap"
                >
                  <Check className="w-3 h-3" />
                  Resolve
                </button>
              </div>
            </div>
          </div>
        )}
        {error && <p className="text-xs text-destructive mt-1">{error}</p>}
      </div>
    </div>
  )
}

export function Dashboard() {
  const { state, error, loading } = useProjectState()
  const { isRunning } = useAgentRuns()
  const [config, setConfig] = useState<ProjectConfig | null>(null)
  const [showArchive, setShowArchive] = useState(false)
  const navigate = useNavigate()
  const hasCheckedSetup = useRef(false)

  useEffect(() => {
    if (hasCheckedSetup.current) return
    hasCheckedSetup.current = true

    Promise.all([
      api.getConfig().catch(() => null),
      api.getVision().catch(() => null),
      api.getArchitecture().catch(() => null),
      api.getProjectAgents().catch((): AgentDefinition[] => []),
    ]).then(([cfg, vision, arch, agents]) => {
      if (cfg) setConfig(cfg)
      const noProject = !cfg?.project.description || (!vision?.exists && !arch?.exists)
      const noTeam = agents.length === 0
      if (noProject || noTeam) navigate('/setup')
    })
  }, [navigate])

  if (error) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    )
  }

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto space-y-6 p-6">
        <div className="space-y-2">
          <Skeleton width="w-48" className="h-7" />
          <Skeleton width="w-64" className="h-4" />
          <Skeleton width="w-80" className="h-3" />
        </div>
        <div className="bg-card border border-border rounded-xl p-4 space-y-3">
          <Skeleton width="w-24" className="h-4" />
          <Skeleton width="w-full" className="h-10" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          <SkeletonCard /><SkeletonCard /><SkeletonCard />
        </div>
      </div>
    )
  }

  const featureBySlug = new Map(state.features.map(f => [f.slug, f]))
  const activeMilestones = state.milestones.filter(m => m.status !== 'released')
  const releasedMilestones = state.milestones.filter(m => m.status === 'released')
  const assignedSlugs = new Set(state.milestones.flatMap(m => m.features))
  const ungrouped = state.features.filter(f => !assignedSlugs.has(f.slug) && !f.archived)

  // --- What's Next logic ---
  const hitlFeatures = state.features.filter(
    f => !f.archived && (f.next_action === 'wait_for_approval' || f.next_action === 'unblock_dependency')
  )
  const activeFeatures = state.features.filter(
    f => !f.archived && f.next_action !== 'done' && f.next_action !== 'wait_for_approval' && f.next_action !== 'unblock_dependency'
  )

  const doneCount = state.features.filter(f => !f.archived && f.next_action === 'done').length
  const blockedCount = state.blocked.length

  return (
    <div className="max-w-5xl mx-auto p-6">

      {/* Project Overview */}
      <div className="mb-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold">{state.project}</h2>
            {config?.project.description && (
              <p className="text-sm text-muted-foreground mt-0.5">{config.project.description}</p>
            )}
          </div>
          {config && (
            <span className="text-xs text-muted-foreground bg-muted/60 border border-border/50 px-2 py-0.5 rounded font-mono shrink-0">
              v{config.version}
            </span>
          )}
        </div>

      </div>

      {/* Stats bar */}
      <div className="flex items-center gap-4 mb-6 text-xs text-muted-foreground">
        <span>{state.features.filter(f => !f.archived).length} features</span>
        <span>·</span>
        <span>{activeMilestones.length} milestones</span>
        {activeFeatures.length > 0 && (
          <>
            <span>·</span>
            <span className="text-primary">{activeFeatures.length} active</span>
          </>
        )}
        {blockedCount > 0 && (
          <>
            <span>·</span>
            <span className="text-amber-400">{blockedCount} blocked</span>
          </>
        )}
        {doneCount > 0 && (
          <>
            <span>·</span>
            <span className="text-green-400">{doneCount} done</span>
          </>
        )}
      </div>

      {/* Needs Your Attention — escalations from agents */}
      {state.escalations?.length > 0 && (
        <div className="bg-amber-950/20 border border-amber-500/30 rounded-xl p-4 mb-6">
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-amber-400" />
            <h3 className="text-sm font-semibold">Needs Your Attention</h3>
            <span className="text-xs text-muted-foreground bg-amber-500/10 px-1.5 py-0.5 rounded-md">
              {state.escalations.length} open
            </span>
          </div>
          <div className="space-y-3 divide-y divide-border/50">
            {state.escalations.map(e => (
              <div key={e.id} className="pt-3 first:pt-0">
                <EscalationCard
                  item={e}
                  onResolved={() => {
                    // ProjectState will refresh via SSE; no manual reload needed.
                  }}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Wave Plan */}
      <PreparePanel />

      {/* HITL / blocked needing human */}
      {hitlFeatures.length > 0 && (
        <div className="bg-amber-950/30 border border-amber-500/20 rounded-xl p-4 mb-6 space-y-2">
          {hitlFeatures.map(f => (
            <div key={f.slug} className="flex items-start gap-2.5">
              <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
              <div className="min-w-0">
                <Link to={`/features/${f.slug}`} className="text-sm font-medium hover:text-primary transition-colors">
                  {f.title}
                </Link>
                <p className="text-xs text-muted-foreground mt-0.5">{f.next_message}</p>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Active directives (in-flight) */}
      {state.active_directives.length > 0 && (
        <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6 space-y-2">
          {state.active_directives.map(d => {
            const f = featureBySlug.get(d.feature)
            return (
              <div key={d.feature} className="flex items-start gap-2.5">
                <Clock className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                <div className="min-w-0">
                  <Link to={`/features/${d.feature}`} className="text-sm font-medium hover:text-primary transition-colors">
                    {f?.title ?? d.feature}
                  </Link>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    {d.action} · started {new Date(d.started_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </p>
                </div>
              </div>
            )
          })}
        </div>
      )}

      {/* Milestones */}
      {activeMilestones.map(milestone => {
        const features = milestone.features
          .map(s => featureBySlug.get(s))
          .filter((f): f is NonNullable<typeof f> => f != null && !f.archived)
        if (features.length === 0) return null

        const isComplete = milestone.status === 'released'
        const nextFeature = isComplete ? null : milestone.features.find(s => {
          const f = featureBySlug.get(s)
          return f && !f.archived && f.next_action !== 'done'
        })
        const cmd = isComplete
          ? `/sdlc-milestone-verify ${milestone.slug}`
          : nextFeature
            ? `/sdlc-run ${nextFeature}`
            : null

        return (
          <section key={milestone.slug} className="mb-8">
            <div className="flex items-center gap-2 mb-1.5">
              <Link to={`/milestones/${milestone.slug}`} className="text-sm font-semibold hover:text-primary transition-colors">
                {milestone.title}
              </Link>
              <StatusBadge status={milestone.status} />
              <span className="text-xs font-mono text-muted-foreground/60">{milestone.slug}</span>
              <span className="text-xs text-muted-foreground ml-auto">{features.length} features</span>
            </div>
            {cmd && (
              <div className={`mb-3 ${nextFeature && isRunning(nextFeature) ? 'opacity-50' : ''}`}>
                <CommandBlock cmd={cmd} />
              </div>
            )}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {features.map((f, idx) => (
                <FeatureCard key={f.slug} feature={f} position={idx + 1} />
              ))}
            </div>
          </section>
        )
      })}

      {ungrouped.length > 0 && (
        <section className="mb-8">
          <h3 className="text-sm font-semibold text-muted-foreground mb-3">Ungrouped</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {ungrouped.map(f => <FeatureCard key={f.slug} feature={f} />)}
          </div>
        </section>
      )}

      {/* Released archive */}
      {releasedMilestones.length > 0 && (
        <section className="mb-8">
          <button
            onClick={() => setShowArchive(v => !v)}
            className="flex items-center gap-2 w-full text-left px-3 py-2 rounded-lg border border-border/50 bg-muted/20 hover:bg-muted/40 transition-colors mb-2"
          >
            {showArchive ? <ChevronDown className="w-4 h-4 text-muted-foreground shrink-0" /> : <ChevronRight className="w-4 h-4 text-muted-foreground shrink-0" />}
            <span className="text-sm font-medium">Archive</span>
            <span className="text-xs text-muted-foreground">({releasedMilestones.length} released)</span>
          </button>
          {showArchive && (
            <div className="space-y-1.5">
              {releasedMilestones.map(m => (
                <div key={m.slug} className="flex items-center gap-2 px-3 py-2 bg-muted/30 border border-border/40 rounded-lg">
                  <Link to={`/milestones/${m.slug}`} className="text-sm font-medium hover:text-primary transition-colors">
                    {m.title}
                  </Link>
                  <StatusBadge status={m.status} />
                  <span className="text-xs font-mono text-muted-foreground/50">{m.slug}</span>
                  <span className="text-xs text-muted-foreground ml-auto">{m.features.length} features</span>
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      {state.features.length === 0 && (
        <div className="text-center py-16">
          <p className="text-muted-foreground text-sm">No features yet.</p>
          <p className="text-xs text-muted-foreground mt-1">
            Use <code className="text-primary">sdlc feature create</code> to get started.
          </p>
        </div>
      )}
    </div>
  )
}
