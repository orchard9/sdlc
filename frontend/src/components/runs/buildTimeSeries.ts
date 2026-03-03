import type { RawRunEvent } from '@/lib/types'

export interface BucketData {
  /** Milliseconds from run start at bucket start */
  startMs: number
  /** Milliseconds from run start at bucket end */
  endMs: number
  /** Milliseconds in this bucket spent waiting on LLM inference */
  llm: number
  /** Milliseconds in this bucket spent on tool execution */
  tool: number
  /** Milliseconds in this bucket spent on subagent work */
  subagent: number
  /** Milliseconds in this bucket with no classified activity */
  idle: number
}

export interface TimeSeriesData {
  buckets: BucketData[]
  runDurationMs: number
}

type WaitType = 'llm' | 'tool' | 'subagent' | 'idle'

interface Interval {
  type: WaitType
  startMs: number
  endMs: number
}

/**
 * Compute the ms overlap between an interval [iStart, iEnd] and a bucket [bStart, bEnd].
 */
function overlap(iStart: number, iEnd: number, bStart: number, bEnd: number): number {
  return Math.max(0, Math.min(iEnd, bEnd) - Math.max(iStart, bStart))
}

/**
 * Build a bucketed time-series breakdown from a flat array of run events.
 *
 * Returns null if fewer than 2 events carry a `timestamp` wall-clock field,
 * which means the run predates the telemetry-wallclock-timestamps feature.
 * The field name is `timestamp` — matching message_to_event() in runs.rs.
 *
 * @param events   Raw run events from GET /api/runs/:id/telemetry
 * @param bucketCount  Number of equal-width time buckets (default 20)
 */
export function buildTimeSeries(
  events: RawRunEvent[],
  bucketCount = 20,
): TimeSeriesData | null {
  // --- Step 1: extract timestamped events ---
  interface TimestampedEvent {
    type: RawRunEvent['type']
    tsMs: number
    task_id?: string
  }

  const timestamped: TimestampedEvent[] = []
  for (const e of events) {
    if (e.timestamp) {
      const ms = Date.parse(e.timestamp)
      if (!isNaN(ms)) {
        timestamped.push({ type: e.type, tsMs: ms, task_id: e.task_id })
      }
    }
  }

  if (timestamped.length < 2) return null

  const runStartMs = timestamped[0].tsMs
  const runEndMs = timestamped[timestamped.length - 1].tsMs
  const runDurationMs = runEndMs - runStartMs

  if (runDurationMs <= 0) return null

  // --- Step 2: derive typed intervals ---
  const intervals: Interval[] = []

  // Track open subagent spans keyed by task_id
  const openSubagents = new Map<string, number>()

  for (let i = 0; i < timestamped.length; i++) {
    const cur = timestamped[i]
    const next = timestamped[i + 1]

    // Handle subagent open/close
    if (cur.type === 'subagent_started' && cur.task_id) {
      openSubagents.set(cur.task_id, cur.tsMs)
    } else if (cur.type === 'subagent_completed' && cur.task_id) {
      const startMs = openSubagents.get(cur.task_id)
      if (startMs != null) {
        intervals.push({ type: 'subagent', startMs, endMs: cur.tsMs })
        openSubagents.delete(cur.task_id)
      }
    }

    if (!next) break

    const spanStartMs = cur.tsMs - runStartMs
    const spanEndMs = next.tsMs - runStartMs

    // assistant → user: LLM inference
    if (cur.type === 'assistant' && next.type === 'user') {
      intervals.push({ type: 'llm', startMs: spanStartMs, endMs: spanEndMs })
    }
    // user → assistant: tool execution
    else if (cur.type === 'user' && next.type === 'assistant') {
      intervals.push({ type: 'tool', startMs: spanStartMs, endMs: spanEndMs })
    }
    // All other consecutive pairs: idle
    else if (
      cur.type !== 'subagent_started' &&
      cur.type !== 'subagent_completed' &&
      cur.type !== 'subagent_progress'
    ) {
      intervals.push({ type: 'idle', startMs: spanStartMs, endMs: spanEndMs })
    }
  }

  // Close any open subagents at the last timestamp
  for (const [, startMs] of openSubagents) {
    intervals.push({ type: 'subagent', startMs: startMs - runStartMs, endMs: runDurationMs })
  }

  // Normalise subagent interval boundaries to [0, runDurationMs]
  const clampedIntervals = intervals.map(iv => ({
    ...iv,
    startMs: Math.max(0, iv.startMs),
    endMs: Math.min(runDurationMs, iv.endMs),
  }))

  // --- Step 3: fill buckets ---
  const bucketWidthMs = runDurationMs / bucketCount
  const buckets: BucketData[] = []

  for (let b = 0; b < bucketCount; b++) {
    const bStart = b * bucketWidthMs
    const bEnd = bStart + bucketWidthMs

    let llm = 0
    let tool = 0
    let subagent = 0

    for (const iv of clampedIntervals) {
      const ov = overlap(iv.startMs, iv.endMs, bStart, bEnd)
      if (ov <= 0) continue
      if (iv.type === 'llm') llm += ov
      else if (iv.type === 'tool') tool += ov
      else if (iv.type === 'subagent') subagent += ov
    }

    // Idle = whatever is not claimed by the three active types, clamped to [0, bucketWidth]
    const idle = Math.max(0, bucketWidthMs - llm - tool - subagent)

    buckets.push({ startMs: bStart, endMs: bEnd, llm, tool, subagent, idle })
  }

  return { buckets, runDurationMs }
}
