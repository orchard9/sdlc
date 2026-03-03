import { Link } from 'react-router-dom'
import { AlertTriangle, ArrowRight, Check, Clock, ClipboardList, Layers, Rocket, X } from 'lucide-react'
import { Skeleton } from '@/components/shared/Skeleton'
import { useChangelog } from '@/hooks/useChangelog'
import type { ChangeEvent, EventKind } from '@/hooks/useChangelog'

// ---------------------------------------------------------------------------
// Relative time helper — no external dependency
// ---------------------------------------------------------------------------

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`
  const days = Math.floor(hours / 24)
  return `${days} day${days === 1 ? '' : 's'} ago`
}

// ---------------------------------------------------------------------------
// Event sorting: run_failed first (desc timestamp), then rest (desc timestamp)
// ---------------------------------------------------------------------------

function sortEvents(events: ChangeEvent[]): ChangeEvent[] {
  const failed = events
    .filter(e => e.kind === 'run_failed')
    .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
  const rest = events
    .filter(e => e.kind !== 'run_failed')
    .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
  return [...failed, ...rest]
}

// ---------------------------------------------------------------------------
// Event icon
// ---------------------------------------------------------------------------

function EventIcon({ kind }: { kind: EventKind }) {
  switch (kind) {
    case 'run_failed':
      return <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
    case 'feature_merged':
      return <Rocket className="w-3.5 h-3.5 text-green-400 shrink-0 mt-0.5" />
    case 'review_approved':
    case 'audit_approved':
    case 'qa_approved':
      return <Check className="w-3.5 h-3.5 text-primary shrink-0 mt-0.5" />
    case 'feature_phase_advanced':
      return <ArrowRight className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
    case 'milestone_wave_completed':
      return <Layers className="w-3.5 h-3.5 text-blue-400 shrink-0 mt-0.5" />
    default:
      return <ArrowRight className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
  }
}

// ---------------------------------------------------------------------------
// Event row
// ---------------------------------------------------------------------------

function EventRow({ event }: { event: ChangeEvent }) {
  const link = event.kind === 'run_failed'
    ? '/runs'
    : event.kind === 'feature_merged'
      ? `/features/${event.slug}`
      : null

  const inner = (
    <div className="flex items-start gap-2 py-1 min-w-0">
      <EventIcon kind={event.kind} />
      <span className="font-mono text-muted-foreground bg-muted/70 px-1.5 py-0.5 rounded text-[10px] shrink-0">
        {event.kind}
      </span>
      <span className="text-xs font-mono text-muted-foreground shrink-0 hidden sm:inline">·</span>
      <span className="text-xs font-mono text-muted-foreground shrink-0 hidden sm:inline">{event.slug}</span>
      <span className="text-xs text-muted-foreground truncate">{event.title}</span>
    </div>
  )

  if (link) {
    return (
      <Link
        to={link}
        className="block hover:bg-accent/40 rounded -mx-1 px-1 transition-colors"
      >
        {inner}
      </Link>
    )
  }

  return <div className="rounded -mx-1 px-1">{inner}</div>
}

// ---------------------------------------------------------------------------
// WhatChangedBanner
// ---------------------------------------------------------------------------

export function WhatChangedBanner() {
  const { events, total, lastVisitAt, loading, dismissed, dismiss } = useChangelog()

  if (loading) {
    return (
      <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6">
        <Skeleton width="w-48" className="h-4" />
      </div>
    )
  }

  if (dismissed || events.length === 0) {
    return null
  }

  const sorted = sortEvents(events)
  const isFirstVisit = lastVisitAt === null

  return (
    <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6">
      {/* Header */}
      <div className="flex items-center justify-between gap-3 mb-2">
        <div className="flex items-center gap-2">
          {isFirstVisit ? (
            <ClipboardList className="w-4 h-4 text-primary shrink-0" />
          ) : (
            <Clock className="w-4 h-4 text-primary shrink-0" />
          )}
          <span className="text-sm font-semibold">
            {isFirstVisit
              ? 'Recent project activity'
              : `${total} change${total === 1 ? '' : 's'} since ${relativeTime(lastVisitAt)}`}
          </span>
        </div>
        <button
          onClick={dismiss}
          className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors shrink-0"
          aria-label="Dismiss changelog banner"
        >
          <X className="w-3 h-3" />
          Dismiss
        </button>
      </div>

      {/* Event list */}
      <div className="space-y-0.5 max-h-48 overflow-y-auto">
        {sorted.map(event => (
          <EventRow key={event.id} event={event} />
        ))}
      </div>
    </div>
  )
}
