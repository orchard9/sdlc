import type { HeatmapData } from '@/hooks/useHeatmap'

interface ConcurrencyStripProps {
  data: HeatmapData
  height?: number
}

export function ConcurrencyStrip({ data, height = 24 }: ConcurrencyStripProps) {
  const { buckets, peakConcurrency } = data

  if (buckets.length === 0) {
    return <div style={{ height }} className="w-full bg-muted/20 rounded" />
  }

  return (
    <div
      className="w-full flex items-end gap-px overflow-hidden rounded"
      style={{ height }}
      aria-label={`Concurrency strip: ${peakConcurrency} peak concurrent runs`}
    >
      {buckets.map((count, i) => {
        const ratio = peakConcurrency > 0 ? count / peakConcurrency : 0
        const barHeight = count === 0 ? 1 : Math.max(2, Math.round(ratio * height))
        const opacity = count === 0 ? 0.15 : 0.4 + ratio * 0.6

        return (
          <div
            key={i}
            title={count === 0 ? 'idle' : `${count} active`}
            className="flex-1 min-w-[2px]"
            style={{ height: `${barHeight}px`, opacity }}
          >
            <div
              className={count === 0 ? 'bg-muted-foreground' : 'bg-primary'}
              style={{ width: '100%', height: '100%' }}
            />
          </div>
        )
      })}
    </div>
  )
}
