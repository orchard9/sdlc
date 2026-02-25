import { useCallback, useRef, useState } from 'react'
import { connectSSE, type RunEvent } from '@/lib/sse'

export interface RunLine {
  type: 'stdout' | 'stderr'
  text: string
}

export interface UseRunStreamOptions {
  onComplete?: () => void
}

export function useRunStream(options?: UseRunStreamOptions) {
  const [lines, setLines] = useState<RunLine[]>([])
  const [running, setRunning] = useState(false)
  const [exitCode, setExitCode] = useState<number | null>(null)
  const esRef = useRef<EventSource | null>(null)
  const onCompleteRef = useRef(options?.onComplete)
  onCompleteRef.current = options?.onComplete

  const start = useCallback((runId: string) => {
    setLines([])
    setRunning(true)
    setExitCode(null)

    esRef.current = connectSSE(runId, (event: RunEvent) => {
      if (event.type === 'stdout' && event.line !== undefined) {
        setLines(prev => [...prev, { type: 'stdout', text: event.line! }])
      } else if (event.type === 'stderr' && event.line !== undefined) {
        setLines(prev => [...prev, { type: 'stderr', text: event.line! }])
      } else if (event.type === 'finished') {
        setExitCode(event.exit_code ?? -1)
        setRunning(false)
        onCompleteRef.current?.()
      } else if (event.type === 'error') {
        setLines(prev => [...prev, { type: 'stderr', text: `Error: ${event.message ?? 'unknown'}` }])
        setRunning(false)
      }
    })
  }, [])

  const stop = useCallback(() => {
    esRef.current?.close()
    setRunning(false)
  }, [])

  return { lines, running, exitCode, start, stop }
}
