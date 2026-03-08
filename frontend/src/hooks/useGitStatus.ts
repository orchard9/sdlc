import { useCallback, useEffect, useRef, useState } from 'react'
import { useSSE } from './useSSE'

export interface GitStatus {
  branch: string
  dirty_count: number
  staged_count: number
  untracked_count: number
  ahead: number
  behind: number
  has_conflicts: boolean
  conflict_count: number
  severity: 'green' | 'yellow' | 'red'
  summary: string
}

export function useGitStatus(intervalMs = 10_000) {
  const [status, setStatus] = useState<GitStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const fetchStatus = useCallback(async () => {
    try {
      const res = await fetch('/api/git/status')
      if (!res.ok) throw new Error(res.statusText)
      const data: GitStatus = await res.json()
      setStatus(data)
      setError(false)
    } catch {
      setError(true)
    } finally {
      setLoading(false)
    }
  }, [])

  // Initial fetch + interval
  useEffect(() => {
    fetchStatus()
    timerRef.current = setInterval(fetchStatus, intervalMs)
    return () => {
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [fetchStatus, intervalMs])

  // Pause when tab hidden, resume + re-fetch on visible
  useEffect(() => {
    const handleVisibility = () => {
      if (document.hidden) {
        if (timerRef.current) {
          clearInterval(timerRef.current)
          timerRef.current = null
        }
      } else {
        fetchStatus()
        timerRef.current = setInterval(fetchStatus, intervalMs)
      }
    }
    document.addEventListener('visibilitychange', handleVisibility)
    return () => document.removeEventListener('visibilitychange', handleVisibility)
  }, [fetchStatus, intervalMs])

  // Re-fetch on window focus
  useEffect(() => {
    const handleFocus = () => fetchStatus()
    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [fetchStatus])

  // Re-fetch on SSE update events (e.g. GitCommitCompleted)
  useSSE(fetchStatus)

  return { status, loading, error, refetch: fetchStatus }
}
