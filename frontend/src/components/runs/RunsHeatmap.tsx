import { useHeatmap } from '@/hooks/useHeatmap'
import { ConcurrencyStrip } from './ConcurrencyStrip'
import type { RunRecord, RunType } from '@/lib/types'

const RUN_TYPE_COLORS: Record<RunType, string> = {
  feature: 'bg-blue-500/70',
  milestone_uat: 'bg-purple-500/70',
  milestone_prepare: 'bg-amber-500/70',
  milestone_run_wave: 'bg-amber-600/70',
  ponder: 'bg-teal-500/70',
  investigation: 'bg-teal-600/70',
  vision_align: 'bg-green-500/70',
  architecture_align: 'bg-green-600/70',
}
const DEFAULT_COLOR = 'bg-muted-foreground/40'

function getRunColor(runType: RunType | string): string {
  return (RUN_TYPE_COLORS as Record<string, string>)[runType] ?? DEFAULT_COLOR
}

function formatDuration(startedAt: string, completedAt?: string): string {
  const start = new Date(startedAt).getTime()
  const end = completedAt ? new Date(completedAt).getTime() : Date.now()
  const ms = end - start
  const seconds = Math.round(ms / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remSec = seconds % 60
  return remSec > 0 ? `${minutes}m ${remSec}s` : `${minutes}m`
}

interface RunsHeatmapProps {
  runs: RunRecord[]
  onRunClick?: (run: RunRecord) => void
  compact?: boolean
}

export function RunsHeatmap({ runs, onRunClick, compact = false }: RunsHeatmapProps) {
  const data = useHeatmap(runs)
  const { buckets, lanes, peakConcurrency, spanLabel, bucketSizeMs } = data

  const totalBuckets = buckets.length

  if (totalBuckets === 0) return null

  if (compact) {
    return (
      <div className="space-y-1">
        <ConcurrencyStrip data={data} height={20} />
        <p className="text-[10px] text-muted-foreground">
          {runs.length} run{runs.length !== 1 ? 's' : ''} · peak {peakConcurrency} concurrent · {spanLabel}
        </p>
      </div>
    )
  }

  // Time axis: one tick every 5 buckets
  const tickInterval = 5
  const tickIndices: number[] = []
  for (let i = 0; i < totalBuckets; i += tickInterval) {
    tickIndices.push(i)
  }

  return (
    <div className="space-y-2 overflow-x-auto">
      {/* Span label */}
      <p className="text-xs text-muted-foreground">Span: {spanLabel}</p>

      {/* Concurrency strip */}
      <div className="pr-2">
        <ConcurrencyStrip data={data} height={32} />
      </div>

      {/* Run lanes */}
      <div className="space-y-0.5 min-w-[400px]">
        {lanes.map(({ run, startBucket, endBucket }) => {
          const leftPct = (startBucket / totalBuckets) * 100
          const widthPct = Math.max(0.5, ((endBucket - startBucket + 1) / totalBuckets) * 100)
          const colorClass = getRunColor(run.run_type)
          const duration = formatDuration(run.started_at, run.completed_at)
          const tooltip = `${run.label} — ${run.run_type} — ${duration}`

          return (
            <div key={run.id} className="flex items-center gap-2 h-5">
              {/* Label column */}
              <div className="w-28 shrink-0 text-[10px] text-muted-foreground truncate text-right" title={run.label}>
                {run.label}
              </div>

              {/* Bar area */}
              <div className="flex-1 relative h-4 rounded overflow-hidden bg-muted/20">
                <div
                  className={`absolute top-0 h-full rounded ${colorClass} ${onRunClick ? 'cursor-pointer hover:ring-1 hover:ring-primary/60' : ''}`}
                  style={{
                    left: `${leftPct}%`,
                    width: `${widthPct}%`,
                  }}
                  title={tooltip}
                  onClick={() => onRunClick?.(run)}
                />
              </div>
            </div>
          )
        })}
      </div>

      {/* Time axis */}
      <div className="flex relative text-[10px] text-muted-foreground ml-[120px] min-w-[280px]">
        {tickIndices.map(i => {
          const offsetMs = i * bucketSizeMs
          const offsetMin = Math.round(offsetMs / 60_000)
          const label = offsetMin === 0 ? '+0m' : `+${offsetMin}m`
          const leftPct = (i / totalBuckets) * 100

          return (
            <span
              key={i}
              className="absolute"
              style={{ left: `${leftPct}%`, transform: 'translateX(-50%)' }}
            >
              {label}
            </span>
          )
        })}
      </div>
      {/* Spacer for time axis */}
      <div className="h-4" />
    </div>
  )
}
