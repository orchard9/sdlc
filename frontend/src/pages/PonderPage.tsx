import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton } from '@/components/shared/Skeleton'
import { DialoguePanel } from '@/components/ponder/DialoguePanel'
import { WorkspacePanel } from '@/components/ponder/WorkspacePanel'
import {
  Plus, X, ArrowLeft, Lightbulb, Loader2, Users, Files, GitMerge, Sparkles, SlidersHorizontal,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import type { AdvisoryHistory, AdvisorySseEvent, Finding, FindingStatus, MaturityStage, PonderDetail, PonderStatus, PonderSummary } from '@/lib/types'

// ---------------------------------------------------------------------------
// Status tabs
// ---------------------------------------------------------------------------

const STATUS_TABS: { label: string; value: PonderStatus | 'all' }[] = [
  { label: 'All', value: 'all' },
  { label: 'Exploring', value: 'exploring' },
  { label: 'Converging', value: 'converging' },
  { label: 'Committed', value: 'committed' },
  { label: 'Parked', value: 'parked' },
]

function titleToSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
    .slice(0, 40)
}

// ---------------------------------------------------------------------------
// Left pane: entry list item
// ---------------------------------------------------------------------------

function EntryRow({
  entry,
  selected,
  onSelect,
}: {
  entry: PonderSummary
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
        <span className="text-sm font-medium truncate">{entry.title}</span>
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
        {entry.team_size > 0 && (
          <span className="flex items-center gap-0.5">
            <Users className="w-3 h-3" />
            {entry.team_size}
          </span>
        )}
      </div>
      {entry.tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-1">
          {entry.tags.map(t => (
            <span key={t} className="text-xs text-muted-foreground/70">#{t}</span>
          ))}
        </div>
      )}
    </button>
  )
}

// ---------------------------------------------------------------------------
// Left pane: new idea form (inline)
// ---------------------------------------------------------------------------

function NewIdeaForm({
  onCreated,
  onCancel,
  initialTitle,
  initialSlug,
  initialBrief,
}: {
  onCreated: (slug: string) => void
  onCancel: () => void
  initialTitle?: string
  initialSlug?: string
  initialBrief?: string
}) {
  const [slug, setSlug] = useState(initialSlug ?? '')
  const [title, setTitle] = useState(initialTitle ?? '')
  const [brief, setBrief] = useState(initialBrief ?? '')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!slug.trim() || !title.trim()) return
    setSubmitting(true)
    setError(null)
    try {
      await api.createPonderEntry({
        slug: slug.trim(),
        title: title.trim(),
        brief: brief.trim() || undefined,
      })
      const seed = brief.trim() ? `${title.trim()}\n\n${brief.trim()}` : title.trim()
      api.startPonderChat(slug.trim(), seed).catch(() => {})
      onCreated(slug.trim())
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create')
    } finally {
      setSubmitting(false)
    }
  }

  const handleTitleChange = (value: string) => {
    setTitle(value)
    if (!slug || slug === titleToSlug(title)) {
      setSlug(titleToSlug(value))
    }
  }

  return (
    <form onSubmit={handleSubmit} className="p-3 space-y-2 border-b border-border">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">New Idea</span>
        <button type="button" onClick={onCancel} className="p-0.5 text-muted-foreground hover:text-foreground transition-colors">
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
      <input
        type="text"
        value={title}
        onChange={e => handleTitleChange(e.target.value)}
        placeholder="What are you thinking about?"
        className="w-full px-2.5 py-1.5 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground"
        // eslint-disable-next-line jsx-a11y/no-autofocus
        autoFocus
      />
      <input
        type="text"
        value={slug}
        onChange={e => setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-'))}
        placeholder="slug"
        className="w-full px-2.5 py-1 text-xs font-mono bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground"
      />
      <textarea
        value={brief}
        onChange={e => setBrief(e.target.value)}
        placeholder="Brief description (optional)"
        rows={2}
        className="w-full px-2.5 py-1.5 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground resize-none"
      />
      {error && <p className="text-xs text-destructive">{error}</p>}
      <div className="flex justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          className="px-2.5 py-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={!slug.trim() || !title.trim() || submitting}
          className="px-2.5 py-1 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {submitting ? 'Creating...' : 'Create'}
        </button>
      </div>
    </form>
  )
}

// ---------------------------------------------------------------------------
// Advisory panel
// ---------------------------------------------------------------------------

const STAGE_ORDER: MaturityStage[] = ['health', 'consistency', 'refactor', 'structure', 'roadmap', 'advanced']

