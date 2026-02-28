import { useState } from 'react'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { RunCard } from './RunCard'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import { PanelRightClose, Maximize2 } from 'lucide-react'

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
  const { panelOpen, setPanelOpen } = useAgentRuns()
  const [fullscreen, setFullscreen] = useState(false)

  if (!panelOpen) return null

  return (
    <>
      <aside className="hidden md:flex flex-col w-72 border-l border-border bg-background shrink-0 overflow-hidden">
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

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
          <RunList />
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
      </FullscreenModal>
    </>
  )
}
