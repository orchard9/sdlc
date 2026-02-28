import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { Skeleton } from '@/components/shared/Skeleton'
import { DialoguePanel } from '@/components/ponder/DialoguePanel'
import { WorkspacePanel } from '@/components/ponder/WorkspacePanel'
import {
  Plus, X, ArrowLeft, Lightbulb, Loader2, Users, Files, GitMerge, Check,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PonderSummary, PonderStatus, PonderDetail } from '@/lib/types'

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

function NewIdeaForm({ onCreated, onCancel }: { onCreated: (slug: string) => void; onCancel: () => void }) {
  const [slug, setSlug] = useState('')
  const [title, setTitle] = useState('')
  const [brief, setBrief] = useState('')
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
// Right pane: entry header + dialogue
// ---------------------------------------------------------------------------

function EntryDetailPane({ slug, onRefresh, onBack }: { slug: string; onRefresh: () => void; onBack: () => void }) {
  const [entry, setEntry] = useState<PonderDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [mobileWorkspaceOpen, setMobileWorkspaceOpen] = useState(false)
  const [commitCopied, setCommitCopied] = useState(false)

  const load = useCallback(() => {
    api.getPonderEntry(slug)
      .then(data => { setEntry(data); setError(null) })
      .catch(() => setError('Entry not found'))
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
        {entry.sessions > 0 && entry.status !== 'committed' && entry.status !== 'parked' && (
          <button
            onClick={() => {
              navigator.clipboard.writeText(`/sdlc-ponder-commit ${entry.slug}`)
              setCommitCopied(true)
              setTimeout(() => setCommitCopied(false), 2000)
            }}
            className={cn(
              'shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border transition-colors',
              entry.status === 'converging'
                ? 'bg-primary text-primary-foreground border-primary hover:bg-primary/90'
                : 'text-muted-foreground/60 border-border/40 hover:text-foreground hover:bg-accent/50',
            )}
            title="Copy /sdlc-ponder-commit command"
          >
            {commitCopied
              ? <Check className="w-3 h-3" />
              : <GitMerge className="w-3 h-3" />}
            <span className="hidden sm:inline">{commitCopied ? 'Copied!' : 'Commit'}</span>
          </button>
        )}
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
          <DialoguePanel entry={entry} onRefresh={() => { load(); onRefresh() }} />
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

  const load = useCallback(() => {
    api.getRoadmap()
      .then(data => setEntries(data))
      .catch(() => { })
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { load() }, [load])
  useSSE(load)

  const filtered = activeTab === 'all'
    ? entries
    : entries.filter(e => e.status === activeTab)

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
              <button
                onClick={() => setShowForm(true)}
                className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
                title="New Idea"
              >
                <Plus className="w-4 h-4" />
              </button>
            )}
          </div>

          {/* New idea form */}
          {showForm && (
            <NewIdeaForm
              onCreated={(newSlug) => {
                setShowForm(false)
                load()
                navigate(`/ponder/${newSlug}`)
              }}
              onCancel={() => setShowForm(false)}
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
                    className="mt-2 inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
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
                <p className="text-xs text-muted-foreground/60 mt-1">
                  or press <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">+</kbd> to start a new one
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

