import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { Skeleton } from '@/components/shared/Skeleton'
import { NewResearchModal } from '@/components/knowledge/NewResearchModal'
import {
  Library, ChevronRight, ArrowLeft, ExternalLink, AlertTriangle,
  Clock, RefreshCw, Loader2, Tag, FlaskConical,
  BookOpen, Search, Layers,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import type {
  KnowledgeCatalog,
  KnowledgeCatalogClass,
  KnowledgeEntrySummary,
  KnowledgeEntryDetail,
  KnowledgeStatus,
} from '@/lib/types'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function statusColor(status: KnowledgeStatus): string {
  switch (status) {
    case 'published': return 'text-emerald-500'
    case 'draft':     return 'text-yellow-500'
    case 'archived':  return 'text-muted-foreground/40'
    default:          return 'text-muted-foreground'
  }
}

function stalenessLabel(flags: string[]): string | null {
  if (!flags || flags.length === 0) return null
  if (flags.includes('url_404')) return 'URL not found'
  if (flags.includes('aged_out')) return 'May be outdated'
  if (flags.includes('code_ref_gone')) return 'Code reference missing'
  return flags[0].replace(/_/g, ' ')
}

// ---------------------------------------------------------------------------
// CatalogPane (T6)
// ---------------------------------------------------------------------------

function CatalogPane({
  catalog,
  loading,
  selectedCode,
  onSelect,
}: {
  catalog: KnowledgeCatalog | null
  loading: boolean
  selectedCode: string | null
  onSelect: (code: string | null) => void
}) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  const toggle = (code: string) => {
    setExpanded(prev => {
      const next = new Set(prev)
      if (next.has(code)) next.delete(code)
      else next.add(code)
      return next
    })
  }

  if (loading) {
    return (
      <div className="space-y-2 px-2 pt-3">
        <Skeleton width="w-full" className="h-7" />
        <Skeleton width="w-full" className="h-7" />
        <Skeleton width="w-full" className="h-7" />
      </div>
    )
  }

  const classes = catalog?.classes ?? []

  return (
    <div className="flex-1 overflow-y-auto px-2 pb-2">
      {/* All entries */}
      <button
        onClick={() => onSelect(null)}
        className={cn(
          'w-full text-left flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors',
          selectedCode === null
            ? 'bg-accent text-accent-foreground font-medium'
            : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
        )}
      >
        <Library className="w-3.5 h-3.5 shrink-0" />
        All entries
      </button>

      {classes.length === 0 && (
        <p className="text-xs text-muted-foreground/60 text-center py-4 px-2">
          No catalog yet. Run <code className="font-mono">sdlc knowledge librarian init</code> to seed.
        </p>
      )}

      {classes.map((cls: KnowledgeCatalogClass) => {
        const isOpen = expanded.has(cls.code)
        const isActive = selectedCode?.startsWith(cls.code) && selectedCode !== null
        return (
          <div key={cls.code}>
            <button
              onClick={() => { toggle(cls.code); onSelect(cls.code) }}
              className={cn(
                'w-full text-left flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors mt-0.5',
                isActive
                  ? 'bg-accent text-accent-foreground font-medium'
                  : 'text-foreground hover:bg-accent/50',
              )}
            >
              <ChevronRight
                className={cn('w-3 h-3 shrink-0 transition-transform', isOpen && 'rotate-90')}
              />
              <span className="font-mono text-xs text-muted-foreground/60 w-8 shrink-0">{cls.code}</span>
              <span className="flex-1 truncate">{cls.name}</span>
            </button>

            {isOpen && cls.divisions.map(div => (
              <button
                key={div.code}
                onClick={() => onSelect(div.code)}
                className={cn(
                  'w-full text-left flex items-center gap-2 pl-9 pr-3 py-1 rounded-lg text-xs transition-colors',
                  selectedCode === div.code
                    ? 'bg-accent text-accent-foreground font-medium'
                    : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
                )}
              >
                <span className="font-mono text-muted-foreground/50 w-12 shrink-0">{div.code}</span>
                <span className="flex-1 truncate">{div.name}</span>
              </button>
            ))}
          </div>
        )
      })}
    </div>
  )
}

// ---------------------------------------------------------------------------
// EntryListPane (T7)
// ---------------------------------------------------------------------------

