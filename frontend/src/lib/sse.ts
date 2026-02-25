export interface RunEvent {
  type: 'stdout' | 'stderr' | 'finished' | 'error'
  line?: string
  exit_code?: number
  duration_seconds?: number
  message?: string
}

export function connectSSE(
  runId: string,
  onEvent: (event: RunEvent) => void,
  onError?: (err: Event) => void
): EventSource {
  const es = new EventSource(`/api/runs/${runId}/stream`)

  es.onmessage = (msg) => {
    try {
      const event: RunEvent = JSON.parse(msg.data)
      onEvent(event)
      if (event.type === 'finished' || event.type === 'error') {
        es.close()
      }
    } catch {
      // ignore parse errors
    }
  }

  es.onerror = (err) => {
    onError?.(err)
    es.close()
  }

  return es
}
