import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton } from '@/components/shared/Skeleton'
import { InvestigationDialoguePanel } from '@/components/investigation/InvestigationDialoguePanel'
import { LensCards } from '@/components/investigation/LensCards'
import { EvolveOutputGate } from '@/components/investigation/EvolveOutputGate'
import { ArtifactContent } from '@/components/shared/ArtifactContent'
import { WorkspacePanel } from '@/components/ponder/WorkspacePanel'
import { WorkspaceShell } from '@/components/layout/WorkspaceShell'
import { CreateWorkspaceModal } from '@/components/shared/CreateWorkspaceModal'
import {
  Plus, ArrowLeft, Loader2, Files,
  Search, BarChart3, GitFork, Map, Zap,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import type {
  InvestigationSummary,
  InvestigationStatus,
  InvestigationDetail,
} from '@/lib/types'

// ---------------------------------------------------------------------------
// Status tabs
// ---------------------------------------------------------------------------

const STATUS_TABS: { label: string; value: InvestigationStatus | 'all' }[] = [
  { label: 'All', value: 'all' },
  { label: 'Active', value: 'in_progress' },
  { label: 'Complete', value: 'complete' },
  { label: 'Parked', value: 'parked' },
]

// ---------------------------------------------------------------------------
// Phase badge
// ---------------------------------------------------------------------------

function PhaseBadge({ phase }: { phase: string }) {
  if (!phase || phase === 'done') return null
  return (
    <span className="shrink-0 text-xs font-mono px-1.5 py-0.5 rounded bg-muted/60 text-muted-foreground/60 border border-border/30">
      {phase}
    </span>
  )
}

// ---------------------------------------------------------------------------
// Entry row
// ---------------------------------------------------------------------------

function EntryRow({
  entry,
  selected,
  onSelect,
}: {
  entry: InvestigationSummary
  selected: boolean
  onSelect: () => void
}) {
  return (
    <button
      onClick={onSelect}
      className={cn(
        'w-full text-left px-3 py-2.5 rounded-lg transition-colors',
        selected
          ? 'bg-accent text-accent-foreground'
          : 'hover:bg-accent/40 text-foreground',
      )}
    >
      <div className="flex items-center gap-2 mb-0.5">
        <span className="text-sm font-medium truncate flex-1">{entry.title}</span>
        <PhaseBadge phase={entry.phase} />
        <StatusBadge status={entry.status} />
      </div>
      <div className="flex items-center gap-2.5 text-xs text-muted-foreground">
        {entry.sessions > 0 ? (
          <span className="text-muted-foreground/60">
            {entry.sessions} {entry.sessions === 1 ? 'session' : 'sessions'}
          </span>
        ) : (
          <span className="text-muted-foreground/40 italic">no sessions yet</span>
        )}
      </div>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Entry detail pane
// ---------------------------------------------------------------------------

function EntryDetailPane({
  slug,
  onRefresh,
  onBack,
}: {
  slug: string
  onRefresh: () => void
  onBack: () => void
}) {
  const [entry, setEntry] = useState<InvestigationDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [mobileWorkspaceOpen, setMobileWorkspaceOpen] = useState(false)

  const load = useCallback(() => {
    api.getInvestigation(slug)
      .then(data => { setEntry(data); setError(null) })
      .catch(() => setError('Session not found'))
      .finally(() => setLoading(false))
  }, [slug])

  useEffect(() => {
    setLoading(true)
    setEntry(null)
    load()
  }, [load])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error || !entry) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-sm text-muted-foreground">{error ?? 'Session not found'}</p>
      </div>
    )
  }

  const artifactCount = entry.artifacts.length

  // Build the phase-aware panel for evolve investigations
  let phasePanel: ReactNode = null
  if (entry.phase === 'analyze') {
    phasePanel = <LensCards lensScores={entry.lens_scores} />
  } else if (entry.phase === 'paths' || entry.phase === 'roadmap') {
    const filename = entry.phase === 'paths' ? 'paths.md' : 'roadmap.md'
    const artifact = entry.artifacts.find(a => a.filename === filename)
    if (artifact?.content) {
      phasePanel = (
        <div className="overflow-auto max-h-48 px-3 py-2">
          <ArtifactContent filename={filename} content={artifact.content} />
        </div>
      )
    }
  } else if (entry.phase === 'output') {
    phasePanel = <EvolveOutputGate investigation={entry} />
  }

  return (
    <div className="h-full flex flex-col min-h-0 relative overflow-hidden">
      <div className="shrink-0 flex items-center gap-2 px-4 pt-4 pb-3 border-b border-border/50">
        <button
          onClick={onBack}
          className="md:hidden shrink-0 -ml-1 p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
          aria-label="Back"
        >
          <ArrowLeft className="w-4 h-4" />
        </button>
        <div className="flex-1 min-w-0">
          <h2 className="text-base font-semibold leading-snug truncate">{entry.title}</h2>
          {entry.scope && (
            <p className="text-xs text-muted-foreground/50 font-mono truncate">{entry.scope}</p>
          )}
        </div>
        <StatusBadge status={entry.status} />
        <div className="relative shrink-0 md:hidden">
          <button
            onClick={() => setMobileWorkspaceOpen(o => !o)}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              mobileWorkspaceOpen
                ? 'bg-accent text-accent-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
            )}
            aria-label="Toggle workspace"
            title="Workspace"
          >
            <Files className="w-4 h-4" />
          </button>
          {artifactCount > 0 && (
            <span className="pointer-events-none absolute -top-1 -right-1 w-3.5 h-3.5 flex items-center justify-center rounded-full bg-primary text-primary-foreground text-[9px] font-semibold">
              {artifactCount}
            </span>
          )}
        </div>
      </div>

      <div className="flex-1 flex min-h-0">
        <div className="flex-1 min-w-0 min-h-0">
          <InvestigationDialoguePanel entry={entry} onRefresh={() => { load(); onRefresh() }} />
        </div>
        <div className="hidden md:flex w-64 shrink-0 border-l border-border flex-col min-h-0">
          <WorkspacePanel
            artifacts={entry.artifacts}
            phasePanel={phasePanel}
          />
        </div>
      </div>

      {mobileWorkspaceOpen && (
        <div
          className="md:hidden absolute inset-0 bg-black/30 z-40"
          onClick={() => setMobileWorkspaceOpen(false)}
        />
      )}
      <div
        className={cn(
          'md:hidden absolute inset-x-0 bottom-0 z-50 flex flex-col bg-card border-t border-border rounded-t-2xl shadow-2xl transition-transform duration-300',
          mobileWorkspaceOpen ? 'translate-y-0' : 'translate-y-full',
        )}
        style={{ height: '60%' }}
      >
        <div className="shrink-0 flex justify-center py-2">
          <div className="w-10 h-1 rounded-full bg-border/60" />
        </div>
        <WorkspacePanel
          artifacts={entry.artifacts}
          onClose={() => setMobileWorkspaceOpen(false)}
          phasePanel={phasePanel}
        />
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function EvolvePage() {
  const { slug } = useParams<{ slug: string }>()
  const navigate = useNavigate()
  const [entries, setEntries] = useState<InvestigationSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [listError, setListError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<InvestigationStatus | 'all'>('all')
  const [showModal, setShowModal] = useState(false)

  const load = useCallback(() => {
    api.getInvestigations('evolve')
      .then(data => { setEntries(data); setListError(null) })
      .catch(() => setListError('Failed to load evolve sessions'))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { load() }, [load])

  const handleUpdate = useCallback(() => { load() }, [load])
  useSSE(handleUpdate)

  const filtered = activeTab === 'all'
    ? entries
    : entries.filter(e => e.status === activeTab)

  const showMobileDetail = !!slug

  const listPane = (
    <>
      <div className="px-3 pt-4 pb-2 flex items-center justify-between">
        <h2 className="text-base font-semibold">Evolve</h2>
        <button
          onClick={() => setShowModal(true)}
          className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
          title="New Evolution Session"
        >
          <Plus className="w-4 h-4" />
        </button>
      </div>

      <CreateWorkspaceModal
        open={showModal}
        onClose={() => setShowModal(false)}
        onCreated={(newSlug) => {
          setShowModal(false)
          load()
          navigate(`/evolve/${newSlug}`)
        }}
        title="New Evolve Session"
        fields={{
          showScope: true,
          scopePlaceholder: 'scope — e.g. crates/sdlc-server/src/ (optional)',
          showContext: true,
          contextPlaceholder: "What's the pain? What triggered this? What should be better?",
          requireContext: true,
        }}
        onSubmit={async ({ slug, title, scope, context }) => {
          await api.createInvestigation({ slug, title, kind: 'evolve', context })
          if (scope) await api.updateInvestigation(slug, { scope }).catch(() => {})
        }}
      />

      <div className="px-2 pb-2 space-y-0.5">
        {STATUS_TABS.map(tab => {
          const count = tab.value === 'all'
            ? entries.length
            : entries.filter(e => e.status === tab.value).length
          return (
            <button
              key={tab.value}
              onClick={() => setActiveTab(tab.value)}
              className={cn(
                'w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
                activeTab === tab.value
                  ? 'bg-accent text-accent-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
              )}
            >
              <span>{tab.label}</span>
              <span className={cn(
                'tabular-nums',
                activeTab === tab.value ? 'text-accent-foreground/70' : 'text-muted-foreground/50',
              )}>
                {count}
              </span>
            </button>
          )
        })}
      </div>
      <div className="border-b border-border mx-3 mb-1" />

      <div className="flex-1 overflow-y-auto px-2 pb-2 space-y-0.5">
        {loading ? (
          <div className="space-y-2 px-1 pt-2">
            <Skeleton width="w-full" className="h-12" />
            <Skeleton width="w-full" className="h-12" />
            <Skeleton width="w-full" className="h-12" />
          </div>
        ) : listError ? (
          <p className="text-xs text-destructive px-3 py-4">{listError}</p>
        ) : filtered.length === 0 ? (
          <div className="text-center py-8 px-3">
            <p className="text-xs text-muted-foreground">
              {activeTab === 'all'
                ? 'No evolution sessions yet.'
                : `No ${activeTab.replace('_', ' ')} sessions.`}
            </p>
            {activeTab === 'all' && (
              <button
                onClick={() => setShowModal(true)}
                className="mt-2 inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors whitespace-nowrap"
              >
                <Plus className="w-3 h-3" />
                New Evolution
              </button>
            )}
          </div>
        ) : (
          filtered.map(entry => (
            <EntryRow
              key={entry.slug}
              entry={entry}
              selected={entry.slug === slug}
              onSelect={() => navigate(`/evolve/${entry.slug}`)}
            />
          ))
        )}
      </div>
    </>
  )

  const detailPane = slug ? (
    <EntryDetailPane key={slug} slug={slug} onRefresh={load} onBack={() => navigate('/evolve')} />
  ) : (
    <div className="h-full overflow-y-auto">
      <div className="max-w-xl mx-auto px-6 py-10 space-y-8">
        {/* Hero */}
        <div className="space-y-2">
          <h2 className="text-lg font-semibold tracking-tight">What is Evolve?</h2>
          <p className="text-sm text-muted-foreground leading-relaxed">
            Evolve is a structured workspace for improving what already exists. Point it at a
            system, codebase area, or architecture concern and it runs an agent-driven analysis
            through five phases — from survey to action plan. The output is a concrete roadmap
            you can commit into milestones and features.
          </p>
        </div>

        {/* Phase flow */}
        <div className="space-y-3">
          <h3 className="text-sm font-semibold">How does it work?</h3>
          <div className="grid gap-2">
            {[
              { icon: Search, phase: 'Survey', desc: 'Scan the target area — structure, entry points, docs state, TODOs and FIXMEs.' },
              { icon: BarChart3, phase: 'Analyze', desc: 'Score maturity across five lenses: pit of success, coupling, growth readiness, self-documenting, and failure modes.' },
              { icon: GitFork, phase: 'Paths', desc: 'Propose 2-4 evolution paths with effort/impact tradeoffs.' },
              { icon: Map, phase: 'Roadmap', desc: 'Build a phased roadmap: proper solution, enabling changes, extended vision.' },
              { icon: Zap, phase: 'Output', desc: 'Produce the action plan — chosen path, rationale, concrete next steps.' },
            ].map(({ icon: Icon, phase, desc }) => (
              <div key={phase} className="flex items-start gap-3 p-3 rounded-lg border border-border/50 bg-card/50">
                <Icon className="w-4 h-4 mt-0.5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <span className="text-sm font-medium">{phase}</span>
                  <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* CTA */}
        <div className="pt-2">
          <button
            onClick={() => setShowModal(true)}
            className="inline-flex items-center gap-1.5 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            Start an Evolution
          </button>
          <p className="text-xs text-muted-foreground/60 mt-2">
            Describe the pain point or area you want to improve and an agent takes it from there.
          </p>
        </div>
      </div>
    </div>
  )

  return (
    <WorkspaceShell
      showDetail={showMobileDetail}
      listPane={listPane}
      detailPane={detailPane}
    />
  )
}