function EntryListPane({
  entries,
  loading,
  selectedSlug,
  onSelect,
  onResearch,
}: {
  entries: KnowledgeEntrySummary[]
  loading: boolean
  selectedSlug: string | null
  onSelect: (slug: string) => void
  onResearch: (slug: string, title: string) => void
}) {
  if (loading) {
    return (
      <div className="space-y-2 px-2 pt-3">
        <Skeleton width="w-full" className="h-12" />
        <Skeleton width="w-full" className="h-12" />
        <Skeleton width="w-full" className="h-12" />
      </div>
    )
  }

  if (entries.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-center px-4">
        <div>
          <Library className="w-7 h-7 mx-auto mb-2 opacity-20" />
          <p className="text-xs text-muted-foreground">No entries in this category.</p>
          <p className="text-xs text-muted-foreground/50 mt-1">
            Use <code className="font-mono">sdlc knowledge add</code> to add one.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto px-2 pb-2 space-y-0.5">
      {entries.map(entry => (
        <button
          key={entry.slug}
          onClick={() => onSelect(entry.slug)}
          className={cn(
            'group w-full text-left px-3 py-2.5 rounded-lg transition-colors',
            selectedSlug === entry.slug
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/40 text-foreground',
          )}
        >
          <div className="flex items-start gap-2">
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium truncate leading-snug">{entry.title}</p>
              {entry.summary && (
                <p className="text-xs text-muted-foreground/70 mt-0.5 line-clamp-2 leading-snug">
                  {entry.summary}
                </p>
              )}
            </div>
            <div className="shrink-0 flex items-center gap-1 mt-0.5">
              <span className={cn('text-xs font-mono', statusColor(entry.status))}>
                {entry.status}
              </span>
              <button
                type="button"
                aria-label={`Research: ${entry.title}`}
                title="Research More"
                onClick={e => {
                  e.stopPropagation()
                  onResearch(entry.slug, entry.title)
                }}
                className="opacity-0 group-hover:opacity-100 transition-opacity p-0.5 rounded text-muted-foreground hover:text-primary"
              >
                <FlaskConical className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
          {entry.tags.length > 0 && (
            <div className="flex items-center gap-1 mt-1 flex-wrap">
              {entry.tags.slice(0, 4).map(tag => (
                <span
                  key={tag}
                  className="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded-full bg-muted/50 text-muted-foreground/60"
                >
                  {tag}
                </span>
              ))}
              {entry.tags.length > 4 && (
                <span className="text-[10px] text-muted-foreground/40">+{entry.tags.length - 4}</span>
              )}
            </div>
          )}
        </button>
      ))}
    </div>
  )
}

// ---------------------------------------------------------------------------
// EntryDetailPane (T8)
// ---------------------------------------------------------------------------

