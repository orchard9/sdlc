import { useEffect, useRef } from 'react'
import { Loader2 } from 'lucide-react'
import { AgentEventLine } from './AgentEventLine'
import type { AgentEvent } from '@/lib/types'

interface AgentLogProps {
  running: boolean
  events: AgentEvent[]
}

export function AgentLog({ running, events }: AgentLogProps) {
  const logRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight
    }
  }, [events])

  if (!running && events.length === 0) return null

  return (
    <div
      ref={logRef}
      className="bg-muted/30 border border-border/50 rounded-lg p-3 max-h-72 overflow-y-auto font-mono text-xs space-y-1"
    >
      {running && events.length === 0 && (
        <div className="flex items-center gap-2 text-muted-foreground">
          <Loader2 className="w-3 h-3 animate-spin" />
          Spawning agent...
        </div>
      )}
      {events.map((event, i) => (
        <AgentEventLine key={i} event={event} />
      ))}
    </div>
  )
}
