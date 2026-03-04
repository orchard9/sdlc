import { useEffect, useRef, useState, useMemo } from 'react'
import { buildTimeSeries } from './buildTimeSeries'
import type { RawRunEvent } from '@/lib/types'

// ---------------------------------------------------------------------------
// Color palette — muted to fit the existing dark UI aesthetic
// ---------------------------------------------------------------------------

const COLORS = {
  llm:      'hsl(270 55% 58%)',  // violet — LLM inference
  tool:     'hsl(35 65% 55%)',   // amber  — tool execution
  subagent: 'hsl(210 55% 55%)', // steel  — subagent delegation
  idle:     'hsl(220 10% 32%)', // gray   — idle / overhead
} as const

type WaitKey = keyof typeof COLORS

const LEGEND_LABELS: { key: WaitKey; label: string }[] = [
  { key: 'llm',      label: 'LLM' },
  { key: 'tool',     label: 'Tool' },
  { key: 'subagent', label: 'Subagent' },
  { key: 'idle',     label: 'Idle' },
]

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatMs(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)}ms`
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`
  const m = Math.floor(ms / 60_000)
  const s = Math.round((ms % 60_000) / 1000)
  return `${m}m ${s}s`
}

function xAxisLabels(runDurationMs: number, count: number): { label: string; pct: number }[] {
  const labels: { label: string; pct: number }[] = []
  for (let i = 0; i <= count; i++) {
    const ms = (runDurationMs * i) / count
    labels.push({ label: formatMs(ms), pct: i / count })
  }
  return labels
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

interface TooltipState {
  x: number
  y: number
  bucketIndex: number
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

interface ActivityTimeSeriesProps {
  events: RawRunEvent[]
  isRunning: boolean
}

const CHART_HEIGHT = 72
const PADDING_LEFT = 4
const PADDING_RIGHT = 4
const PADDING_BOTTOM = 20  // room for x-axis labels
const PADDING_TOP = 2
const BAR_GAP = 1

export function ActivityTimeSeries({ events }: ActivityTimeSeriesProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [chartWidth, setChartWidth] = useState(300)
  const [tooltip, setTooltip] = useState<TooltipState | null>(null)

  // Observe container width changes
  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const ro = new ResizeObserver(entries => {
      const w = entries[0]?.contentRect.width
      if (w && w > 0) setChartWidth(w)
    })
    ro.observe(el)
    // Initial measure
    const w = el.getBoundingClientRect().width
    if (w > 0) setChartWidth(w)
    return () => ro.disconnect()
  }, [])

  const data = useMemo(() => buildTimeSeries(events, 20), [events])

  if (!data) {
    return (
      <p className="text-[11px] text-muted-foreground/50 italic py-2">
        Time breakdown not available (run predates timestamps)
      </p>
    )
  }

  const { buckets, runDurationMs } = data
  const plotWidth = chartWidth - PADDING_LEFT - PADDING_RIGHT
  const plotHeight = CHART_HEIGHT - PADDING_TOP - PADDING_BOTTOM
  const barWidth = Math.max(1, (plotWidth - BAR_GAP * (buckets.length - 1)) / buckets.length)

  // Maximum value across all buckets for normalisation
  const maxBucketTotal = Math.max(...buckets.map(b => b.llm + b.tool + b.subagent + b.idle), 1)

  const xLabels = xAxisLabels(runDurationMs, 4)

  return (
    <div ref={containerRef} className="relative w-full select-none overflow-hidden">
      {/* Legend */}
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 mb-1">
        <span className="text-[10px] text-muted-foreground/60 font-medium">Activity</span>
        {LEGEND_LABELS.map(({ key, label }) => (
          <span key={key} className="flex items-center gap-1">
            <span
              className="inline-block w-2 h-2 rounded-sm shrink-0"
              style={{ backgroundColor: COLORS[key] }}
            />
            <span className="text-[10px] text-muted-foreground/70">{label}</span>
          </span>
        ))}
      </div>

      {/* SVG chart */}
      <svg
        width={chartWidth}
        height={CHART_HEIGHT}
        className="overflow-hidden"
        onMouseLeave={() => setTooltip(null)}
      >
        {/* Bars */}
        {buckets.map((bucket, i) => {
          const x = PADDING_LEFT + i * (barWidth + BAR_GAP)
          const total = bucket.llm + bucket.tool + bucket.subagent + bucket.idle
          const scale = total > 0 ? plotHeight / maxBucketTotal : 0

          type Seg = { key: WaitKey; ms: number }
          const segments: Seg[] = [
            { key: 'idle',     ms: bucket.idle },
            { key: 'subagent', ms: bucket.subagent },
            { key: 'tool',     ms: bucket.tool },
            { key: 'llm',      ms: bucket.llm },
          ]

          let yOffset = PADDING_TOP + plotHeight
          const rects = segments.map(seg => {
            const h = seg.ms * scale
            yOffset -= h
            return { key: seg.key, x, y: yOffset, width: barWidth, height: h }
          })

          return (
            <g
              key={i}
              onMouseEnter={e => {
                const rect = containerRef.current?.getBoundingClientRect()
                const svgRect = (e.currentTarget.ownerSVGElement as SVGElement).getBoundingClientRect()
                const relX = svgRect.left - (rect?.left ?? 0) + x + barWidth / 2
                setTooltip({ x: relX, y: PADDING_TOP, bucketIndex: i })
              }}
              style={{ cursor: 'default' }}
            >
              {/* Invisible hit area for easier hover */}
              <rect
                x={x}
                y={PADDING_TOP}
                width={barWidth}
                height={plotHeight}
                fill="transparent"
              />
              {rects.map(r =>
                r.height > 0.5 ? (
                  <rect
                    key={r.key}
                    x={r.x}
                    y={r.y}
                    width={r.width}
                    height={r.height}
                    fill={COLORS[r.key]}
                    rx={1}
                  />
                ) : null
              )}
            </g>
          )
        })}

        {/* X-axis baseline */}
        <line
          x1={PADDING_LEFT}
          y1={PADDING_TOP + plotHeight}
          x2={PADDING_LEFT + plotWidth}
          y2={PADDING_TOP + plotHeight}
          stroke="hsl(220 10% 25%)"
          strokeWidth={1}
        />

        {/* X-axis labels */}
        {xLabels.map(({ label, pct }, i) => (
          <text
            key={i}
            x={PADDING_LEFT + pct * plotWidth}
            y={CHART_HEIGHT - 2}
            fontSize={9}
            fill="hsl(220 10% 45%)"
            textAnchor={i === 0 ? 'start' : i === xLabels.length - 1 ? 'end' : 'middle'}
          >
            {label}
          </text>
        ))}
      </svg>

      {/* Tooltip */}
      {tooltip != null && (() => {
        const b = buckets[tooltip.bucketIndex]
        const startLabel = formatMs(b.startMs)
        const endLabel = formatMs(b.endMs)
        return (
          <div
            className="absolute z-10 pointer-events-none"
            style={{
              left: Math.min(tooltip.x, chartWidth - 120),
              top: tooltip.y + 4,
            }}
          >
            <div className="bg-card border border-border/60 rounded px-2 py-1.5 shadow-md text-[10px] space-y-0.5 min-w-[100px]">
              <p className="text-muted-foreground font-medium mb-1">{startLabel} – {endLabel}</p>
              {b.llm > 0.5      && <p><span style={{ color: COLORS.llm }}>LLM</span>      <span className="float-right ml-4 text-foreground/80">{formatMs(b.llm)}</span></p>}
              {b.tool > 0.5     && <p><span style={{ color: COLORS.tool }}>Tool</span>     <span className="float-right ml-4 text-foreground/80">{formatMs(b.tool)}</span></p>}
              {b.subagent > 0.5 && <p><span style={{ color: COLORS.subagent }}>Subagent</span> <span className="float-right ml-4 text-foreground/80">{formatMs(b.subagent)}</span></p>}
              {b.idle > 0.5     && <p><span style={{ color: COLORS.idle }}>Idle</span>     <span className="float-right ml-4 text-foreground/80">{formatMs(b.idle)}</span></p>}
            </div>
          </div>
        )
      })()}
    </div>
  )
}
