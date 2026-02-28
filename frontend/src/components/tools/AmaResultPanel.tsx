import { useState } from 'react'
import { ChevronDown, ChevronRight, AlertTriangle } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { AmaData, AmaSource } from '@/lib/types'

interface AmaSourceCardProps {
  source: AmaSource
  index: number
}

function AmaSourceCard({ source, index }: AmaSourceCardProps) {
  const [expanded, setExpanded] = useState(index === 0)

  const locationLabel = `${source.path}:${source.lines[0]}–${source.lines[1]}`
  const scorePercent = Math.round(source.score * 100)

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(prev => !prev)}
        className="w-full flex items-center gap-2 px-3 py-2 bg-muted/30 hover:bg-muted/50 transition-colors text-left"
      >
        {expanded
          ? <ChevronDown className="w-3.5 h-3.5 shrink-0 text-muted-foreground" />
          : <ChevronRight className="w-3.5 h-3.5 shrink-0 text-muted-foreground" />
        }
        <span className="font-mono text-xs text-foreground truncate flex-1">{locationLabel}</span>
        <span className={cn(
          'text-[10px] font-mono px-1.5 py-0.5 rounded border shrink-0',
          scorePercent >= 80
            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
            : scorePercent >= 50
              ? 'bg-amber-500/10 border-amber-500/30 text-amber-400'
              : 'bg-muted border-border text-muted-foreground',
        )}>
          {scorePercent}%
        </span>
        {source.stale && (
          <span className="flex items-center gap-1 text-[10px] text-amber-400 shrink-0">
            <AlertTriangle className="w-3 h-3" />
            stale
          </span>
        )}
      </button>
      {expanded && (
        <pre className="px-3 py-2 text-xs font-mono bg-card text-muted-foreground overflow-x-auto whitespace-pre-wrap leading-relaxed">
          {source.excerpt}
        </pre>
      )}
    </div>
  )
}

interface AmaResultPanelProps {
  data: AmaData
}

export function AmaResultPanel({ data }: AmaResultPanelProps) {
  if (data.sources.length === 0) {
    return (
      <p className="text-sm text-muted-foreground italic">No relevant sources found. Try re-running setup or rephrasing your question.</p>
    )
  }

  return (
    <div className="space-y-2">
      <p className="text-xs text-muted-foreground mb-3">
        {data.sources.length} {data.sources.length === 1 ? 'source' : 'sources'} — click to expand code excerpts
      </p>
      {data.sources.map((source, i) => (
        <AmaSourceCard key={`${source.path}-${i}`} source={source} index={i} />
      ))}
    </div>
  )
}
