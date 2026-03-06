import { useState } from 'react'
import { ChevronDown, ChevronRight, Bot, Loader2, CheckCircle2, XCircle } from 'lucide-react'
import type { PairedSubagentExchange } from '@/lib/types'

interface SubagentCardProps {
  event: PairedSubagentExchange
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

function StatusBadge({ status, isComplete }: { status?: string; isComplete: boolean }) {
  if (!isComplete) {
    return (
      <span className="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20">
        <Loader2 className="w-2.5 h-2.5 animate-spin" />
        running
      </span>
    )
  }

  const isFailed = status === 'failed' || status === 'error'

  if (isFailed) {
    return (
      <span className="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 border border-red-500/20">
        <XCircle className="w-2.5 h-2.5" />
        failed
      </span>
    )
  }

  return (
    <span className="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
      <CheckCircle2 className="w-2.5 h-2.5" />
      completed
    </span>
  )
}

export function SubagentCard({ event }: SubagentCardProps) {
  const [expanded, setExpanded] = useState(false)
  const hasDetails = event.lastToolName || event.totalTokens != null || event.durationMs != null

  return (
    <div className="border-l-2 border-indigo-500 pl-3 py-1 space-y-1">
      {/* Header row */}
      <div className="flex items-center gap-2 flex-wrap">
        <Bot className="w-3 h-3 text-indigo-400 shrink-0" />
        <span className="text-xs font-medium text-indigo-300">
          {event.description || 'Subagent'}
        </span>
        <StatusBadge status={event.status} isComplete={event.isComplete} />
        {event.durationMs != null && (
          <span className="text-[10px] text-muted-foreground/60">
            {formatDuration(event.durationMs)}
          </span>
        )}
        {hasDetails && (
          <button
            onClick={() => setExpanded(v => !v)}
            className="flex items-center gap-0.5 text-[10px] text-muted-foreground hover:text-foreground transition-colors ml-auto"
          >
            {expanded
              ? <ChevronDown className="w-3 h-3" />
              : <ChevronRight className="w-3 h-3" />
            }
            {expanded ? 'hide details' : 'details'}
          </button>
        )}
      </div>

      {/* Collapsible details */}
      {expanded && hasDetails && (
        <div className="text-[10px] text-muted-foreground bg-muted/40 border border-border/40 rounded px-2 py-1.5 space-y-0.5">
          {event.lastToolName && (
            <p>Last tool: <span className="font-mono text-foreground/70">{event.lastToolName}</span></p>
          )}
          {event.totalTokens != null && (
            <p>Tokens: <span className="text-foreground/70">{formatTokens(event.totalTokens)}</span></p>
          )}
          {event.durationMs != null && (
            <p>Duration: <span className="text-foreground/70">{formatDuration(event.durationMs)}</span></p>
          )}
        </div>
      )}

      {/* Summary text */}
      {event.summary && (
        <p className="text-[10px] text-muted-foreground/70 pl-0.5 line-clamp-3">
          {event.summary}
        </p>
      )}
    </div>
  )
}
