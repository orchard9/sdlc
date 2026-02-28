import { cn } from '@/lib/utils'
import { Wrench, AlertTriangle } from 'lucide-react'
import type { ToolMeta } from '@/lib/types'

interface ToolCardProps {
  tool: ToolMeta
  selected: boolean
  onSelect: () => void
}

export function ToolCard({ tool, selected, onSelect }: ToolCardProps) {
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
        <Wrench className="w-3.5 h-3.5 shrink-0 text-muted-foreground" />
        <span className="text-sm font-mono font-medium truncate">{tool.name}</span>
        <span className="ml-auto shrink-0 text-[10px] font-mono bg-muted/60 border border-border/50 rounded px-1.5 py-0.5 text-muted-foreground">
          v{tool.version}
        </span>
      </div>
      <p className="text-xs text-muted-foreground line-clamp-2 pl-5">{tool.description}</p>
      {tool.requires_setup && (
        <div className="flex items-center gap-1 mt-1 pl-5">
          <AlertTriangle className="w-3 h-3 text-amber-400 shrink-0" />
          <span className="text-[10px] text-amber-400/80">setup required</span>
        </div>
      )}
    </button>
  )
}
