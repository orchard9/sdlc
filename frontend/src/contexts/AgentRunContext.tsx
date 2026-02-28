import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import type { RunRecord, RunSseEvent, RunStatus } from '@/lib/types'

interface StartRunOpts {
  key: string
  runType: 'feature' | 'milestone_uat' | 'milestone_prepare' | 'ponder'
  target: string
  label: string
  startUrl: string
  stopUrl: string
}

interface AgentRunContextValue {
  runs: RunRecord[]
  activeRuns: RunRecord[]
  isRunning: (key: string) => boolean
  getRunForKey: (key: string) => RunRecord | undefined
  startRun: (opts: StartRunOpts) => Promise<void>
  stopRun: (key: string, stopUrl: string) => Promise<void>
  panelOpen: boolean
  setPanelOpen: (open: boolean) => void
  expandedRunIds: Set<string>
  toggleRun: (id: string) => void
  focusRun: (id: string) => void
}

const AgentRunContext = createContext<AgentRunContextValue | null>(null)

const PANEL_STORAGE_KEY = 'sdlc-agent-panel-open'

function isDesktop() {
  return typeof window !== 'undefined' && window.innerWidth >= 768
}

export function AgentRunProvider({ children }: { children: ReactNode }) {
  const [runs, setRuns] = useState<RunRecord[]>([])
  const [panelOpen, setPanelOpenRaw] = useState(() => {
    const stored = localStorage.getItem(PANEL_STORAGE_KEY)
    if (stored !== null) return stored === 'true'
    return isDesktop()
  })
  const [expandedRunIds, setExpandedRunIds] = useState<Set<string>>(new Set())
  const scrollRef = useRef<string | null>(null)

  const setPanelOpen = useCallback((open: boolean) => {
    setPanelOpenRaw(open)
    localStorage.setItem(PANEL_STORAGE_KEY, String(open))
  }, [])

  // Fetch initial run history
  useEffect(() => {
    api.getRuns()
      .then(setRuns)
      .catch(() => {})
  }, [])

  // Handle SSE run events
  const handleRunEvent = useCallback((event: RunSseEvent) => {
    if (event.type === 'run_started') {
      setRuns(prev => {
        // Avoid duplicates
        if (prev.some(r => r.id === event.id)) return prev
        const newRun: RunRecord = {
          id: event.id,
          key: event.key,
          run_type: 'feature', // will be corrected on next fetch
          target: event.key,
          label: event.label ?? event.key,
          status: 'running' as RunStatus,
          started_at: new Date().toISOString(),
        }
        return [newRun, ...prev]
      })
    } else if (event.type === 'run_finished') {
      setRuns(prev =>
        prev.map(r =>
          r.id === event.id
            ? { ...r, status: (event.status ?? 'completed') as RunStatus, completed_at: new Date().toISOString() }
            : r,
        ),
      )
      // Refresh full record to get cost/turns
      api.getRuns().then(setRuns).catch(() => {})
    }
  }, [])

  // SSE: no-op for update, no ponder handler, run events handled above
  const noop = useCallback(() => {}, [])
  useSSE(noop, undefined, handleRunEvent)

  const activeRuns = useMemo(() => runs.filter(r => r.status === 'running'), [runs])

  const isRunning = useCallback(
    (key: string) => runs.some(r => r.key === key && r.status === 'running'),
    [runs],
  )

  const getRunForKey = useCallback(
    (key: string) => runs.find(r => r.key === key && r.status === 'running'),
    [runs],
  )

  const startRun = useCallback(async (opts: StartRunOpts) => {
    try {
      await fetch(opts.startUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      })
      // SSE will handle adding the run to the list
    } catch {
      // ignore — SSE or next fetch will reconcile
    }
  }, [])

  const stopRun = useCallback(async (_key: string, stopUrl: string) => {
    try {
      await fetch(stopUrl, { method: 'POST' })
      // SSE will handle updating the run status
    } catch {
      // ignore
    }
  }, [])

  const toggleRun = useCallback((id: string) => {
    setExpandedRunIds(prev => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const focusRun = useCallback((id: string) => {
    setPanelOpen(true)
    setExpandedRunIds(prev => new Set(prev).add(id))
    scrollRef.current = id
  }, [setPanelOpen])

  const value = useMemo<AgentRunContextValue>(
    () => ({
      runs,
      activeRuns,
      isRunning,
      getRunForKey,
      startRun,
      stopRun,
      panelOpen,
      setPanelOpen,
      expandedRunIds,
      toggleRun,
      focusRun,
    }),
    [runs, activeRuns, isRunning, getRunForKey, startRun, stopRun, panelOpen, setPanelOpen, expandedRunIds, toggleRun, focusRun],
  )

  return <AgentRunContext.Provider value={value}>{children}</AgentRunContext.Provider>
}

export function useAgentRuns(): AgentRunContextValue {
  const ctx = useContext(AgentRunContext)
  if (!ctx) throw new Error('useAgentRuns must be used within AgentRunProvider')
  return ctx
}
