import { useState } from 'react'
import { Activity, X } from 'lucide-react'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { RunCard } from './RunCard'

export function AgentPanelFab() {
  const { runs, activeRuns, expandedRunIds, toggleRun } = useAgentRuns()
  const [drawerOpen, setDrawerOpen] = useState(false)

  const completedRuns = runs.filter(r => r.status !== 'running')
  const hasActivity = runs.length > 0

  if (!hasActivity && !drawerOpen) return null

  return (
    <>
      {/* FAB button — mobile only */}
      <button
        onClick={() => setDrawerOpen(true)}
        className="md:hidden fixed bottom-[56px] right-4 z-40 w-12 h-12 rounded-full bg-card border border-border shadow-lg flex items-center justify-center hover:bg-muted transition-colors"
        aria-label="Open agent activity"
      >
        <Activity className="w-5 h-5 text-foreground" />
        {activeRuns.length > 0 && (
          <span className="absolute -top-0.5 -right-0.5 w-4 h-4 rounded-full bg-green-500 text-[9px] font-bold text-white flex items-center justify-center">
            {activeRuns.length}
          </span>
        )}
      </button>

      {/* Slide-out drawer — mobile only */}
      {drawerOpen && (
        <>
          {/* Backdrop */}
          <div
            className="md:hidden fixed inset-0 z-50 bg-black/50"
            onClick={() => setDrawerOpen(false)}
          />

          {/* Drawer */}
          <div className="md:hidden fixed inset-y-0 right-0 z-50 w-80 max-w-[85vw] bg-background border-l border-border flex flex-col animate-in slide-in-from-right duration-200">
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-2.5 border-b border-border">
              <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                Agent Activity
              </h2>
              <button
                onClick={() => setDrawerOpen(false)}
                className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
              {runs.length === 0 && (
                <p className="text-xs text-muted-foreground text-center py-8">No recent activity</p>
              )}

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
            </div>
          </div>
        </>
      )}
    </>
  )
}
