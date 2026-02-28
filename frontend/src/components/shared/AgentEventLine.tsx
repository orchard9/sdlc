import type { AgentEvent } from '@/lib/types'

interface AgentEventLineProps {
  event: AgentEvent
}

export function AgentEventLine({ event }: AgentEventLineProps) {
  switch (event.type) {
    case 'init':
      return <div className="text-blue-400">Agent started — {event.model} ({event.mcp_servers?.join(', ')})</div>
    case 'assistant':
      return (
        <div>
          {event.text && <div className="text-foreground whitespace-pre-wrap">{event.text}</div>}
          {event.tools?.map((t, i) => (
            <div key={i} className="text-yellow-400">
              {'\u2192'} {t.name}
            </div>
          ))}
        </div>
      )
    case 'tool_progress':
      return (
        <div className="text-muted-foreground">
          {event.tool} ({event.elapsed_seconds?.toFixed(1)}s)
        </div>
      )
    case 'tool_summary':
      return <div className="text-muted-foreground">{event.summary}</div>
    case 'result':
      return (
        <div className={event.is_error ? 'text-red-400' : 'text-green-400'}>
          {event.is_error ? 'Failed' : 'Done'}
          {event.text ? ` — ${event.text.slice(0, 200)}` : ''}
          {event.cost_usd != null && (
            <span className="text-muted-foreground ml-2">
              (${event.cost_usd.toFixed(4)}, {event.turns} turns)
            </span>
          )}
        </div>
      )
    case 'error':
      return <div className="text-red-400">Error: {event.message}</div>
    case 'status':
      return <div className="text-muted-foreground">{event.status}</div>
    default:
      return null
  }
}
