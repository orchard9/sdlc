import { useEffect, useState } from 'react'
import { api } from '@/api/client'
import type { ProjectState } from '@/lib/types'

export function useProjectState() {
  const [state, setState] = useState<ProjectState | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const refresh = async () => {
    try {
      setLoading(true)
      const data = await api.getState()
      setState(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load state')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { refresh() }, [])

  return { state, error, loading, refresh }
}
