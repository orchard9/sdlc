import { useState } from 'react'
import { ChevronDown, ChevronRight, AlertCircle, Wrench, Clock } from 'lucide-react'
import type { PairedToolExchange } from '@/lib/types'

interface ToolCallCardProps {
  event: PairedToolExchange
}

export function ToolCallCard({ event }: ToolCallCardProps) {
  const [expanded, setExpanded] = useState(false)
  const hasInput = event.input != null && Object.keys(event.input as object).length > 0

  return (
    <div className="border-l-2 border-blue-500 pl-3 py-1 space-y-1">
      {/* Header row */}
      <div className="flex items-center gap-2 flex-wrap">
        <Wrench className="w-3 h-3 text-blue-400 shrink-0" />
        <span className="text-xs font-medium font-mono text-blue-300">{event.toolName}</span>
        {event.isError && (
          <span className="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 border border-red-500/20">
            <AlertCircle className="w-2.5 h-2.5" />
            error
          </span>
        )}
        {event.elapsed_seconds != null && (
          <span className="flex items-center gap-1 text-[10px] text-muted-foreground/60">
            <Clock className="w-2.5 h-2.5" />
            {event.elapsed_seconds.toFixed(1)}s
          </span>
        )}
        {hasInput && (
          <button
            onClick={() => setExpanded(v => !v)}
            className="flex items-center gap-0.5 text-[10px] text-muted-foreground hover:text-foreground transition-colors ml-auto"
          >
            {expanded
              ? <ChevronDown className="w-3 h-3" />
              : <ChevronRight className="w-3 h-3" />
            }
            {expanded ? 'hide input' : 'show input'}
          </button>
        )}
      </div>

      {/* Collapsible input JSON */}
      {expanded && hasInput && (
        <pre className="text-[10px] font-mono text-muted-foreground bg-muted/40 border border-border/40 rounded px-2 py-1.5 overflow-x-auto max-h-48 whitespace-pre-wrap">
          {JSON.stringify(event.input, null, 2)}
        </pre>
      )}

      {/* Summary / result text */}
      {event.summary && (
        <p className="text-[10px] text-muted-foreground/70 pl-0.5 line-clamp-3">
          {event.summary}
        </p>
      )}
    </div>
  )
}
