import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { api } from '@/api/client'
import { Skeleton } from '@/components/shared/Skeleton'
import { FlaskConical, ArrowLeft, Copy, Check, ExternalLink, HelpCircle, Beaker, Scale } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { SpikeSummary, SpikeDetail, SpikeVerdict } from '@/lib/types'

// ---------------------------------------------------------------------------
// Verdict badge
// ---------------------------------------------------------------------------

function VerdictBadge({ verdict, size = 'sm' }: { verdict: SpikeVerdict; size?: 'sm' | 'md' }) {
  const classes: Record<SpikeVerdict, string> = {
    ADOPT: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300',
    ADAPT: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300',
    REJECT: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300',
  }
  return (
    <span className={cn(
      'inline-flex items-center font-semibold rounded tracking-wide shrink-0',
      size === 'sm' ? 'text-[10px] px-1.5 py-0.5' : 'text-xs px-2 py-1',
      classes[verdict],
    )}>
      {verdict}
    </span>
  )
}

// ---------------------------------------------------------------------------
// Verdict filter tabs
// ---------------------------------------------------------------------------

type VerdictFilter = 'all' | SpikeVerdict

const FILTER_TABS: { label: string; value: VerdictFilter }[] = [
  { label: 'All', value: 'all' },
  { label: 'ADOPT', value: 'ADOPT' },
  { label: 'ADAPT', value: 'ADAPT' },
  { label: 'REJECT', value: 'REJECT' },
]

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

function SpikeEmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <FlaskConical className="w-10 h-10 mb-4 opacity-20 text-foreground" />
      <h3 className="text-sm font-semibold mb-2">No spikes yet</h3>
      <p className="text-xs text-muted-foreground leading-relaxed max-w-xs mb-4">
        Spikes are time-boxed investigations that answer one focused question.
        When a spike concludes, it gets an <span className="font-medium">ADOPT</span>,{' '}
        <span className="font-medium">ADAPT</span>, or{' '}
        <span className="font-medium">REJECT</span> verdict — and that verdict drives what happens next.
      </p>
      <div className="text-xs font-mono bg-muted/60 border border-border rounded-lg px-3 py-2 text-muted-foreground">
        /sdlc-spike &lt;slug&gt; — &lt;the question to answer&gt;
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Spike row (left pane list item)
// ---------------------------------------------------------------------------

