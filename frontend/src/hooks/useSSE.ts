import { useEffect } from 'react'
import type { InvestigationSseEvent, PonderSseEvent, RunSseEvent } from '@/lib/types'

/** Subscribe to /api/events and call onUpdate whenever state changes.
 *  Rapid updates are debounced (500ms) to prevent connection saturation
 *  during agent runs.
 *
 *  Optionally pass onPonderEvent to receive typed ponder run lifecycle events.
 *  Ponder events are NOT debounced — they are structural signals (run started,
 *  completed, stopped) and must arrive promptly to update UI lock state.
 *
 *  Optionally pass onRunEvent to receive agent run lifecycle events.
 *  Run events are NOT debounced — they are structural signals.
 *
 *  Optionally pass onInvestigationEvent to receive investigation run lifecycle events.
 *  Investigation events are NOT debounced — they are structural signals.
 */
export function useSSE(
  onUpdate: () => void,
  onPonderEvent?: (event: PonderSseEvent) => void,
  onRunEvent?: (event: RunSseEvent) => void,
  onInvestigationEvent?: (event: InvestigationSseEvent) => void,
) {
  useEffect(() => {
    const es = new EventSource('/api/events')
    let timer: ReturnType<typeof setTimeout> | null = null

    es.addEventListener('update', () => {
      if (timer) clearTimeout(timer)
      timer = setTimeout(onUpdate, 500)
    })

    if (onPonderEvent) {
      es.addEventListener('ponder', (e: MessageEvent) => {
        try {
          const data = JSON.parse(e.data) as PonderSseEvent
          onPonderEvent(data)
        } catch {
          // malformed event — ignore
        }
      })
    }

    if (onRunEvent) {
      es.addEventListener('run', (e: MessageEvent) => {
        try {
          const data = JSON.parse(e.data) as RunSseEvent
          onRunEvent(data)
        } catch {
          // malformed event — ignore
        }
      })
    }

    if (onInvestigationEvent) {
      es.addEventListener('investigation', (e: MessageEvent) => {
        try {
          const data = JSON.parse(e.data) as InvestigationSseEvent
          onInvestigationEvent(data)
        } catch {
          // malformed event — ignore
        }
      })
    }

    es.onerror = () => {} // browser auto-reconnects
    return () => {
      if (timer) clearTimeout(timer)
      es.close()
    }
  }, [onUpdate, onPonderEvent, onRunEvent, onInvestigationEvent])
}
