import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { useSSE } from './useSSE'
import type { ProjectState } from '@/lib/types'

export function useProjectState() {
  const [state, setState] = useState<ProjectState | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const refresh = useCallback(async () => {
    try {
      const data = await api.getState()
      setState(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load state')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { refresh() }, [refresh])
  useSSE(refresh)

  return { state, error, loading }
}
