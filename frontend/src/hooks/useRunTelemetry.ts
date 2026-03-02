import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '@/api/client'
import type { RunTelemetry } from '@/lib/types'

/**
 * Fetch telemetry for a given run.
 * While the run is active (status === 'running'), re-fetches every 2 seconds.
 * Stops polling once the run completes.
 */
export function useRunTelemetry(runId: string, isRunning: boolean) {
  const [telemetry, setTelemetry] = useState<RunTelemetry | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const fetch = useCallback(async () => {
    if (!runId) return
    try {
      const data = await api.getRunTelemetry(runId)
      setTelemetry(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load telemetry')
    } finally {
      setIsLoading(false)
    }
  }, [runId])

  useEffect(() => {
    if (!runId) return
    setIsLoading(true)
    fetch()

    if (isRunning) {
      intervalRef.current = setInterval(fetch, 2000)
    }

    return () => {
      if (intervalRef.current != null) {
        clearInterval(intervalRef.current)
        intervalRef.current = null
      }
    }
  }, [runId, isRunning, fetch])

  // Stop polling when run transitions from running to not running
  useEffect(() => {
    if (!isRunning && intervalRef.current != null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
      // Final fetch to capture completed state
      fetch()
    }
  }, [isRunning, fetch])

  return { telemetry, isLoading, error }
}
