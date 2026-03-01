import { useEffect, useRef } from 'react'
import type { AdvisorySseEvent, DocsSseEvent, InvestigationSseEvent, PonderSseEvent, RunSseEvent } from '@/lib/types'

/** Subscribe to /api/events and call onUpdate whenever state changes.
 *  Uses POST (not EventSource GET) so that SSE works through Cloudflare Quick
 *  Tunnels, which intentionally buffer GET streaming responses. POST streaming
 *  bypasses that guardrail.
 *
 *  Rapid updates are debounced (500ms) to prevent connection saturation
 *  during agent runs.
 *
 *  Optionally pass onPonderEvent / onRunEvent / onInvestigationEvent to receive
 *  typed lifecycle events. These are NOT debounced.
 *
 *  Callbacks are held in refs so the connection is established once on mount
 *  and never restarted due to callback identity changes. Callers do not need
 *  to wrap callbacks in useCallback.
 */
export function useSSE(
  onUpdate: () => void,
  onPonderEvent?: (event: PonderSseEvent) => void,
  onRunEvent?: (event: RunSseEvent) => void,
  onInvestigationEvent?: (event: InvestigationSseEvent) => void,
  onDocsEvent?: (event: DocsSseEvent) => void,
  onAdvisoryEvent?: (event: AdvisorySseEvent) => void,
) {
  const onUpdateRef = useRef(onUpdate)
  const onPonderRef = useRef(onPonderEvent)
  const onRunRef = useRef(onRunEvent)
  const onInvestigationRef = useRef(onInvestigationEvent)
  const onDocsRef = useRef(onDocsEvent)
  const onAdvisoryRef = useRef(onAdvisoryEvent)

  // Keep refs current on every render without triggering the effect
  useEffect(() => {
    onUpdateRef.current = onUpdate
    onPonderRef.current = onPonderEvent
    onRunRef.current = onRunEvent
    onInvestigationRef.current = onInvestigationEvent
    onDocsRef.current = onDocsEvent
    onAdvisoryRef.current = onAdvisoryEvent
  })

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout> | null = null
    let active = true

    function dispatch(type: string, data: string) {
      if (type === 'update') {
        if (timer) clearTimeout(timer)
        timer = setTimeout(() => onUpdateRef.current(), 500)
      } else if (type === 'ponder' && onPonderRef.current) {
        try { onPonderRef.current(JSON.parse(data) as PonderSseEvent) } catch { /* malformed */ }
      } else if (type === 'run' && onRunRef.current) {
        try { onRunRef.current(JSON.parse(data) as RunSseEvent) } catch { /* malformed */ }
      } else if (type === 'investigation' && onInvestigationRef.current) {
        try { onInvestigationRef.current(JSON.parse(data) as InvestigationSseEvent) } catch { /* malformed */ }
      } else if (type === 'docs' && onDocsRef.current) {
        try { onDocsRef.current(JSON.parse(data) as DocsSseEvent) } catch { /* malformed */ }
      } else if (type === 'advisory' && onAdvisoryRef.current) {
        try { onAdvisoryRef.current(JSON.parse(data) as AdvisorySseEvent) } catch { /* malformed */ }
      }
    }

    async function connect() {
      while (active) {
        try {
          const response = await fetch('/api/events', {
            method: 'POST',
            signal: controller.signal,
            headers: { Accept: 'text/event-stream' },
          })

          if (!response.ok || !response.body) {
            await new Promise(r => setTimeout(r, 3000))
            continue
          }

          const reader = response.body.getReader()
          const decoder = new TextDecoder()
          let buffer = ''
          let currentType = 'message'
          let currentData = ''

          while (active) {
            const { done, value } = await reader.read()
            if (done) {
              // Server closed the stream cleanly — wait before reconnecting
              // to avoid a tight reconnect loop.
              await new Promise(r => setTimeout(r, 2000))
              break
            }

            buffer += decoder.decode(value, { stream: true })
            const lines = buffer.split('\n')
            buffer = lines.pop()! // last partial line goes back to buffer

            for (const line of lines) {
              if (line.startsWith('event:')) {
                currentType = line.slice(6).trim()
              } else if (line.startsWith('data:')) {
                currentData = line.slice(5).trim()
              } else if (line === '' || line === '\r') {
                // blank line = end of event
                if (currentData) dispatch(currentType, currentData)
                currentType = 'message'
                currentData = ''
              }
              // lines starting with ':' are comments (e.g. padding) — ignored
            }
          }
        } catch {
          if (!active) break
          await new Promise(r => setTimeout(r, 3000))
        }
      }
    }

    connect()

    return () => {
      active = false
      if (timer) clearTimeout(timer)
      controller.abort()
    }
  }, []) // empty deps — connect once on mount, never restart due to callback changes
}
