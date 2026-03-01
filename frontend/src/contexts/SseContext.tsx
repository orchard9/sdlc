import { createContext, useCallback, useContext, useEffect, useRef, type ReactNode } from 'react'
import type { AdvisorySseEvent, DocsSseEvent, InvestigationSseEvent, PonderSseEvent, RunSseEvent } from '@/lib/types'

export interface SseCallbacks {
  onUpdate?: () => void
  onPonderEvent?: (event: PonderSseEvent) => void
  onRunEvent?: (event: RunSseEvent) => void
  onInvestigationEvent?: (event: InvestigationSseEvent) => void
  onDocsEvent?: (event: DocsSseEvent) => void
  onAdvisoryEvent?: (event: AdvisorySseEvent) => void
}

interface SseContextValue {
  subscribe: (callbacks: SseCallbacks) => () => void
}

const SseContext = createContext<SseContextValue | null>(null)

export function SseProvider({ children }: { children: ReactNode }) {
  // Subscriber registry — stored in a ref so subscribe/unsubscribe never trigger re-renders
  const subscribersRef = useRef<Set<SseCallbacks>>(new Set())

  // Single SSE connection — started once on mount, never restarted due to subscriber changes
  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout> | null = null
    let active = true

    function dispatch(type: string, data: string) {
      // Snapshot before iterating — safe if a subscriber unregisters during dispatch
      const subs = Array.from(subscribersRef.current)

      if (type === 'update') {
        // Shared debounce: one timer fires all onUpdate subscribers together
        if (timer) clearTimeout(timer)
        timer = setTimeout(() => {
          // Re-snapshot at fire time — subscriber list may have changed during 500ms wait
          for (const sub of Array.from(subscribersRef.current)) {
            sub.onUpdate?.()
          }
        }, 500)
      } else if (type === 'ponder') {
        try {
          const event = JSON.parse(data) as PonderSseEvent
          for (const sub of subs) sub.onPonderEvent?.(event)
        } catch { /* malformed */ }
      } else if (type === 'run') {
        try {
          const event = JSON.parse(data) as RunSseEvent
          for (const sub of subs) sub.onRunEvent?.(event)
        } catch { /* malformed */ }
      } else if (type === 'investigation') {
        try {
          const event = JSON.parse(data) as InvestigationSseEvent
          for (const sub of subs) sub.onInvestigationEvent?.(event)
        } catch { /* malformed */ }
      } else if (type === 'docs') {
        try {
          const event = JSON.parse(data) as DocsSseEvent
          for (const sub of subs) sub.onDocsEvent?.(event)
        } catch { /* malformed */ }
      } else if (type === 'advisory') {
        try {
          const event = JSON.parse(data) as AdvisorySseEvent
          for (const sub of subs) sub.onAdvisoryEvent?.(event)
        } catch { /* malformed */ }
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
              await new Promise(r => setTimeout(r, 2000))
              break
            }

            buffer += decoder.decode(value, { stream: true })
            const lines = buffer.split('\n')
            buffer = lines.pop()!

            for (const line of lines) {
              if (line.startsWith('event:')) {
                currentType = line.slice(6).trim()
              } else if (line.startsWith('data:')) {
                currentData = line.slice(5).trim()
              } else if (line === '' || line === '\r') {
                if (currentData) dispatch(currentType, currentData)
                currentType = 'message'
                currentData = ''
              }
              // lines starting with ':' are comments (padding) — ignored
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
  }, []) // empty deps — single connection for the provider's lifetime

  // Stable subscribe function — memoized with [] so useSSE's effect never re-runs due to identity change
  const subscribe = useCallback((callbacks: SseCallbacks): (() => void) => {
    subscribersRef.current.add(callbacks)
    return () => { subscribersRef.current.delete(callbacks) }
  }, [])

  return (
    <SseContext.Provider value={{ subscribe }}>
      {children}
    </SseContext.Provider>
  )
}

export function useSseContext(): SseContextValue {
  const ctx = useContext(SseContext)
  if (!ctx) throw new Error('useSseContext must be used within SseProvider')
  return ctx
}
