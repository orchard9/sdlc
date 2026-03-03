import { BarChart2 } from 'lucide-react'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { RunsHeatmap } from '@/components/runs/RunsHeatmap'

function EmptyState() {
  return (
    <div className="text-center py-16 text-muted-foreground">
      <BarChart2 className="w-10 h-10 mx-auto mb-3 opacity-30" />
      <p className="text-sm">No concurrent runs to display yet.</p>
      <p className="text-xs mt-1">Start at least 2 agent runs to see the heatmap.</p>
    </div>
  )
}

export function RunsPage() {
  const { runs, focusRun } = useAgentRuns()

  return (
    <div className="max-w-5xl mx-auto p-4 sm:p-6 space-y-6">
      <div className="flex items-center gap-2 mb-1">
        <BarChart2 className="w-5 h-5 text-muted-foreground" />
        <h2 className="text-xl font-semibold">Run History</h2>
      </div>
      <p className="text-sm text-muted-foreground">
        Cross-run concurrency view. Spot parallelism opportunities and idle gaps.
      </p>

      {runs.length < 2 ? (
        <EmptyState />
      ) : (
        <RunsHeatmap runs={runs} onRunClick={run => focusRun(run.id)} />
      )}
    </div>
  )
}
