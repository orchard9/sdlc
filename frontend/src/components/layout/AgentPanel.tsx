import { useState, useCallback, useRef, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { RunCard } from './RunCard'
import { QuotaPanel } from './QuotaPanel'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import { RunsHeatmap } from '@/components/runs/RunsHeatmap'
import { PanelRightClose, Maximize2 } from 'lucide-react'
import { api } from '@/api/client'
import type { ProjectConfig } from '@/lib/types'

const AGENT_WIDTH_LS_KEY = 'sdlc:agent-panel-width'
const MIN_WIDTH = 200
const MAX_WIDTH = 520
const DEFAULT_WIDTH = 288

function clamp(val: number, min: number, max: number) {
  return Math.min(Math.max(val, min), max)
}

function RunList() {
  const { runs, expandedRunIds, toggleRun } = useAgentRuns()

  const activeRuns = runs.filter(r => r.status === 'running')
  const completedRuns = runs.filter(r => r.status !== 'running')

  if (runs.length === 0) {
    return <p className="text-xs text-muted-foreground text-center py-8">No recent activity</p>
  }

  return (
    <>
      {activeRuns.length > 0 && (
        <div className="space-y-1.5">
          {activeRuns.map(run => (
            <RunCard
              key={run.id}
              run={run}
              expanded={expandedRunIds.has(run.id)}
              onToggle={() => toggleRun(run.id)}
            />
          ))}
        </div>
      )}

      {activeRuns.length > 0 && completedRuns.length > 0 && (
        <div className="border-t border-border/50 my-2" />
      )}

      {completedRuns.length > 0 && (
        <div className="space-y-1.5">
          {completedRuns.map(run => (
            <RunCard
              key={run.id}
              run={run}
              expanded={expandedRunIds.has(run.id)}
              onToggle={() => toggleRun(run.id)}
            />
          ))}
        </div>
      )}
    </>
  )
}

export function AgentPanel() {
  const { panelOpen, setPanelOpen, runs, focusRun } = useAgentRuns()
  const [fullscreen, setFullscreen] = useState(false)
  const [config, setConfig] = useState<ProjectConfig | null>(null)
  const [panelWidth, setPanelWidth] = useState(() => {
    try {
      const stored = localStorage.getItem(AGENT_WIDTH_LS_KEY)
      return stored ? clamp(parseInt(stored, 10), MIN_WIDTH, MAX_WIDTH) : DEFAULT_WIDTH
    } catch { return DEFAULT_WIDTH }
  })
  const dragStartX = useRef<number | null>(null)
  const dragStartWidth = useRef<number>(DEFAULT_WIDTH)

  useEffect(() => {
    api.getConfig().then(cfg => setConfig(cfg)).catch(() => null)
  }, [])

  const handlePointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    dragStartX.current = e.clientX
    dragStartWidth.current = panelWidth
    e.currentTarget.setPointerCapture(e.pointerId)
  }, [panelWidth])

  const handlePointerMove = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (dragStartX.current === null) return
    const delta = dragStartX.current - e.clientX
    const newWidth = clamp(dragStartWidth.current + delta, MIN_WIDTH, MAX_WIDTH)
    setPanelWidth(newWidth)
  }, [])

  const handlePointerUp = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (dragStartX.current === null) return
    dragStartX.current = null
    try { localStorage.setItem(AGENT_WIDTH_LS_KEY, String(panelWidth)) } catch { /* ok */ }
    e.currentTarget.releasePointerCapture(e.pointerId)
  }, [panelWidth])

  if (!panelOpen) return null

  return (
    <>
      <aside
        className="hidden md:flex flex-col border-l border-border bg-background shrink-0 overflow-hidden relative"
        style={{ width: `${panelWidth}px` }}
      >
        {/* Drag handle — left border */}
        {!fullscreen && (
          <div
            data-testid="agent-resize-handle"
            className="absolute left-0 inset-y-0 w-1 cursor-col-resize hover:bg-accent/60 transition-colors z-10"
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
          />
        )}

        {/* Header */}
        <div className="flex items-center justify-between px-3 py-2.5 border-b border-border">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Agent Activity
          </h2>
          <div className="flex items-center gap-0.5">
            <button
              onClick={() => setFullscreen(true)}
              className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
              aria-label="Fullscreen"
            >
              <Maximize2 className="w-3.5 h-3.5" />
            </button>
            <button
              onClick={() => setPanelOpen(false)}
              className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
              aria-label="Collapse panel"
            >
              <PanelRightClose className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Compact heatmap strip — shown when 2+ runs exist */}
        {runs.length >= 2 && (
          <div className="px-3 py-2 border-b border-border/50">
            <RunsHeatmap runs={runs} compact onRunClick={run => focusRun(run.id)} />
            <Link
              to="/runs"
              className="block text-right text-[10px] text-primary hover:underline mt-1"
            >
              full view →
            </Link>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
          <RunList />
        </div>

        {/* Quota panel — always visible at the bottom */}
        <div className="border-t border-border/50 px-2 py-2">
          <QuotaPanel dailyBudgetUsd={config?.observability?.daily_budget_usd} />
        </div>
      </aside>

      <FullscreenModal
        open={fullscreen}
        onClose={() => setFullscreen(false)}
        title="Agent Activity"
      >
        <div className="space-y-2">
          <RunList />
        </div>
        <div className="border-t border-border/50 pt-3 mt-2">
          <QuotaPanel dailyBudgetUsd={config?.observability?.daily_budget_usd} />
        </div>
      </FullscreenModal>
    </>
  )
}