function EntryDetailPane({
  slug,
  onBack,
}: {
  slug: string
  onBack: () => void
}) {
  const [entry, setEntry] = useState<KnowledgeEntryDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [researching, setResearching] = useState(false)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    api.getKnowledgeEntry(slug)
      .then(data => setEntry(data))
      .catch(() => setError('Entry not found'))
      .finally(() => setLoading(false))
  }, [slug])

  useEffect(() => {
    setEntry(null)
    load()
  }, [load])

  const handleResearch = async () => {
    if (!entry) return
    setResearching(true)
    try {
      await api.researchKnowledge(slug)
    } catch {
      // silently ignore — agent run starts async
    } finally {
      setResearching(false)
    }
  }

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

  const stale = stalenessLabel(entry.staleness_flags)

  return (
    <div className="h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="shrink-0 px-4 pt-4 pb-3 border-b border-border/50">
        <div className="flex items-start gap-2">
          <button
            onClick={onBack}
            className="md:hidden shrink-0 -ml-1 p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Back to knowledge list"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-base font-semibold leading-snug">{entry.title}</h2>
              <span className={cn('text-xs font-mono', statusColor(entry.status))}>
                {entry.status}
              </span>
              <span className="text-xs font-mono text-muted-foreground/50">{entry.code}</span>
            </div>
            {entry.summary && (
              <p className="text-sm text-muted-foreground mt-1 leading-snug">{entry.summary}</p>
            )}
          </div>
          <button
            onClick={handleResearch}
            disabled={researching}
            aria-busy={researching}
            title="Research More"
            className={cn(
              'shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg transition-colors',
              researching
                ? 'bg-muted text-muted-foreground cursor-not-allowed'
                : 'bg-primary/10 text-primary hover:bg-primary/20',
            )}
          >
            {researching
              ? <Loader2 className="w-3 h-3 animate-spin" />
              : <RefreshCw className="w-3 h-3" />}
            {researching ? 'Researching…' : 'Research More'}
          </button>
        </div>

        {stale && (
          <div className="mt-2 flex items-center gap-1.5 text-xs text-amber-500">
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            {stale}
          </div>
        )}

        {entry.tags.length > 0 && (
          <div className="flex items-center gap-1 mt-2 flex-wrap">
            <Tag className="w-3 h-3 text-muted-foreground/50 shrink-0" />
            {entry.tags.map(tag => (
              <span
                key={tag}
                className="inline-flex items-center text-[10px] px-1.5 py-0.5 rounded-full bg-muted/60 text-muted-foreground/60"
              >
                {tag}
              </span>
            ))}
          </div>
        )}

        {entry.sources.length > 0 && (
          <div className="mt-2 flex items-center gap-3 flex-wrap">
            {entry.sources.map((src, i) => (
              src.url ? (
                <a
                  key={i}
                  href={src.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-xs text-primary/70 hover:text-primary transition-colors"
                >
                  <ExternalLink className="w-3 h-3" />
                  {src.type}
                </a>
              ) : (
                <span key={i} className="text-xs text-muted-foreground/50 font-mono">
                  {src.type}{src.path ? `: ${src.path}` : ''}
                </span>
              )
            ))}
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-3">
        {entry.last_verified_at && (
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground/50 mb-3">
            <Clock className="w-3 h-3" />
            Last verified: {new Date(entry.last_verified_at).toLocaleDateString()}
          </div>
        )}

        {entry.content ? (
          <div className="prose prose-sm prose-invert max-w-none">
            <pre className="whitespace-pre-wrap text-sm text-foreground/80 font-sans leading-relaxed">
              {entry.content}
            </pre>
          </div>
        ) : (
          <div className="flex items-center justify-center h-32 text-center">
            <div>
              <p className="text-sm text-muted-foreground/60">No content yet.</p>
              <p className="text-xs text-muted-foreground/40 mt-1">
                Click "Research More" to generate content.
              </p>
            </div>
          </div>
        )}

        {entry.related.length > 0 && (
          <div className="mt-4 pt-3 border-t border-border/30">
            <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
              Related
            </p>
            <div className="flex flex-wrap gap-1.5">
              {entry.related.map(slug => (
                <span
                  key={slug}
                  className="text-xs font-mono px-2 py-0.5 rounded-full bg-muted/50 text-muted-foreground/70"
                >
                  {slug}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// KnowledgePage root (T9)
// ---------------------------------------------------------------------------

export function KnowledgePage() {
  const { slug } = useParams<{ slug: string }>()
  const navigate = useNavigate()

  const [catalog, setCatalog] = useState<KnowledgeCatalog | null>(null)
  const [catalogLoading, setCatalogLoading] = useState(true)

  const [entries, setEntries] = useState<KnowledgeEntrySummary[]>([])
  const [entriesLoading, setEntriesLoading] = useState(false)
  const [selectedCode, setSelectedCode] = useState<string | null>(null)

  const [researchTarget, setResearchTarget] = useState<{ slug: string; title: string } | null>(null)

  // Load catalog once on mount
  useEffect(() => {
    api.getKnowledgeCatalog()
      .then(data => setCatalog(data))
      .catch(() => setCatalog(null))
      .finally(() => setCatalogLoading(false))
  }, [])

  // Load entry list whenever selectedCode changes
  const loadEntries = useCallback(() => {
    setEntriesLoading(true)
    api.listKnowledge(selectedCode ? { code: selectedCode } : undefined)
      .then(data => setEntries(data))
      .catch(() => setEntries([]))
      .finally(() => setEntriesLoading(false))
  }, [selectedCode])

  useEffect(() => { loadEntries() }, [loadEntries])

  // Re-fetch on SSE updates (knowledge events and run completions)
  const handleUpdate = useCallback(() => { loadEntries() }, [loadEntries])
  useSSE(handleUpdate)

  const showMobileDetail = !!slug

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 flex min-h-0">

        {/* Left pane: catalog tree */}
        <div className={cn(
          'w-52 shrink-0 border-r border-border flex flex-col bg-card',
          showMobileDetail ? 'hidden md:flex' : 'flex',
        )}>
          <div className="px-3 pt-4 pb-2 shrink-0">
            <h2 className="text-base font-semibold">Knowledge</h2>
            <p className="text-xs text-muted-foreground/60 mt-0.5">Catalog</p>
          </div>
          <div className="border-b border-border mx-3 mb-1 shrink-0" />
          <CatalogPane
            catalog={catalog}
            loading={catalogLoading}
            selectedCode={selectedCode}
            onSelect={code => {
              setSelectedCode(code)
              // Clear slug selection when switching categories on mobile
              if (slug) navigate('/knowledge')
            }}
          />
        </div>

        {/* Middle pane: entry list */}
        <div className={cn(
          'w-64 shrink-0 border-r border-border flex flex-col bg-card/50',
          showMobileDetail ? 'hidden md:flex' : 'flex',
        )}>
          <div className="px-3 pt-4 pb-2 shrink-0">
            <p className="text-xs text-muted-foreground/60 font-medium">
              {selectedCode ? `Code: ${selectedCode}` : 'All entries'}
              {!entriesLoading && (
                <span className="ml-2 text-muted-foreground/40 tabular-nums">({entries.length})</span>
              )}
            </p>
          </div>
          <div className="border-b border-border mx-3 mb-1 shrink-0" />
          <EntryListPane
            entries={entries}
            loading={entriesLoading}
            selectedSlug={slug ?? null}
            onSelect={s => navigate(`/knowledge/${s}`)}
            onResearch={(entrySlug, entryTitle) => setResearchTarget({ slug: entrySlug, title: entryTitle })}
          />
        </div>

        {/* Right pane: entry detail */}
        <div className={cn(
          'flex-1 min-w-0',
          showMobileDetail ? 'flex flex-col' : 'hidden md:flex md:flex-col',
        )}>
          {slug ? (
            <EntryDetailPane
              key={slug}
              slug={slug}
              onBack={() => navigate('/knowledge')}
            />
          ) : (
            <div className="h-full overflow-y-auto">
              <div className="max-w-xl mx-auto px-6 py-10 space-y-8">
                {/* Hero */}
                <div className="text-center space-y-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-primary/10 mb-1">
                    <Library className="w-6 h-6 text-primary" />
                  </div>
                  <h2 className="text-xl font-semibold">What the team knows.</h2>
                  <p className="text-sm text-muted-foreground leading-relaxed max-w-md mx-auto">
                    Knowledge is a structured catalog of everything the project has learned —
                    decisions, patterns, rejected approaches, and external references. Organized
                    by a Dewey Decimal-inspired catalog so nothing gets lost.
                  </p>
                </div>

                {/* How it works */}
                <div className="space-y-3">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">How it works</h3>
                  <div className="grid gap-2">
                    {[
                      { icon: BookOpen, title: 'Catalog organized', desc: 'Entries are classified into a hierarchical catalog — browse by class and division, or search across all.' },
                      { icon: Search, title: 'Research agents', desc: 'Point a research agent at any entry to expand it with web sources, codebase evidence, and cross-references.' },
                      { icon: Layers, title: 'Staleness tracking', desc: 'Entries are monitored for drift — broken URLs, missing code references, and aged-out content get flagged.' },
                    ].map(({ icon: Icon, title, desc }) => (
                      <div key={title} className="flex items-start gap-3 p-3 rounded-lg border border-border/50 bg-card/50">
                        <Icon className="w-4 h-4 mt-0.5 shrink-0 text-muted-foreground" />
                        <div className="min-w-0">
                          <p className="text-sm font-medium">{title}</p>
                          <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{desc}</p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* CTA */}
                <div className="text-center pt-2">
                  <div className="inline-flex items-center gap-1 bg-muted/60 border border-border rounded-lg px-3 py-2 text-xs font-mono text-muted-foreground">
                    sdlc knowledge add &lt;slug&gt;
                  </div>
                  <p className="text-xs text-muted-foreground/60 mt-2">
                    Add entries from the CLI, or they accumulate naturally from spikes, investigations, and guidelines.
                  </p>
                </div>

                {entries.length > 0 && (
                  <p className="text-center text-xs text-muted-foreground/40">
                    Or select an entry from the catalog to read it.
                  </p>
                )}
              </div>
            </div>
          )}
        </div>

      </div>

      {researchTarget && (
        <NewResearchModal
          open
          entrySlug={researchTarget.slug}
          entryTitle={researchTarget.title}
          onClose={() => setResearchTarget(null)}
          onStarted={() => setResearchTarget(null)}
        />
      )}
    </div>
  )
}
