import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { useProjectState } from '@/hooks/useProjectState'
import { FeatureCard } from '@/components/features/FeatureCard'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton, SkeletonCard } from '@/components/shared/Skeleton'
import { api } from '@/api/client'
import type { ProjectConfig } from '@/lib/types'
import { Copy, Check, AlertTriangle, Clock, ArrowRight, ChevronDown, ChevronRight } from 'lucide-react'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import { MarkdownContent } from '@/components/shared/MarkdownContent'

function CopyButton({ text, copied, onCopy }: { text: string; copied: string | null; onCopy: (t: string) => void }) {
  return (
    <button
      onClick={() => onCopy(text)}
      className="shrink-0 p-1.5 rounded-lg border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
      title="Copy command"
    >
      {copied === text ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
    </button>
  )
}

function VisionSection({ content }: { content: string }) {
  const [open, setOpen] = useState(false)
  const preview = content.replace(/^#+\s*/gm, '').replace(/[*`_~]/g, '').trim()
  return (
    <>
      <div className="mt-3 cursor-pointer group" onClick={() => setOpen(true)}>
        <p className="text-sm text-muted-foreground leading-relaxed line-clamp-2 group-hover:text-foreground/80 transition-colors">
          {preview}
        </p>
        <button className="mt-1 text-xs text-muted-foreground hover:text-foreground transition-colors">
          View vision →
        </button>
      </div>
      <FullscreenModal open={open} onClose={() => setOpen(false)} title="Vision">
        <MarkdownContent content={content} />
      </FullscreenModal>
    </>
  )
}

function CommandBlock({ cmd, copied, onCopy }: { cmd: string; copied: string | null; onCopy: (t: string) => void }) {
  return (
    <div className="flex items-center gap-2">
      <code className="flex-1 text-xs font-mono bg-muted/60 border border-border/50 px-3 py-1.5 rounded-lg text-muted-foreground select-all">
        {cmd}
      </code>
      <CopyButton text={cmd} copied={copied} onCopy={onCopy} />
    </div>
  )
}

export function Dashboard() {
  const { state, error, loading } = useProjectState()
  const [config, setConfig] = useState<ProjectConfig | null>(null)
  const [vision, setVision] = useState<{ content: string; exists: boolean } | null>(null)
  const [copied, setCopied] = useState<string | null>(null)
  const [showArchive, setShowArchive] = useState(false)

  useEffect(() => {
    Promise.all([api.getConfig(), api.getVision()])
      .then(([cfg, vis]) => { setConfig(cfg); setVision(vis) })
      .catch(() => {})
  }, [])

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(text)
      setTimeout(() => setCopied(null), 2000)
    })
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    )
  }

  if (loading || !state) {
    return (
      <div className="max-w-5xl mx-auto space-y-6">
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

  // Primary next command: first active milestone's next feature, or first ungrouped active feature
  let primaryCmd: string | null = null
  let primaryLabel: string | null = null
  let primaryMessage: string | null = null

  for (const milestone of activeMilestones) {
    const nextSlug = milestone.features.find(s => {
      const f = featureBySlug.get(s)
      return f && !f.archived && f.next_action !== 'done'
        && f.next_action !== 'wait_for_approval'
        && f.next_action !== 'unblock_dependency'
    })
    if (nextSlug) {
      const f = featureBySlug.get(nextSlug)
      primaryCmd = `/sdlc-run ${nextSlug}`
      primaryLabel = f?.title ?? nextSlug
      primaryMessage = f?.next_message ?? null
      break
    }
  }

  if (!primaryCmd && activeFeatures.length > 0) {
    const f = activeFeatures[0]
    primaryCmd = `/sdlc-run ${f.slug}`
    primaryLabel = f.title
    primaryMessage = f.next_message
  }

  const doneCount = state.features.filter(f => !f.archived && f.next_action === 'done').length
  const blockedCount = state.blocked.length

  return (
    <div className="max-w-5xl mx-auto">

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

        {/* Vision */}
        {vision?.exists && vision.content && (
          <VisionSection content={vision.content} />
        )}
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

      {/* What's Next */}
      {(hitlFeatures.length > 0 || primaryCmd || state.blocked.length > 0) && (
        <section className="mb-8">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">What's Next</h3>

          {/* HITL / blocked needing human */}
          {hitlFeatures.length > 0 && (
            <div className="bg-amber-950/30 border border-amber-500/20 rounded-xl p-4 mb-3 space-y-2">
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
            <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 mb-3 space-y-2">
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

          {/* Primary next command */}
          {primaryCmd && hitlFeatures.length === 0 && (
            <div className="bg-card border border-border rounded-xl p-4">
              <div className="flex items-center gap-2 mb-2">
                <ArrowRight className="w-4 h-4 text-primary shrink-0" />
                <span className="text-sm font-medium">{primaryLabel}</span>
              </div>
              {primaryMessage && (
                <p className="text-xs text-muted-foreground mb-3 ml-6">{primaryMessage}</p>
              )}
              <CommandBlock cmd={primaryCmd} copied={copied} onCopy={handleCopy} />
            </div>
          )}

          {/* Blocked items */}
          {state.blocked.length > 0 && (
            <div className="mt-3 space-y-1.5">
              {state.blocked.map(b => {
                const f = featureBySlug.get(b.feature)
                return (
                  <div key={b.feature} className="flex items-start gap-2 text-xs text-muted-foreground">
                    <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                    <span>
                      <Link to={`/features/${b.feature}`} className="font-medium hover:text-foreground transition-colors">
                        {f?.title ?? b.feature}
                      </Link>
                      {' '}— {b.reason}
                    </span>
                  </div>
                )
              })}
            </div>
          )}
        </section>
      )}

      {/* All done */}
      {state.features.length > 0 && activeFeatures.length === 0 && hitlFeatures.length === 0 && state.blocked.length === 0 && (
        <div className="mb-8 bg-green-950/20 border border-green-500/20 rounded-xl p-4 text-center">
          <p className="text-sm text-green-400 font-medium">All features complete</p>
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
              <div className="mb-3">
                <CommandBlock cmd={cmd} copied={copied} onCopy={handleCopy} />
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
