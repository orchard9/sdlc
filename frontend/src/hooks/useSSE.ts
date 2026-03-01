import { useEffect, useRef } from 'react'
import { useSseContext } from '@/contexts/SseContext'
import type { AdvisorySseEvent, DocsSseEvent, InvestigationSseEvent, PonderSseEvent, RunSseEvent } from '@/lib/types'

/** Subscribe to /api/events and call handlers whenever events arrive.
 *
 *  The single SSE connection is owned by SseContext at the app root. This hook
 *  registers with that context on mount and deregisters on unmount — no connection
 *  is opened per call site.
 *
 *  Callbacks are held in refs so callers do not need to wrap them in useCallback.
 *  The subscription is established once on mount and never restarted due to callback
 *  identity changes.
 *
 *  The update debounce (500ms) is shared across all subscribers — all onUpdate
 *  callbacks fire together on the same tick.
 */
export function useSSE(
  onUpdate: () => void,
  onPonderEvent?: (event: PonderSseEvent) => void,
  onRunEvent?: (event: RunSseEvent) => void,
  onInvestigationEvent?: (event: InvestigationSseEvent) => void,
  onDocsEvent?: (event: DocsSseEvent) => void,
  onAdvisoryEvent?: (event: AdvisorySseEvent) => void,
) {
  const { subscribe } = useSseContext()

  const onUpdateRef = useRef(onUpdate)
  const onPonderRef = useRef(onPonderEvent)
  const onRunRef = useRef(onRunEvent)
  const onInvestigationRef = useRef(onInvestigationEvent)
  const onDocsRef = useRef(onDocsEvent)
  const onAdvisoryRef = useRef(onAdvisoryEvent)

  // Keep refs current on every render without triggering the subscription effect
  useEffect(() => {
    onUpdateRef.current = onUpdate
    onPonderRef.current = onPonderEvent
    onRunRef.current = onRunEvent
    onInvestigationRef.current = onInvestigationEvent
    onDocsRef.current = onDocsEvent
    onAdvisoryRef.current = onAdvisoryEvent
  })

  // Register with the shared SSE context — runs once on mount, cleans up on unmount.
  // subscribe is stable (useCallback [] in SseContext), so this effect never re-runs.
  useEffect(() => {
    return subscribe({
      onUpdate: () => onUpdateRef.current(),
      onPonderEvent: (e) => onPonderRef.current?.(e),
      onRunEvent: (e) => onRunRef.current?.(e),
      onInvestigationEvent: (e) => onInvestigationRef.current?.(e),
      onDocsEvent: (e) => onDocsRef.current?.(e),
      onAdvisoryEvent: (e) => onAdvisoryRef.current?.(e),
    })
  }, [subscribe])
}
