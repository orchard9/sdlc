import { AlertTriangle } from 'lucide-react'
import { useAgentRuns } from '@/contexts/AgentRunContext'

const DEFAULT_DAILY_BUDGET_USD = 1000.0

interface QuotaPanelProps {
  dailyBudgetUsd?: number
}

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

export function QuotaPanel({ dailyBudgetUsd }: QuotaPanelProps) {
  const { runs } = useAgentRuns()
  const budget = dailyBudgetUsd ?? DEFAULT_DAILY_BUDGET_USD

  const todayRuns = runs.filter(
    r => r.cost_usd != null && r.status !== 'running' && isToday(r.started_at)
  )

  const totalCostToday = todayRuns.reduce((sum, r) => sum + (r.cost_usd ?? 0), 0)
  const pct = (totalCostToday / budget) * 100
  const barPct = Math.min(pct, 100)

  const completedToday = todayRuns.filter(r => r.status === 'completed')
  const avgCostPerRun =
    completedToday.length >= 2 ? totalCostToday / completedToday.length : null

  const remainingRuns =
    avgCostPerRun !== null && avgCostPerRun > 0
      ? Math.max(0, Math.floor((budget - totalCostToday) / avgCostPerRun))
      : null

  const isWarning = pct >= 80 && pct < 100
  const isExceeded = pct >= 100

  const barColor = isExceeded
    ? 'bg-red-500'
    : isWarning
      ? 'bg-amber-500'
      : 'bg-primary'

  return (
    <div className="space-y-1.5">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/40">
        Quota
      </p>

      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-mono text-foreground">
          ${totalCostToday.toFixed(2)} today
        </span>
        <div className="flex items-center gap-1">
          <span className={`text-xs font-mono ${isExceeded ? 'text-red-500' : isWarning ? 'text-amber-500' : 'text-muted-foreground'}`}>
            {Math.round(pct)}%
          </span>
          {(isWarning || isExceeded) && (
            <span aria-label={isExceeded ? 'Daily budget exceeded' : 'Approaching daily limit'}>
              <AlertTriangle className={`w-3 h-3 ${isExceeded ? 'text-red-500' : 'text-amber-500'}`} aria-hidden="true" />
            </span>
          )}
        </div>
      </div>

      {/* Progress bar */}
      <div
        role="progressbar"
        aria-valuenow={Math.round(barPct)}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Daily API quota usage"
        className="h-1.5 rounded-full bg-muted overflow-hidden"
      >
        <div
          className={`h-full rounded-full transition-all duration-300 ${barColor}`}
          style={{ width: `${barPct}%` }}
        />
      </div>

      {/* Status line */}
      {isExceeded ? (
        <p className="text-[10px] text-red-500">Daily budget exceeded</p>
      ) : remainingRuns !== null ? (
        <p className="text-[10px] text-muted-foreground">
          ≈ {remainingRuns} run{remainingRuns !== 1 ? 's' : ''} remaining
          {avgCostPerRun !== null && (
            <span> (at ${avgCostPerRun.toFixed(2)}/run avg)</span>
          )}
        </p>
      ) : null}
    </div>
  )
}
