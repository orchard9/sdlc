import { useEffect, useRef, useState } from 'react'
import { X } from 'lucide-react'
import { useAgentRuns } from '@/contexts/AgentRunContext'

const STORAGE_KEY = 'sdlc_first_wave_seen'

/**
 * One-time overlay that appears after the first milestone wave run completes.
 * Persisted via localStorage — shown exactly once per user/browser.
 */
export function WaveCompleteOverlay() {
  const { runs } = useAgentRuns()
  const [visible, setVisible] = useState(false)
  const [featureCount, setFeatureCount] = useState(0)
  const seenIds = useRef<Set<string>>(new Set())

  useEffect(() => {
    if (localStorage.getItem(STORAGE_KEY) === 'true') return

    for (const run of runs) {
      if (
        run.run_type === 'milestone_run_wave' &&
        run.status === 'completed' &&
        !seenIds.current.has(run.id)
      ) {
        seenIds.current.add(run.id)
        // Count is unavailable here — we'll just show "features" without a count
        // unless we can derive it from the run label or target.
        setFeatureCount(0)
        setVisible(true)
        break
      }
    }
  }, [runs])

  const dismiss = () => {
    localStorage.setItem(STORAGE_KEY, 'true')
    setVisible(false)
  }

  if (!visible) return null

  return (
    <div
      role="dialog"
      aria-label="Wave complete"
      className="fixed bottom-6 right-6 z-50 max-w-sm w-full bg-card border border-border rounded-xl shadow-xl p-5 animate-in slide-in-from-bottom-4 fade-in duration-300"
      onClick={dismiss}
    >
      <button
        onClick={e => { e.stopPropagation(); dismiss() }}
        className="absolute top-3 right-3 p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
        aria-label="Dismiss"
      >
        <X className="w-3.5 h-3.5" />
      </button>
      <p className="text-sm font-semibold mb-1">
        Wave complete.{featureCount > 0 ? ` ${featureCount} features built in parallel.` : ''}
      </p>
      <p className="text-sm text-muted-foreground leading-relaxed">
        This is how SDLC works: you ponder, you commit, you run — then check in on results.
        You don't need to watch while agents work.
      </p>
    </div>
  )
}
