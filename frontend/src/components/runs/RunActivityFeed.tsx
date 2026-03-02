import { useMemo } from 'react'
import { Loader2, AlertCircle } from 'lucide-react'
import { useRunTelemetry } from '@/hooks/useRunTelemetry'
import { pairEvents } from './pairEvents'
import { RunInitCard } from './RunInitCard'
import { ToolCallCard } from './ToolCallCard'
import { AssistantTextBlock } from './AssistantTextBlock'
import { RunResultCard } from './RunResultCard'
import type { PairedEvent, RawRunEvent } from '@/lib/types'

interface RunActivityFeedProps {
  runId: string
  isRunning: boolean
  /** If provided, skip the internal useRunTelemetry call and use these events directly */
  events?: RawRunEvent[]
  /** Prompt for the run (used in the init card), forwarded from caller when events are provided */
  prompt?: string | null
}

function PairedEventRow({ event }: { event: PairedEvent }) {
  switch (event.kind) {
    case 'init':
      return <RunInitCard event={event} />
    case 'tool_exchange':
      return <ToolCallCard event={event} />
    case 'assistant_text':
      return <AssistantTextBlock event={event} />
    case 'run_result':
      return <RunResultCard event={event} />
    default:
      return null
  }
}

export function RunActivityFeed({ runId, isRunning, events: eventsProp, prompt: promptProp }: RunActivityFeedProps) {
  // If caller provides events directly, skip the internal fetch
  const skipFetch = eventsProp != null
  const { telemetry, isLoading, error } = useRunTelemetry(skipFetch ? '' : runId, isRunning)

  const pairedEvents = useMemo(() => {
    if (skipFetch) {
      return pairEvents(eventsProp ?? [], promptProp ?? null)
    }
    if (!telemetry) return []
    return pairEvents(telemetry.events, telemetry.prompt)
  }, [skipFetch, eventsProp, promptProp, telemetry])

  if (isLoading && !telemetry) {
    return (
      <div className="flex items-center gap-2 text-muted-foreground text-xs py-4 pl-2">
        <Loader2 className="w-3.5 h-3.5 animate-spin" />
        Loading activity…
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center gap-2 text-destructive text-xs py-2 pl-2">
        <AlertCircle className="w-3.5 h-3.5 shrink-0" />
        {error}
      </div>
    )
  }

  if (pairedEvents.length === 0) {
    return (
      <p className="text-xs text-muted-foreground/60 py-4 pl-2 italic">
        No activity recorded yet.
      </p>
    )
  }

  return (
    <div className="space-y-2 py-1">
      {pairedEvents.map((event, i) => (
        <PairedEventRow key={i} event={event} />
      ))}
      {isRunning && (
        <div className="flex items-center gap-2 text-muted-foreground text-xs py-1 pl-2">
          <Loader2 className="w-3 h-3 animate-spin" />
          <span className="text-muted-foreground/60">Running…</span>
        </div>
      )}
    </div>
  )
}
