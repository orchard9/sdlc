import { useState } from 'react'
import { ChevronRight, ChevronDown } from 'lucide-react'

interface Props {
  tool: string
  summary: string
}

export function ToolCallBlock({ tool, summary }: Props) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="my-1">
      <button
        onClick={() => setExpanded(v => !v)}
        className="flex items-start gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors group w-full text-left"
      >
        {expanded
          ? <ChevronDown className="w-3.5 h-3.5 mt-0.5 shrink-0 text-muted-foreground/60" />
          : <ChevronRight className="w-3.5 h-3.5 mt-0.5 shrink-0 text-muted-foreground/60" />}
        <span className="font-mono text-muted-foreground/50 group-hover:text-muted-foreground transition-colors">
          [tool]
        </span>
        <span className="font-medium truncate">{tool}</span>
      </button>
      {expanded && summary && (
        <div className="ml-5 mt-1.5 pl-3 border-l border-border/40 text-xs text-muted-foreground/80 whitespace-pre-wrap font-mono leading-relaxed">
          {summary}
        </div>
      )}
    </div>
  )
}
