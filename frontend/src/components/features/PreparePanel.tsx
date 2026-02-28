import { useCallback, useEffect, useState } from 'react'
import { useSSE } from '@/hooks/useSSE'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { api } from '@/api/client'
import { WavePlan } from './WavePlan'
import { CommandBlock } from '@/components/shared/CommandBlock'
import type { PrepareResult } from '@/lib/types'
import { AlertTriangle, CheckCircle, Layers, Play, Loader2 } from 'lucide-react'

function PhaseLabel({ phase }: { phase: PrepareResult['project_phase'] }) {
  const labels: Record<string, string> = {
    idle: 'Idle',
    pondering: 'Pondering',
    planning: 'Planning',
    executing: 'Executing',
    verifying: 'Verifying',
  }
  const colors: Record<string, string> = {
    idle: 'text-muted-foreground',
    pondering: 'text-purple-400',
    planning: 'text-blue-400',
    executing: 'text-primary',
    verifying: 'text-green-400',
  }

  return (
    <span className={`text-xs font-semibold ${colors[phase.phase] ?? 'text-muted-foreground'}`}>
      {labels[phase.phase] ?? phase.phase}
      {phase.milestone && <span className="font-normal text-muted-foreground ml-1">({phase.milestone})</span>}
    </span>
  )
}

function ProgressBar({ progress }: { progress: PrepareResult['milestone_progress'] }) {
  if (!progress) return null
  const pct = progress.total > 0 ? Math.round((progress.released / progress.total) * 100) : 0

  return (
    <div className="flex items-center gap-3 text-xs text-muted-foreground">
      <div className="flex-1 h-1.5 bg-muted rounded-full overflow-hidden">
        <div className="h-full bg-green-500 rounded-full transition-all" style={{ width: `${pct}%` }} />
      </div>
      <span className="tabular-nums shrink-0">
        {progress.released}/{progress.total} released
      </span>
      {progress.in_progress > 0 && <span>{progress.in_progress} active</span>}
      {progress.blocked > 0 && <span className="text-amber-400">{progress.blocked} blocked</span>}
    </div>
  )
}


function VerifyingPanel({ phase, milestone }: {
  phase: PrepareResult['project_phase']
  milestone?: string
}) {
  const slug = milestone ?? ''
  const key = `milestone-uat:${slug}`
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()
  const running = isRunning(key)
  const activeRun = getRunForKey(key)

  const handleStart = () => {
    startRun({
      key,
      runType: 'milestone_uat',
      target: slug,
      label: `UAT: ${slug}`,
      startUrl: `/api/milestone/${encodeURIComponent(slug)}/uat`,
      stopUrl: `/api/milestone/${encodeURIComponent(slug)}/uat/stop`,
    })
  }

  const handleFocus = () => {
    if (activeRun) focusRun(activeRun.id)
  }

  return (
    <section className="mb-6">
      <div className="flex items-center gap-2 mb-2">
        <Layers className="w-3.5 h-3.5 text-muted-foreground" />
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Wave Plan</h3>
        <PhaseLabel phase={phase} />
      </div>
      <div className="bg-green-950/20 border border-green-500/20 rounded-xl p-4 space-y-3">
        <div className="flex items-start justify-between gap-3">
          <div>
            <div className="flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-green-400" />
              <p className="text-sm text-green-400 font-medium">All features released</p>
            </div>
            <p className="text-xs text-muted-foreground mt-1">Run UAT to verify and close the milestone</p>
          </div>
          {running ? (
            <button
              onClick={handleFocus}
              className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-muted text-muted-foreground border border-border text-xs font-medium hover:bg-muted/80 transition-colors"
            >
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Running...
            </button>
          ) : (
            <button
              onClick={handleStart}
              className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-green-500/20 text-green-400 border border-green-500/30 text-xs font-medium hover:bg-green-500/30 transition-colors"
            >
              <Play className="w-3.5 h-3.5" />
              Run UAT
            </button>
          )}
        </div>

        {slug && (
          <CommandBlock cmd={`/sdlc-milestone-uat ${slug}`} />
        )}
      </div>
    </section>
  )
}

export function PreparePanel() {
  const [result, setResult] = useState<PrepareResult | null>(null)
  const [err, setErr] = useState<string | null>(null)

  const load = useCallback(() => {
    api.getProjectPrepare()
      .then(r => { setResult(r); setErr(null) })
      .catch(e => setErr(e.message))
  }, [])

  useEffect(load, [load])
  useSSE(load)

  if (err) {
    return (
      <section className="mb-6">
        <div className="flex items-center gap-2 mb-2">
          <Layers className="w-3.5 h-3.5 text-muted-foreground" />
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Wave Plan</h3>
        </div>
        <div className="bg-red-950/20 border border-red-500/20 rounded-xl p-4 text-center">
          <AlertTriangle className="w-5 h-5 text-red-400 mx-auto mb-1" />
          <p className="text-sm text-red-400">Failed to load wave plan</p>
          <p className="text-xs text-muted-foreground mt-0.5">{err}</p>
        </div>
      </section>
    )
  }

  if (!result) return null

  const { project_phase, waves, blocked, milestone_progress, next_commands } = result

  // Phase-specific messaging for empty waves
  if (waves.length === 0) {
    if (project_phase.phase === 'idle') {
      return (
        <section className="mb-6">
          <div className="flex items-center gap-2 mb-2">
            <Layers className="w-3.5 h-3.5 text-muted-foreground" />
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Wave Plan</h3>
            <PhaseLabel phase={project_phase} />
          </div>
          <div className="bg-card border border-dashed border-border rounded-xl p-4 text-center">
            <p className="text-sm text-muted-foreground">Nothing planned yet</p>
            <p className="text-xs text-muted-foreground/60 mt-0.5">Use /sdlc-ponder to start exploring ideas</p>
          </div>
        </section>
      )
    }
    if (project_phase.phase === 'pondering') {
      return (
        <section className="mb-6">
          <div className="flex items-center gap-2 mb-2">
            <Layers className="w-3.5 h-3.5 text-muted-foreground" />
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Wave Plan</h3>
            <PhaseLabel phase={project_phase} />
          </div>
          <div className="bg-card border border-dashed border-border rounded-xl p-4 text-center">
            <p className="text-sm text-muted-foreground">Active ponders in progress</p>
            <p className="text-xs text-muted-foreground/60 mt-0.5">Use /sdlc-ponder to continue developing ideas</p>
          </div>
        </section>
      )
    }
    if (project_phase.phase === 'verifying') {
      return (
        <VerifyingPanel phase={project_phase} milestone={project_phase.milestone} />
      )
    }
    // Planning/Executing with no waves = nothing actionable
    return null
  }

  return (
    <section className="mb-6">
      <div className="flex items-center gap-2 mb-2">
        <Layers className="w-3.5 h-3.5 text-muted-foreground" />
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Wave Plan</h3>
        <PhaseLabel phase={project_phase} />
      </div>

      <ProgressBar progress={milestone_progress} />

      <div className={`${milestone_progress ? 'mt-2' : ''}`}>
        <WavePlan waves={waves} blocked={blocked} nextCommands={next_commands} />
      </div>
    </section>
  )
}
