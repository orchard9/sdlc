import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useSseContext } from '@/contexts/SseContext'

const STORAGE_KEY = 'sdlc_last_visit_at'
const SEVEN_DAYS_MS = 7 * 24 * 60 * 60 * 1000

export type EventKind =
  | 'feature_merged'
  | 'run_failed'
  | 'milestone_wave_completed'
  | 'feature_phase_advanced'
  | 'review_approved'
  | 'audit_approved'
  | 'qa_approved'

export interface ChangeEvent {
  id: string
  kind: EventKind
  slug: string
  title: string
  timestamp: string
}

interface ChangelogResponse {
  events: ChangeEvent[]
  total: number
}

export interface ChangelogResult {
  events: ChangeEvent[]
  total: number
  /** ISO timestamp from localStorage — null means first visit */
  lastVisitAt: string | null
  loading: boolean
  /** true after the dismiss() call — banner should hide immediately */
  dismissed: boolean
  /** Sets last_visit_at = now in localStorage and hides the banner */
  dismiss: () => void
}

export function useChangelog(): ChangelogResult {
  const [lastVisitAt] = useState<string | null>(() =>
    localStorage.getItem(STORAGE_KEY)
  )
  const [events, setEvents] = useState<ChangeEvent[]>([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)
  const [dismissed, setDismissed] = useState(false)

  const since = useMemo(
    () => lastVisitAt ?? new Date(Date.now() - SEVEN_DAYS_MS).toISOString(),
    [lastVisitAt]
  )

  const refresh = useCallback(async () => {
    try {
      const params = new URLSearchParams({ since, limit: '50' })
      const res = await fetch(`/api/changelog?${params}`)
      if (res.status === 404) {
        // changelog-api not yet deployed — silently show nothing
        setEvents([])
        setTotal(0)
        return
      }
      if (!res.ok) {
        // Other errors: hide banner quietly
        setEvents([])
        setTotal(0)
        return
      }
      const data: ChangelogResponse = await res.json()
      setEvents(data.events)
      setTotal(data.total)
    } catch {
      // Network error — hide banner quietly
      setEvents([])
      setTotal(0)
    } finally {
      setLoading(false)
    }
  }, [since])

  useEffect(() => { refresh() }, [refresh])

  // Re-fetch only when a ChangelogUpdated SSE event arrives — not on every update event.
  // Debounced: rapid phase-advance writes (e.g. 7 features in a wave) each update changelog.yaml
  // within the same polling window. Coalesce bursts into a single fetch after 2 s of quiet.
  const { subscribe } = useSseContext()
  const refreshRef = useRef(refresh)
  useEffect(() => { refreshRef.current = refresh })
  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | null = null
    const unsubscribe = subscribe({
      onChangelogEvent: () => {
        if (timer) clearTimeout(timer)
        timer = setTimeout(() => refreshRef.current(), 2000)
      },
    })
    return () => {
      unsubscribe()
      if (timer) clearTimeout(timer)
    }
  }, [subscribe])

  const dismiss = useCallback(() => {
    localStorage.setItem(STORAGE_KEY, new Date().toISOString())
    setDismissed(true)
  }, [])

  return { events, total, lastVisitAt, loading, dismissed, dismiss }
}
