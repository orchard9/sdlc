import { useMemo } from 'react'
import type { RunRecord } from '@/lib/types'

export interface RunLane {
  run: RunRecord
  startBucket: number
  endBucket: number
}

export interface HeatmapData {
  bucketSizeMs: number
  startMs: number
  endMs: number
  buckets: number[]
  lanes: RunLane[]
  peakConcurrency: number
  spanLabel: string
}

function selectBucketSize(rangeMs: number): number {
  const TEN_MINUTES = 10 * 60 * 1000
  const ONE_HOUR = 60 * 60 * 1000
  const SIX_HOURS = 6 * 60 * 60 * 1000

  if (rangeMs <= TEN_MINUTES) return 30_000       // 30s
  if (rangeMs <= ONE_HOUR) return 120_000          // 2min
  if (rangeMs <= SIX_HOURS) return 600_000         // 10min
  return 1_800_000                                 // 30min
}

function formatSpanLabel(ms: number): string {
  const minutes = Math.round(ms / 60_000)
  if (minutes < 60) return `${minutes} minute${minutes !== 1 ? 's' : ''}`
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  if (remainingMinutes === 0) return `${hours}h`
  return `${hours}h ${remainingMinutes}m`
}

function computeHeatmap(runs: RunRecord[]): HeatmapData {
  const now = Date.now()

  // Filter to runs with a valid started_at
  const validRuns = runs.filter(r => r.started_at != null && r.started_at !== '')
  if (validRuns.length === 0) {
    return {
      bucketSizeMs: 30_000,
      startMs: now,
      endMs: now,
      buckets: [],
      lanes: [],
      peakConcurrency: 0,
      spanLabel: '0 minutes',
    }
  }

  // Parse run times; treat completed_at = null as now
  const parsedRuns = validRuns.map(r => ({
    run: r,
    startMs: new Date(r.started_at).getTime(),
    endMs: r.completed_at ? new Date(r.completed_at).getTime() : now,
  }))

  const rawStart = Math.min(...parsedRuns.map(r => r.startMs))
  const rawEnd = Math.max(...parsedRuns.map(r => r.endMs))
  const rawRange = rawEnd - rawStart

  // Add 5% margin on each side
  const margin = Math.max(rawRange * 0.05, 1000)
  const startMs = rawStart - margin
  const endMs = rawEnd + margin
  const rangeMs = endMs - startMs

  const bucketSizeMs = selectBucketSize(rawRange)
  const bucketCount = Math.ceil(rangeMs / bucketSizeMs)

  // Initialize bucket array
  const buckets = new Array<number>(bucketCount).fill(0)

  // Build lanes and populate buckets
  const lanes: RunLane[] = []
  for (const { run, startMs: runStart, endMs: runEnd } of parsedRuns) {
    const startBucket = Math.max(0, Math.floor((runStart - startMs) / bucketSizeMs))
    const endBucket = Math.min(bucketCount - 1, Math.floor((runEnd - startMs) / bucketSizeMs))

    lanes.push({ run, startBucket, endBucket })

    for (let b = startBucket; b <= endBucket; b++) {
      buckets[b]++
    }
  }

  const peakConcurrency = buckets.length > 0 ? Math.max(...buckets) : 0
  const spanLabel = formatSpanLabel(rawEnd - rawStart)

  return {
    bucketSizeMs,
    startMs,
    endMs,
    buckets,
    lanes,
    peakConcurrency,
    spanLabel,
  }
}

export function useHeatmap(runs: RunRecord[]): HeatmapData {
  return useMemo(() => computeHeatmap(runs), [runs])
}
