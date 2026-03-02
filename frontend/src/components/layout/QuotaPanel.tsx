import { useAgentRuns } from '@/contexts/AgentRunContext'

function isToday(iso: string): boolean {
  try {
    const d = new Date(iso)
    const now = new Date()
    return (
      d.getFullYear() === now.getFullYear() &&
      d.getMonth() === now.getMonth() &&
      d.getDate() === now.getDate()
    )
  } catch {
    return false
  }
}

export function QuotaPanel() {
  const { runs } = useAgentRuns()

  const totalCostToday = runs
    .filter(r => r.cost_usd != null && r.status !== 'running' && isToday(r.started_at))
    .reduce((sum, r) => sum + (r.cost_usd ?? 0), 0)

  return (
    <div className="space-y-1.5">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/40">
        Quota
      </p>
      <span className="text-xs font-mono text-foreground">
        ${totalCostToday.toFixed(2)} today
      </span>
    </div>
  )
}