const STATUS_NEXT: Record<FindingStatus, FindingStatus> = {
  open: 'acknowledged',
  acknowledged: 'resolved',
  resolved: 'dismissed',
  dismissed: 'open',
}

const STATUS_COLORS: Record<FindingStatus, string> = {
  open: 'border-orange-500/30 text-orange-400 hover:bg-orange-500/10',
  acknowledged: 'border-blue-500/30 text-blue-400 hover:bg-blue-500/10',
  resolved: 'border-green-500/30 text-green-400 hover:bg-green-500/10',
  dismissed: 'border-muted/30 text-muted-foreground hover:bg-muted/10',
}

function AdvisoryPanel({
  onCreated,
  onClose,
}: {
  onCreated: (firstSlug: string) => void
  onClose: () => void
}) {
  const [history, setHistory] = useState<AdvisoryHistory | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [runStarting, setRunStarting] = useState(false)
  const [checked, setChecked] = useState<Set<string>>(new Set())
  const [creating, setCreating] = useState(false)
  const { isRunning } = useAgentRuns()
  const advisoryRunning = isRunning('advisory')

  const load = useCallback(() => {
    api.getAdvisory()
      .then(data => { setHistory(data); setError(null) })
      .catch(() => setError('Could not load advisory history.'))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { load() }, [load])

  // Listen for advisory SSE events to reload when run completes
  useSSE(
    () => {},
    undefined,
    undefined,
    undefined,
    undefined,
    (event: AdvisorySseEvent) => {
      if (event.type === 'advisory_run_completed') load()
    },
  )

  const handleRunAnalysis = async () => {
    setRunStarting(true)
    try {
      await api.startAdvisoryRun()
    } catch {
      // already running — that's fine, spinner will activate via AgentRunContext
    } finally {
      setRunStarting(false)
    }
  }

  const handleStatusCycle = async (finding: Finding) => {
    const next = STATUS_NEXT[finding.status]
    try {
      await api.updateFinding(finding.id, next)
      load()
    } catch {
      // ignore
    }
  }

  const toggleChecked = (id: string) => {
    setChecked(prev => {
      const next = new Set(prev)
      next.has(id) ? next.delete(id) : next.add(id)
      return next
    })
  }

  const handleCreateSelected = async () => {
    const findings = activeFindings.filter(f => checked.has(f.id))
    if (findings.length === 0) return
    setCreating(true)
    let firstSlug: string | null = null
    for (const finding of findings) {
      const slug = titleToSlug(finding.title)
      try {
        await api.createPonderEntry({ slug, title: finding.title, brief: finding.description })
        const seed = finding.description ? `${finding.title}\n\n${finding.description}` : finding.title
        api.startPonderChat(slug, seed).catch(() => {})
        if (!firstSlug) firstSlug = slug
      } catch {
        // slug conflict — entry may already exist, skip
      }
    }
    setCreating(false)
    if (firstSlug) onCreated(firstSlug)
  }

  const activeFindings = history?.findings.filter(
    f => f.status === 'open' || f.status === 'acknowledged',
  ) ?? []

  const findingsByStage = STAGE_ORDER.reduce<Partial<Record<MaturityStage, Finding[]>>>((acc, stage) => {
    const staged = activeFindings.filter(f => f.stage === stage)
    if (staged.length > 0) acc[stage] = staged
    return acc
  }, {})

  const lastRun = history?.runs[history.runs.length - 1]
  const isEmpty = !history || (history.runs.length === 0 && history.findings.length === 0)
  const checkedCount = checked.size

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />
      <div className="relative bg-card border border-border rounded-xl shadow-xl w-full max-w-lg mx-4 overflow-hidden max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="shrink-0 flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-primary" />
            <span className="text-sm font-semibold">Advisory</span>
            {lastRun && (
              <span className="text-xs text-muted-foreground">
                · {new Date(lastRun.run_at).toLocaleDateString()}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleRunAnalysis}
              disabled={advisoryRunning || runStarting}
              className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
            >
              {advisoryRunning || runStarting
                ? <Loader2 className="w-3 h-3 animate-spin" />
                : <Sparkles className="w-3 h-3" />}
              {advisoryRunning ? 'Running…' : 'Run analysis'}
            </button>
            <button
              onClick={onClose}
              className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="flex items-center gap-2 text-sm text-muted-foreground py-6 justify-center">
              <Loader2 className="w-4 h-4 animate-spin" />
              Loading advisory history…
            </div>
          ) : error ? (
            <p className="text-sm text-destructive text-center py-4">{error}</p>
          ) : isEmpty ? (
            <div className="text-center py-8">
              <p className="text-sm text-muted-foreground">No analysis yet.</p>
              <p className="text-xs text-muted-foreground/60 mt-1">
                Run advisory to get suggestions based on the maturity ladder.
              </p>
            </div>
          ) : activeFindings.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-4">
              No open findings. Run analysis to check for new ones.
            </p>
          ) : (
            <div className="space-y-4">
              {(STAGE_ORDER.filter(s => findingsByStage[s]) as MaturityStage[]).map(stage => (
                <div key={stage}>
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 capitalize">
                    {stage}
                  </h3>
                  <div className="space-y-1.5">
                    {(findingsByStage[stage] ?? []).map(finding => (
                      <label
                        key={finding.id}
                        className={cn(
                          'flex items-start gap-2.5 p-2.5 rounded-lg border transition-colors cursor-pointer',
                          checked.has(finding.id)
                            ? 'border-primary/50 bg-primary/5'
                            : 'border-border hover:border-primary/30',
                        )}
                      >
                        <input
                          type="checkbox"
                          checked={checked.has(finding.id)}
                          onChange={() => toggleChecked(finding.id)}
                          className="mt-0.5 shrink-0 accent-primary"
                        />
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium line-clamp-1">{finding.title}</p>
                          <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed line-clamp-2">
                            {finding.description}
                          </p>
                        </div>
                        <button
                          onClick={e => { e.preventDefault(); handleStatusCycle(finding) }}
                          className={cn(
                            'shrink-0 text-xs px-1.5 py-0.5 rounded-full border transition-colors whitespace-nowrap',
                            STATUS_COLORS[finding.status],
                          )}
                          title="Click to cycle status"
                        >
                          {finding.status}
                        </button>
                      </label>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer — shown when at least one item is checked */}
        {checkedCount > 0 && (
          <div className="shrink-0 flex items-center justify-between px-4 py-3 border-t border-border bg-card">
            <span className="text-xs text-muted-foreground">
              {checkedCount} {checkedCount === 1 ? 'idea' : 'ideas'} selected
            </span>
            <button
              onClick={handleCreateSelected}
              disabled={creating}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
            >
              {creating
                ? <><Loader2 className="w-3 h-3 animate-spin" /> Creating…</>
                : <><Plus className="w-3 h-3" /> Create {checkedCount} ponder {checkedCount === 1 ? 'entry' : 'entries'}</>
              }
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Right pane: entry header + dialogue
// ---------------------------------------------------------------------------

function EntryDetailPane({ slug, onRefresh, onBack }: { slug: string; onRefresh: () => void; onBack: () => void }) {
  const [entry, setEntry] = useState<PonderDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [mobileWorkspaceOpen, setMobileWorkspaceOpen] = useState(false)
  const [statusModalOpen, setStatusModalOpen] = useState(false)
  const [pendingStatus, setPendingStatus] = useState<PonderStatus | null>(null)
  const { isRunning, focusRun } = useAgentRuns()
  const commitKey = `ponder-commit:${slug}`
  const commitRunning = isRunning(commitKey)

  const load = useCallback(() => {
    api.getPonderEntry(slug)
      .then(data => { setEntry(data); setError(null) })
      .catch(() => setError('Entry not found'))
      .finally(() => setLoading(false))
  }, [slug])

  const handleCommit = useCallback(async () => {
    try {
      await api.commitPonder(slug)
      focusRun(commitKey)
    } catch {
      // conflict (already running) or other error — FAB shows run state
    }
  }, [slug, commitKey, focusRun])

  const handleStatusChange = useCallback(async (newStatus: PonderStatus) => {
    await api.updatePonderEntry(slug, { status: newStatus })
    setStatusModalOpen(false)
    setPendingStatus(null)
    load()
    onRefresh()
  }, [slug, load, onRefresh])

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
        <p className="text-sm text-muted-foreground">{error ?? 'Entry not found'}</p>
      </div>
    )
  }

  const artifactCount = entry.artifacts.length

  return (
    <div className="h-full flex flex-col min-h-0 relative overflow-hidden">
      {/* Single header row — back arrow (mobile only) + title + workspace toggle */}
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
          <p className="text-xs text-muted-foreground/60 font-mono truncate hidden sm:block">{entry.slug}</p>
        </div>
        <StatusBadge status={entry.status} />
        {entry.status !== 'committed' && entry.status !== 'parked' && (
          <button
            onClick={handleCommit}
            disabled={commitRunning}
            className={cn(
              'shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border transition-colors whitespace-nowrap disabled:opacity-60 disabled:cursor-not-allowed',
              entry.status === 'converging'
                ? 'bg-primary text-primary-foreground border-primary hover:bg-primary/90'
                : 'text-muted-foreground/60 border-border/40 hover:text-foreground hover:bg-accent/50',
            )}
            title="Commit this ponder — synthesize milestones and mark committed"
          >
            {commitRunning
              ? <Loader2 className="w-3 h-3 animate-spin" />
              : <GitMerge className="w-3 h-3" />}
            <span className="hidden sm:inline">{commitRunning ? 'Committing…' : 'Commit'}</span>
          </button>
        )}
        <button
          onClick={() => { setPendingStatus(null); setStatusModalOpen(true) }}
          className="shrink-0 p-1.5 rounded-lg text-muted-foreground/40 hover:text-foreground hover:bg-accent/50 transition-colors"
          title="Change status"
        >
          <SlidersHorizontal className="w-3.5 h-3.5" />
        </button>
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

      {/* Content: dialogue + desktop right sidebar (always visible) */}
      <div className="flex-1 flex min-h-0">
        <div className="flex-1 min-w-0 min-h-0">
          <DialoguePanel
            entry={entry}
            onRefresh={() => { load(); onRefresh() }}
            onCommit={handleCommit}
            commitRunning={commitRunning}
          />
        </div>
        {/* Desktop workspace: always open */}
        <div className="hidden md:flex w-64 shrink-0 border-l border-border flex-col min-h-0">
          <WorkspacePanel artifacts={entry.artifacts} />
        </div>
      </div>

      {/* Mobile: backdrop + bottom sheet (workspace) */}
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
        <WorkspacePanel artifacts={entry.artifacts} onClose={() => setMobileWorkspaceOpen(false)} />
      </div>

      {/* Status change modal */}
      {statusModalOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={() => setStatusModalOpen(false)}
        >
          <div
            className="bg-neutral-900 border border-neutral-700 rounded-lg p-5 w-72 space-y-4"
            onClick={e => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-neutral-100">Change status</h3>
              <button
                onClick={() => setStatusModalOpen(false)}
                className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="text-xs text-muted-foreground flex items-center gap-1.5">
              Current: <StatusBadge status={entry.status} />
            </div>
            <div className="grid grid-cols-2 gap-2">
              {(['exploring', 'converging', 'committed', 'parked'] as PonderStatus[]).map(s => (
                <button
                  key={s}
                  disabled={s === entry.status}
                  onClick={() => setPendingStatus(s)}
                  className={cn(
                    'px-3 py-2 rounded text-xs font-medium border transition-colors',
                    s === entry.status
                      ? 'opacity-40 cursor-not-allowed border-neutral-700 bg-neutral-800'
                      : pendingStatus === s
                        ? 'border-violet-500 bg-violet-500/20 text-neutral-100'
                        : 'border-neutral-700 bg-neutral-800 text-neutral-300 hover:border-neutral-500',
                  )}
                >
                  {s}
                </button>
              ))}
            </div>
            <div className="flex justify-end gap-2 pt-1">
              <button
                onClick={() => setStatusModalOpen(false)}
                className="px-3 py-1.5 text-xs rounded bg-neutral-800 hover:bg-neutral-700 text-neutral-300 transition-colors"
              >
                Cancel
              </button>
              <button
                disabled={!pendingStatus}
                onClick={() => pendingStatus && handleStatusChange(pendingStatus)}
                className="px-3 py-1.5 text-xs rounded bg-violet-600 hover:bg-violet-500 text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
              >
                Apply
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function PonderPage() {
  const { slug } = useParams<{ slug: string }>()
  const navigate = useNavigate()
  const [entries, setEntries] = useState<PonderSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<PonderStatus | 'all'>('all')
  const [showForm, setShowForm] = useState(false)
  const [showSuggest, setShowSuggest] = useState(false)
  const [prefillTitle, setPrefillTitle] = useState<string | null>(null)
  const [prefillSlug, setPrefillSlug] = useState<string | null>(null)
  const [prefillBrief, setPrefillBrief] = useState<string | null>(null)

  const load = useCallback(() => {
    api.getRoadmap()
      .then(data => setEntries(data))
      .catch(() => { })
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { load() }, [load])
  useSSE(load)

  const STATUS_ORDER: Record<PonderStatus, number> = {
    converging: 0,
    exploring: 1,
    committed: 2,
    parked: 3,
  }

  const sorted = [...entries].sort((a, b) => {
    const sd = STATUS_ORDER[a.status] - STATUS_ORDER[b.status]
    if (sd !== 0) return sd
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  })

  const filtered = activeTab === 'all'
    ? sorted
    : sorted.filter(e => e.status === activeTab)

  // Mobile: if slug is present, show detail only
  const showMobileDetail = !!slug

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 flex min-h-0">
        {/* Left pane: entry list */}
        <div className={cn(
          'w-72 shrink-0 border-r border-border flex flex-col bg-card',
          showMobileDetail ? 'hidden md:flex' : 'flex',
        )}>
          {/* Header */}
          <div className="px-3 pt-4 pb-2 flex items-center justify-between">
            <h2 className="text-base font-semibold">Ponder</h2>
            {!showForm && (
              <div className="flex items-center gap-1">
                <button
                  onClick={() => setShowSuggest(true)}
                  className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
                  title="Suggest an idea"
                >
                  <Sparkles className="w-4 h-4" />
                </button>
                <button
                  onClick={() => setShowForm(true)}
                  className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
                  title="New Idea"
                >
                  <Plus className="w-4 h-4" />
                </button>
              </div>
            )}
          </div>

          {/* New idea form */}
          {showForm && (
            <NewIdeaForm
              initialTitle={prefillTitle ?? undefined}
              initialSlug={prefillSlug ?? undefined}
              initialBrief={prefillBrief ?? undefined}
              onCreated={(newSlug) => {
                setShowForm(false)
                setPrefillTitle(null)
                setPrefillSlug(null)
                setPrefillBrief(null)
                load()
                navigate(`/ponder/${newSlug}`)
              }}
              onCancel={() => {
                setShowForm(false)
                setPrefillTitle(null)
                setPrefillSlug(null)
                setPrefillBrief(null)
              }}
            />
          )}

          {/* Status labels */}
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

          {/* Entry list */}
          <div className="flex-1 overflow-y-auto px-2 pb-2 space-y-0.5">
            {loading ? (
              <div className="space-y-2 px-1 pt-2">
                <Skeleton width="w-full" className="h-12" />
                <Skeleton width="w-full" className="h-12" />
                <Skeleton width="w-full" className="h-12" />
              </div>
            ) : filtered.length === 0 ? (
              <div className="text-center py-8 px-3">
                <p className="text-xs text-muted-foreground">
                  {activeTab === 'all'
                    ? 'No ideas yet.'
                    : `No ${activeTab} entries.`}
                </p>
                {activeTab === 'all' && !showForm && (
                  <button
                    onClick={() => setShowForm(true)}
                    className="mt-2 inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors whitespace-nowrap"
                  >
                    <Plus className="w-3 h-3" />
                    New Idea
                  </button>
                )}
              </div>
            ) : (
              filtered.map(entry => (
                <EntryRow
                  key={entry.slug}
                  entry={entry}
                  selected={entry.slug === slug}
                  onSelect={() => navigate(`/ponder/${entry.slug}`)}
                />
              ))
            )}
          </div>
        </div>

        {/* Right pane: detail */}
        <div className={cn(
          'flex-1 min-w-0',
          showMobileDetail ? 'flex flex-col' : 'hidden md:flex md:flex-col',
        )}>
          {slug ? (
            <EntryDetailPane key={slug} slug={slug} onRefresh={load} onBack={() => navigate('/ponder')} />
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <div className="text-center">
                <Lightbulb className="w-8 h-8 mx-auto mb-3 opacity-30" />
                <p className="text-sm">Select an idea to explore</p>
                <div className="flex items-center justify-center gap-2 mt-2">
                  <button
                    onClick={() => setShowSuggest(true)}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-primary/10 text-primary border border-primary/20 rounded-lg hover:bg-primary/20 transition-colors"
                  >
                    <Sparkles className="w-3 h-3" />
                    Suggest an idea
                  </button>
                  <span className="text-xs text-muted-foreground/40">or</span>
                  <button
                    onClick={() => setShowForm(true)}
                    className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-accent/50 rounded-lg transition-colors"
                  >
                    <Plus className="w-3 h-3" />
                    New idea
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Advisory panel */}
      {showSuggest && (
        <AdvisoryPanel
          onCreated={(firstSlug) => {
            setShowSuggest(false)
            load()
            navigate(`/ponder/${firstSlug}`)
          }}
          onClose={() => setShowSuggest(false)}
        />
      )}
    </div>
  )
}

