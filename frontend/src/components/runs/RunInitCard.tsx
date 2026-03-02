import type { PairedInitEvent } from '@/lib/types'
import { Bot } from 'lucide-react'

interface RunInitCardProps {
  event: PairedInitEvent
}

export function RunInitCard({ event }: RunInitCardProps) {
  const { event: raw, prompt } = event
  const mcpList = raw.mcp_servers?.filter(Boolean) ?? []

  return (
    <div className="rounded-lg border border-border/50 bg-card/60 p-3 space-y-2">
      <div className="flex items-center gap-2">
        <Bot className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <span className="text-xs font-medium text-muted-foreground">Run started</span>
        {raw.model && (
          <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20">
            {raw.model}
          </span>
        )}
        {raw.tools_count != null && (
          <span className="text-[10px] text-muted-foreground/60">
            {raw.tools_count} tool{raw.tools_count !== 1 ? 's' : ''}
          </span>
        )}
        {mcpList.length > 0 && (
          <span className="text-[10px] text-muted-foreground/60">
            · MCP: {mcpList.join(', ')}
          </span>
        )}
      </div>
      {prompt && (
        <div className="pl-5">
          <p className="text-xs text-muted-foreground/80 whitespace-pre-wrap line-clamp-6 leading-relaxed">
            {prompt}
          </p>
        </div>
      )}
    </div>
  )
}
