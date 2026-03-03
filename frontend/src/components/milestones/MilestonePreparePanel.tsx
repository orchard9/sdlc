import { useCallback, useEffect, useState } from 'react'
import { useSSE } from '@/hooks/useSSE'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { api } from '@/api/client'
import { WavePlan } from '@/components/features/WavePlan'
import type { PrepareResult, Wave } from '@/lib/types'
import { CheckCircle, Loader2, Play } from 'lucide-react'

function ProgressBarMini({ progress, waves }: {
  progress: NonNullable<PrepareResult['milestone_progress']>
  waves: Wave[]
}) {
  const pct = progress.total > 0 ? Math.round((progress.released / progress.total) * 100) : 0
  const currentWave = waves[0]?.number ?? null

  return (
    <div className="flex items-center gap-3 text-xs text-muted-foreground">
      <div className="flex-1 h-1.5 bg-muted rounded-full overflow-hidden">
        <div className="h-full bg-green-500 rounded-full transition-all" style={{ width: `${pct}%` }} />
      </div>
      <span className="tabular-nums shrink-0">
        {progress.released}/{progress.total} released
      </span>
      {currentWave !== null && pct < 100 && (
        <span className="text-muted-foreground/70">wave {currentWave}</span>
      )}
      {progress.in_progress > 0 && <span>{progress.in_progress} active</span>}
      {progress.blocked > 0 && <span className="text-amber-400">{progress.blocked} blocked</span>}
    </div>
  )
}

function VerifyingMini({ milestoneSlug }: { milestoneSlug: string }) {
  const key = `milestone-uat:${milestoneSlug}`
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()
  const running = isRunning(key)
  const activeRun = getRunForKey(key)

  const handleStart = () => {
    startRun({
      key,
      runType: 'milestone_uat',
      target: milestoneSlug,
      label: `UAT: ${milestoneSlug}`,
      startUrl: `/api/milestone/${encodeURIComponent(milestoneSlug)}/uat`,
      stopUrl: `/api/milestone/${encodeURIComponent(milestoneSlug)}/uat/stop`,
    })
  }

  const handleFocus = () => {
    if (activeRun) focusRun(activeRun.id)
  }

  return (
    <div className="flex items-center justify-between gap-3 bg-green-950/20 border border-green-500/20 rounded-lg px-3 py-2">
      <div className="flex items-center gap-2">
        <CheckCircle className="w-4 h-4 text-green-400 shrink-0" />
        <span className="text-xs text-green-400 font-medium">All features released</span>
      </div>
      {running ? (
        <button
          onClick={handleFocus}
          className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-border bg-muted text-muted-foreground text-[10px] hover:bg-muted/80 transition-colors whitespace-nowrap"
        >
          <Loader2 className="w-3 h-3 animate-spin" />
          Running
        </button>
      ) : (
        <button
          onClick={handleStart}
          className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-green-500/30 bg-green-500/20 text-green-400 text-[10px] hover:bg-green-500/30 transition-colors whitespace-nowrap"
        >
          <Play className="w-3 h-3" />
          Run UAT
        </button>
      )}
    </div>
  )
}

export function MilestonePreparePanel({ milestoneSlug }: { milestoneSlug: string }) {
  const [result, setResult] = useState<PrepareResult | null>(null)

  const load = useCallback(() => {
    api.getProjectPrepare(milestoneSlug)
      .then(r => setResult(r))
      .catch(() => {})
  }, [milestoneSlug])

  useEffect(() => { load() }, [load])
  const noop = useCallback(() => {}, [])
  useSSE(noop, undefined, (event) => { if (event.type === 'run_finished') load() })

  if (!result) return null

  const { waves, blocked, milestone_progress, next_commands } = result

  const isVerifying =
    waves.length === 0 &&
    milestone_progress != null &&
    milestone_progress.total > 0 &&
    milestone_progress.released === milestone_progress.total

  if (waves.length === 0 && !isVerifying) return null

  if (isVerifying) {
    return <VerifyingMini milestoneSlug={milestoneSlug} />
  }

  return (
    <div className="space-y-2">
      {milestone_progress && (
        <ProgressBarMini progress={milestone_progress} waves={waves} />
      )}
      <WavePlan waves={waves} blocked={blocked} nextCommands={next_commands} showCommands={false} milestoneSlug={milestoneSlug} />
    </div>
  )
}
