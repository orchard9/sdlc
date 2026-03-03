import { Link } from 'react-router-dom'
import { AlertTriangle, Clock, Key, HelpCircle, Target, FlaskConical, Check, Zap } from 'lucide-react'
import { useState } from 'react'
import { WhatChangedBanner } from '@/components/layout/WhatChangedBanner'
import { PreparePanel } from '@/components/features/PreparePanel'
import { api } from '@/api/client'
import type { EscalationSummary, FeatureSummary, ActiveDirective } from '@/lib/types'

// ---------------------------------------------------------------------------
// Escalation helpers (extracted from Dashboard)
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

// ---------------------------------------------------------------------------
// AttentionZone
// ---------------------------------------------------------------------------

interface AttentionZoneProps {
  escalations: EscalationSummary[]
  hitlFeatures: FeatureSummary[]
  activeDirectives: ActiveDirective[]
  featureTitleBySlug: Map<string, string>
}

export function AttentionZone({
  escalations,
  hitlFeatures,
  activeDirectives,
  featureTitleBySlug,
}: AttentionZoneProps) {
  const hasContent =
    escalations.length > 0 ||
    hitlFeatures.length > 0 ||
    activeDirectives.length > 0

  return (
    <div className="mb-6 space-y-4">
      {/* What Changed banner — always rendered; internally returns null when nothing to show */}
      <WhatChangedBanner />

      {/* Wave Plan — always rendered; internally shows/hides based on prepare state */}
      <PreparePanel />

      {/* Escalations */}
      {escalations.length > 0 && (
        <div className="bg-amber-950/20 border border-amber-500/30 rounded-xl p-4">
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-amber-400" />
            <h3 className="text-sm font-semibold">Needs Your Attention</h3>
            <span className="text-xs text-muted-foreground bg-amber-500/10 px-1.5 py-0.5 rounded-md">
              {escalations.length} open
            </span>
          </div>
          <div className="space-y-3 divide-y divide-border/50">
            {escalations.map(e => (
              <div key={e.id} className="pt-3 first:pt-0">
                <EscalationCard item={e} onResolved={() => { /* SSE drives refresh */ }} />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* HITL / blocked features */}
      {hitlFeatures.length > 0 && (
        <div className="bg-amber-950/30 border border-amber-500/20 rounded-xl p-4 space-y-2">
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

      {/* Active directives */}
      {activeDirectives.length > 0 && (
        <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 space-y-2">
          {activeDirectives.map(d => (
            <div key={d.feature} className="flex items-start gap-2.5">
              <Clock className="w-4 h-4 text-primary shrink-0 mt-0.5" />
              <div className="min-w-0">
                <Link to={`/features/${d.feature}`} className="text-sm font-medium hover:text-primary transition-colors">
                  {featureTitleBySlug.get(d.feature) ?? d.feature}
                </Link>
                <p className="text-xs text-muted-foreground mt-0.5">
                  {d.action} · started {new Date(d.started_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}

      {!hasContent && null}
    </div>
  )
}
