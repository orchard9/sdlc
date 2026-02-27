import { useEffect } from 'react'

/** Subscribe to /api/events and call onUpdate whenever state changes. */
export function useSSE(onUpdate: () => void) {
  useEffect(() => {
    const es = new EventSource('/api/events')
    es.addEventListener('update', onUpdate)
    es.onerror = () => {} // browser auto-reconnects
    return () => es.close()
  }, [onUpdate])
}
