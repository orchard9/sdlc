import { useCallback, useEffect, useRef, useState } from 'react'
import { Loader2, CheckCircle2, XCircle, StopCircle, ChevronDown, ChevronRight } from 'lucide-react'
import { AgentLog } from '@/components/shared/AgentLog'
import { api } from '@/api/client'
import type { AgentEvent, RunRecord } from '@/lib/types'

interface RunCardProps {
  run: RunRecord
  expanded: boolean
  onToggle: () => void
}

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case 'running':
      return <Loader2 className="w-4 h-4 text-primary animate-spin shrink-0" />
    case 'completed':
      return <CheckCircle2 className="w-4 h-4 text-green-400 shrink-0" />
    case 'failed':
      return <XCircle className="w-4 h-4 text-red-400 shrink-0" />
    case 'stopped':
      return <StopCircle className="w-4 h-4 text-muted-foreground shrink-0" />
    default:
      return null
  }
}

function formatTime(iso: string) {
  try {
    return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

export function RunCard({ run, expanded, onToggle }: RunCardProps) {
  const [events, setEvents] = useState<AgentEvent[]>([])
  const [loadedEvents, setLoadedEvents] = useState(false)
  const eventSourceRef = useRef<EventSource | null>(null)

  // For active runs: open EventSource when expanded
  useEffect(() => {
    if (!expanded || run.status !== 'running') {
      eventSourceRef.current?.close()
      eventSourceRef.current = null
      return
    }

    // Determine the events URL based on run type
    let eventsUrl: string
    if (run.run_type === 'milestone_uat') {
      eventsUrl = `/api/milestone/${encodeURIComponent(run.target)}/uat/events`
    } else if (run.run_type === 'milestone_prepare') {
      eventsUrl = `/api/milestone/${encodeURIComponent(run.target)}/prepare/events`
    } else if (run.run_type === 'ponder') {
      // Ponder runs don't have event streaming
      return
    } else {
      eventsUrl = `/api/run/${encodeURIComponent(run.key)}/events`
    }

    const es = new EventSource(eventsUrl)
    eventSourceRef.current = es

    es.addEventListener('agent', (e) => {
      try {
        const event = JSON.parse(e.data) as AgentEvent
        setEvents(prev => [...prev, event])
      } catch {
        // ignore parse errors
      }
    })

    es.onerror = () => {
      es.close()
      eventSourceRef.current = null
    }

    return () => {
      es.close()
      eventSourceRef.current = null
    }
  }, [expanded, run.status, run.run_type, run.target, run.key])

  // For completed runs: fetch events on-demand when expanded
  useEffect(() => {
    if (!expanded || run.status === 'running' || loadedEvents) return

    api.getRun(run.id)
      .then(data => {
        setEvents(data.events ?? [])
        setLoadedEvents(true)
      })
      .catch(() => {})
  }, [expanded, run.status, run.id, loadedEvents])

  // Reset events when collapsing
  const handleToggle = useCallback(() => {
    if (expanded) {
      setEvents([])
      setLoadedEvents(false)
    }
    onToggle()
  }, [expanded, onToggle])

  const isActive = run.status === 'running'
  const isPonder = run.run_type === 'ponder'

  return (
    <div className={`rounded-lg border ${isActive ? 'border-primary/30 bg-primary/5' : 'border-border/50 bg-card/50'}`}>
      <button
        onClick={handleToggle}
        className="w-full flex items-start gap-2 px-3 py-2 text-left hover:bg-muted/30 transition-colors rounded-lg"
      >
        <StatusIcon status={run.status} />
        <div className="min-w-0 flex-1">
          <p className="text-xs font-medium truncate">{run.label}</p>
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground mt-0.5">
            <span>{formatTime(run.started_at)}</span>
            {run.cost_usd != null && <span>· ${run.cost_usd.toFixed(2)}</span>}
            {run.turns != null && <span>· {run.turns} turns</span>}
            {run.error && <span className="text-red-400 truncate">· {run.error.slice(0, 40)}</span>}
          </div>
        </div>
        {!isPonder && (
          expanded
            ? <ChevronDown className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
            : <ChevronRight className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
        )}
      </button>

      {expanded && !isPonder && (
        <div className="px-3 pb-3">
          <AgentLog running={isActive} events={events} />
        </div>
      )}
    </div>
  )
}