function SpikeRow({
  spike,
  selected,
  onSelect,
}: {
  spike: SpikeSummary
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
      <div className="flex items-center gap-2 mb-1">
        <span className="text-sm font-medium truncate flex-1">{spike.title}</span>
        <VerdictBadge verdict={spike.verdict} />
      </div>
      <p className="text-xs text-muted-foreground line-clamp-1 mb-1.5">{spike.the_question}</p>
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-[10px] text-muted-foreground/60">{spike.date}</span>
        {spike.verdict === 'ADOPT' && (
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300 font-medium shrink-0">
            Next: /sdlc-hypothetical-planning
          </span>
        )}
        {spike.verdict === 'ADAPT' && spike.ponder_slug && (
          <span className="text-[10px] text-muted-foreground/80 truncate">
            Ponder: {spike.ponder_slug}
          </span>
        )}
        {spike.verdict === 'REJECT' && (
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300 font-medium shrink-0">
            Stored in Knowledge
          </span>
        )}
      </div>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Copy button (inline)
// ---------------------------------------------------------------------------

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <button
      onClick={handleCopy}
      className="shrink-0 p-1.5 rounded text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
      title="Copy to clipboard"
    >
      {copied ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
    </button>
  )
}

// ---------------------------------------------------------------------------
// Detail pane — ADOPT section
// ---------------------------------------------------------------------------

function AdoptSection({ slug }: { slug: string }) {
  const command = `/sdlc-hypothetical-planning ${slug}`
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-2">
        What's Next
      </p>
      <p className="text-sm text-muted-foreground leading-relaxed mb-3">
        <span className="font-semibold text-green-600 dark:text-green-400">ADOPT</span> means the approach
        is proven — not yet implemented. The spike answered the question; now it's time to plan the build.
      </p>
      <div className="flex items-center gap-1 bg-muted/60 border border-border rounded-lg px-3 py-2">
        <code className="flex-1 text-xs font-mono text-foreground">{command}</code>
        <CopyButton text={command} />
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Detail pane — ADAPT section
// ---------------------------------------------------------------------------

function AdaptSection({
  spike,
  onPromoted,
}: {
  spike: SpikeDetail
  onPromoted: (ponderSlug: string) => void
}) {
  const navigate = useNavigate()
  const [promoting, setPromoting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handlePromote = async () => {
    setPromoting(true)
    setError(null)
    try {
      const result = await api.promoteSpike(spike.slug)
      onPromoted(result.ponder_slug)
      navigate(`/ponder/${result.ponder_slug}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Promote failed')
    } finally {
      setPromoting(false)
    }
  }

  if (spike.ponder_slug) {
    return (
      <div className="rounded-lg border border-border bg-card p-4">
        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-2">
          Ponder Entry
        </p>
        <p className="text-sm text-muted-foreground leading-relaxed mb-3">
          This spike has already been promoted to a Ponder entry.
        </p>
        <Link
          to={`/ponder/${spike.ponder_slug}`}
          className="inline-flex items-center gap-1.5 text-sm text-primary hover:underline"
        >
          <ExternalLink className="w-3.5 h-3.5" />
          {spike.ponder_slug}
        </Link>
      </div>
    )
  }

  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-2">
        Promote to Ponder
      </p>
      <p className="text-sm text-muted-foreground leading-relaxed mb-3">
        <span className="font-semibold text-yellow-600 dark:text-yellow-400">ADAPT</span> means the
        approach needs refinement. Promote this spike to a Ponder entry to explore and refine the idea.
      </p>
      {error && (
        <p className="text-xs text-destructive mb-2">{error}</p>
      )}
      <button
        onClick={handlePromote}
        disabled={promoting}
        className="px-4 py-2 text-sm font-medium bg-yellow-500 text-white rounded-lg hover:bg-yellow-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {promoting ? 'Promoting...' : 'Promote to Ponder →'}
      </button>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Detail pane — REJECT section
// ---------------------------------------------------------------------------

function RejectSection({ spike }: { spike: SpikeDetail }) {
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-2">
        Stored in Knowledge
      </p>
      <p className="text-sm text-muted-foreground leading-relaxed mb-3">
        <span className="font-semibold text-red-600 dark:text-red-400">REJECT</span> means the approach
        was not viable. The findings have been filed in the knowledge base for future reference.
      </p>
      {spike.knowledge_slug ? (
        <Link
          to={`/knowledge/${spike.knowledge_slug}`}
          className="inline-flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300 hover:opacity-80 transition-opacity"
        >
          <ExternalLink className="w-3 h-3" />
          {spike.knowledge_slug}
        </Link>
      ) : (
        <span className="text-xs px-2.5 py-1.5 rounded-lg bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300">
          Stored in Knowledge
        </span>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Detail pane
// ---------------------------------------------------------------------------

function SpikeDetailPane({
  slug,
  onBack,
  onSpikeUpdated,
}: {
  slug: string
  onBack: () => void
  onSpikeUpdated: () => void
}) {
  const [spike, setSpike] = useState<SpikeDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setSpike(null)
    api.getSpike(slug)
      .then(data => { setSpike(data); setError(null) })
      .catch(() => setError('Spike not found'))
      .finally(() => setLoading(false))
  }, [slug])

  useEffect(() => { load() }, [load])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-5 h-5 border-2 border-border border-t-primary rounded-full animate-spin" />
      </div>
    )
  }

  if (error || !spike) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-sm text-muted-foreground">{error ?? 'Spike not found'}</p>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="shrink-0 px-5 pt-4 pb-3 border-b border-border/50">
        <div className="flex items-center gap-2 mb-1">
          <button
            onClick={onBack}
            className="lg:hidden shrink-0 -ml-1 p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Back"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <p className="text-xs text-muted-foreground/60">
            <Link to="/spikes" className="hover:text-muted-foreground transition-colors">Spikes</Link>
            {' / '}
            <span className="text-muted-foreground">{spike.title}</span>
          </p>
        </div>
        <div className="flex items-start gap-3">
          <h2 className="text-lg font-semibold leading-snug flex-1">{spike.title}</h2>
          <VerdictBadge verdict={spike.verdict} size="md" />
        </div>
        <p className="mt-2 text-sm text-muted-foreground italic leading-relaxed">
          "{spike.the_question}"
        </p>
        <p className="mt-1.5 text-xs text-muted-foreground/50">{spike.date}</p>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto p-5 space-y-4">
        {spike.verdict === 'ADOPT' && (
          <AdoptSection slug={spike.slug} />
        )}
        {spike.verdict === 'ADAPT' && (
          <AdaptSection
            spike={spike}
            onPromoted={() => {
              load()
              onSpikeUpdated()
            }}
          />
        )}
        {spike.verdict === 'REJECT' && (
          <RejectSection spike={spike} />
        )}

        {/* Ponder lineage (shown when not ADAPT or when ponder_slug is set for non-ADAPT) */}
        {spike.ponder_slug && spike.verdict !== 'ADAPT' && (
          <div className="rounded-lg border border-border bg-card p-4">
            <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-2">
              Ponder Lineage
            </p>
            <Link
              to={`/ponder/${spike.ponder_slug}`}
              className="inline-flex items-center gap-1.5 text-sm text-primary hover:underline"
            >
              <ExternalLink className="w-3.5 h-3.5" />
              {spike.ponder_slug}
            </Link>
          </div>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function SpikePage() {
  const { slug } = useParams<{ slug: string }>()
  const navigate = useNavigate()
  const [spikes, setSpikes] = useState<SpikeSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [listError, setListError] = useState<string | null>(null)
  const [activeFilter, setActiveFilter] = useState<VerdictFilter>('all')

  const load = useCallback(() => {
    api.getSpikes()
      .then(data => { setSpikes(data); setListError(null) })
      .catch(() => setListError('Failed to load spikes'))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { load() }, [load])

  const filtered = activeFilter === 'all'
    ? spikes
    : spikes.filter(s => s.verdict === activeFilter)

  const showMobileDetail = !!slug

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 flex min-h-0">
        {/* Left pane: spike list */}
        <div className={cn(
          'w-72 shrink-0 border-r border-border flex flex-col bg-card',
          showMobileDetail ? 'hidden lg:flex' : 'flex',
        )}>
          {/* Header */}
          <div className="px-3 pt-4 pb-2 flex items-center gap-2">
            <FlaskConical className="w-4 h-4 text-muted-foreground" />
            <h2 className="text-base font-semibold">Spikes</h2>
          </div>

          {/* Verdict filter tabs */}
          <div className="px-2 pb-2 space-y-0.5">
            {FILTER_TABS.map(tab => {
              const count = tab.value === 'all'
                ? spikes.length
                : spikes.filter(s => s.verdict === tab.value).length
              return (
                <button
                  key={tab.value}
                  onClick={() => setActiveFilter(tab.value)}
                  className={cn(
                    'w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
                    activeFilter === tab.value
                      ? 'bg-accent text-accent-foreground'
                      : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
                  )}
                >
                  <span>{tab.label}</span>
                  <span className={cn(
                    'tabular-nums',
                    activeFilter === tab.value ? 'text-accent-foreground/70' : 'text-muted-foreground/50',
                  )}>
                    {count}
                  </span>
                </button>
              )
            })}
          </div>
          <div className="border-b border-border mx-3 mb-1" />

          {/* List */}
          <div className="flex-1 overflow-y-auto px-2 pb-2 space-y-0.5">
            {loading ? (
              <div className="space-y-2 px-1 pt-2">
                <Skeleton width="w-full" className="h-16" />
                <Skeleton width="w-full" className="h-16" />
                <Skeleton width="w-full" className="h-16" />
              </div>
            ) : listError ? (
              <p className="text-xs text-destructive px-3 py-4">{listError}</p>
            ) : filtered.length === 0 ? (
              activeFilter === 'all' ? (
                <SpikeEmptyState />
              ) : (
                <p className="text-xs text-muted-foreground text-center py-8 px-3">
                  No {activeFilter} spikes.
                </p>
              )
            ) : (
              filtered.map(spike => (
                <SpikeRow
                  key={spike.slug}
                  spike={spike}
                  selected={spike.slug === slug}
                  onSelect={() => navigate(`/spikes/${spike.slug}`)}
                />
              ))
            )}
          </div>
        </div>

        {/* Right pane: detail */}
        <div className={cn(
          'flex-1 min-w-0',
          showMobileDetail ? 'flex flex-col' : 'hidden lg:flex lg:flex-col',
        )}>
          {slug ? (
            <SpikeDetailPane
              key={slug}
              slug={slug}
              onBack={() => navigate('/spikes')}
              onSpikeUpdated={load}
            />
          ) : (
            <div className="h-full overflow-y-auto">
              <div className="max-w-xl mx-auto px-6 py-10 space-y-8">
                {/* Hero */}
                <div className="text-center space-y-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-primary/10 mb-1">
                    <FlaskConical className="w-6 h-6 text-primary" />
                  </div>
                  <h2 className="text-xl font-semibold">Answer one question fast.</h2>
                  <p className="text-sm text-muted-foreground leading-relaxed max-w-md mx-auto">
                    Spikes are time-boxed investigations that answer a focused technical question.
                    Each spike concludes with a verdict that drives what happens next.
                  </p>
                </div>

                {/* How it works */}
                <div className="space-y-3">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">How it works</h3>
                  <div className="grid gap-2">
                    {[
                      { icon: HelpCircle, title: 'Ask a question', desc: 'Frame the spike as a single, answerable question. The tighter the question, the better the answer.' },
                      { icon: Beaker, title: 'Agent investigates', desc: 'The agent examines a reference project, searches for alternatives, and builds a working prototype in a temp workspace.' },
                      { icon: Scale, title: 'Verdict rendered', desc: 'Every spike ends with ADOPT (proven, ready to build), ADAPT (needs refinement), or REJECT (not viable).' },
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

                {/* Verdict strip */}
                <div className="space-y-3">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Verdicts</h3>
                  <div className="flex items-center gap-2 flex-wrap text-xs font-semibold">
                    <span className="px-2.5 py-1 rounded-full bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300">ADOPT</span>
                    <span className="px-2.5 py-1 rounded-full bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300">ADAPT</span>
                    <span className="px-2.5 py-1 rounded-full bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300">REJECT</span>
                  </div>
                  <p className="text-xs text-muted-foreground/60 leading-relaxed">
                    <strong className="text-muted-foreground">Adopt</strong> feeds into hypothetical planning.{' '}
                    <strong className="text-muted-foreground">Adapt</strong> promotes to Ponder for refinement.{' '}
                    <strong className="text-muted-foreground">Reject</strong> stores findings in the knowledge base.
                  </p>
                </div>

                {/* CTA */}
                <div className="text-center pt-2">
                  <div className="inline-flex items-center gap-1 bg-muted/60 border border-border rounded-lg px-3 py-2 text-xs font-mono text-muted-foreground">
                    /sdlc-spike &lt;slug&gt; — &lt;the question to answer&gt;
                  </div>
                  <p className="text-xs text-muted-foreground/60 mt-2">
                    Start a spike from the CLI or an agent session.
                  </p>
                </div>

                {spikes.length > 0 && (
                  <p className="text-center text-xs text-muted-foreground/40">
                    Or select a spike from the list to view its findings.
                  </p>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
