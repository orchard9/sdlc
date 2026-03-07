import type { HubSummary } from '@/lib/types'

const KPI_ITEMS: Array<{ key: keyof HubSummary; label: string; tone: string }> = [
  { key: 'online', label: 'Online', tone: 'bg-emerald-400' },
  { key: 'degraded', label: 'Degraded', tone: 'bg-amber-400' },
  { key: 'provisioning', label: 'Provisioning', tone: 'bg-sky-400' },
  { key: 'failed', label: 'Failed', tone: 'bg-rose-400' },
  { key: 'active_agents', label: 'Active Agents', tone: 'bg-indigo-400' },
  { key: 'attention_count', label: 'Needs Attention', tone: 'bg-orange-400' },
]

export function KpiStrip({ summary }: { summary: HubSummary | null }) {
  if (!summary) return null

  return (
    <div className="grid grid-cols-2 gap-3 xl:grid-cols-6">
      {KPI_ITEMS.map(item => (
        <div
          key={item.key}
          className="rounded-xl border border-border bg-card px-4 py-4"
        >
          <div className="flex items-center gap-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
            <span className={`h-2 w-2 rounded-full ${item.tone}`} />
            <span>{item.label}</span>
          </div>
          <div className="mt-3 text-2xl font-semibold tracking-tight">{summary[item.key]}</div>
        </div>
      ))}
    </div>
  )
}
