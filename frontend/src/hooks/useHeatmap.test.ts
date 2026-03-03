import { describe, it, expect, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useHeatmap } from './useHeatmap'
import type { RunRecord } from '@/lib/types'

// Helper to create a RunRecord with sensible defaults
function makeRun(overrides: Partial<RunRecord> & { id: string }): RunRecord {
  return {
    key: overrides.id,
    run_type: 'feature',
    target: overrides.id,
    label: overrides.id,
    status: 'completed',
    started_at: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
    completed_at: new Date().toISOString(),
    ...overrides,
  }
}

describe('useHeatmap', () => {
  it('U1: empty array → returns empty HeatmapData', () => {
    const { result } = renderHook(() => useHeatmap([]))
    expect(result.current.buckets).toEqual([])
    expect(result.current.lanes).toEqual([])
    expect(result.current.peakConcurrency).toBe(0)
  })

  it('U2: single run → 1 lane, peakConcurrency=1', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 2 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.lanes).toHaveLength(1)
    expect(result.current.peakConcurrency).toBe(1)
    // All non-zero buckets should be 1 (single run)
    const nonZero = result.current.buckets.filter(b => b > 0)
    nonZero.forEach(b => expect(b).toBe(1))
  })

  it('U3: two fully overlapping runs → peakConcurrency=2', () => {
    const now = Date.now()
    const start = new Date(now - 5 * 60 * 1000).toISOString()
    const end = new Date(now).toISOString()
    const runs = [
      makeRun({ id: 'r1', started_at: start, completed_at: end }),
      makeRun({ id: 'r2', started_at: start, completed_at: end }),
    ]
    const { result } = renderHook(() => useHeatmap(runs))
    expect(result.current.peakConcurrency).toBe(2)
    // Shared buckets should all be 2
    const nonZero = result.current.buckets.filter(b => b > 0)
    nonZero.forEach(b => expect(b).toBe(2))
  })

  it('U4: two non-overlapping runs → gap buckets are 0', () => {
    const base = Date.now() - 20 * 60 * 1000
    const run1 = makeRun({
      id: 'r1',
      started_at: new Date(base).toISOString(),
      completed_at: new Date(base + 3 * 60 * 1000).toISOString(),
    })
    const run2 = makeRun({
      id: 'r2',
      started_at: new Date(base + 10 * 60 * 1000).toISOString(),
      completed_at: new Date(base + 13 * 60 * 1000).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run1, run2]))
    expect(result.current.peakConcurrency).toBe(1)
    // There must be some zero-count buckets in the gap
    const zeroBuckets = result.current.buckets.filter(b => b === 0)
    expect(zeroBuckets.length).toBeGreaterThan(0)
  })

  it('U5: run with completed_at=null → treated as live (extends to now)', () => {
    const fakeNow = Date.now()
    vi.spyOn(Date, 'now').mockReturnValue(fakeNow)

    const run = makeRun({
      id: 'r1',
      started_at: new Date(fakeNow - 5 * 60 * 1000).toISOString(),
      completed_at: undefined,
      status: 'running',
    })
    const { result } = renderHook(() => useHeatmap([run]))
    // Lane should exist and end at the last bucket (near now)
    expect(result.current.lanes).toHaveLength(1)
    const lane = result.current.lanes[0]
    const totalBuckets = result.current.buckets.length
    expect(lane.endBucket).toBe(totalBuckets - 1)

    vi.restoreAllMocks()
  })

  it('U6: run missing started_at → excluded from lanes and buckets', () => {
    const run = makeRun({ id: 'r1', started_at: '' })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.lanes).toHaveLength(0)
    expect(result.current.buckets).toHaveLength(0)
  })

  it('U7: range ≤ 10min → bucketSizeMs=30000', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 5 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.bucketSizeMs).toBe(30_000)
  })

  it('U8: range between 10min and 1h → bucketSizeMs=120000', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 30 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.bucketSizeMs).toBe(120_000)
  })

  it('U9: range between 1h and 6h → bucketSizeMs=600000', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 2 * 60 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.bucketSizeMs).toBe(600_000)
  })

  it('U10: range > 6h → bucketSizeMs=1800000', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 8 * 60 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.bucketSizeMs).toBe(1_800_000)
  })

  it('U11: spanLabel for 43-minute range', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 43 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    expect(result.current.spanLabel).toMatch(/43 minutes?/)
  })

  it('U12: spanLabel for 134-minute range', () => {
    const now = Date.now()
    const run = makeRun({
      id: 'r1',
      started_at: new Date(now - 134 * 60 * 1000).toISOString(),
      completed_at: new Date(now).toISOString(),
    })
    const { result } = renderHook(() => useHeatmap([run]))
    // 134 minutes = 2h 14m
    expect(result.current.spanLabel).toMatch(/2h\s*14m/)
  })
})
