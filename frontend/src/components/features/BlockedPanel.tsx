import { useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, Play, Loader2 } from 'lucide-react'

interface BlockedPanelProps {
  slug: string
  blockers: string[]
  allSlugs: string[]
  isRunning: boolean
  onRunWithDirection: (direction: string) => void
}

export function BlockedPanel({ slug, blockers, allSlugs, isRunning, onRunWithDirection }: BlockedPanelProps) {
  const [removingIdx, setRemovingIdx] = useState<number | null>(null)
  const [reasons, setReasons] = useState<Record<number, string>>({})
  const [submitting, setSubmitting] = useState<number | null>(null)
  const [direction, setDirection] = useState('')

  const handleRemoveClick = (idx: number) => {
    setRemovingIdx(idx)
  }

  const handleCancel = () => {
    setRemovingIdx(null)
  }

  const handleConfirmRemove = async (idx: number) => {
    setSubmitting(idx)
    try {
      const reason = reasons[idx]?.trim()
      await fetch(`/api/features/${slug}/blockers/${idx}`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: reason ? JSON.stringify({ reason }) : undefined,
      })
      // SSE Update event will trigger useFeature to refetch — no manual refresh needed
      setRemovingIdx(null)
      setReasons(prev => {
        const next = { ...prev }
        delete next[idx]
        return next
      })
    } catch {
      // ignore — next SSE refresh will reconcile
    } finally {
      setSubmitting(null)
    }
  }

  const handleRunWithDirection = () => {
    if (!direction.trim() || isRunning) return
    onRunWithDirection(direction.trim())
  }

  return (
    <div className="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 mb-6">
      {/* Header */}
      <div className="flex items-center gap-2 mb-3">
        <AlertTriangle className="w-4 h-4 text-amber-500 shrink-0" />
        <span className="text-sm font-semibold text-amber-500">Blocked</span>
      </div>

      {/* Blocker list */}
      <ul className="space-y-2 mb-4">
        {blockers.map((blocker, idx) => {
          const isInProject = allSlugs.includes(blocker)
          const isRemoving = removingIdx === idx
          const isSubmitting = submitting === idx

          return (
            <li key={idx} className="space-y-1.5">
              <div className="flex items-center gap-2 flex-wrap">
                <span className="text-sm text-muted-foreground font-mono">•</span>
                <span className="text-sm">{blocker}</span>
                {isInProject && (
                  <Link
                    to={`/features/${blocker}`}
                    className="text-xs text-primary hover:underline"
                  >
                    → {blocker}
                  </Link>
                )}
                {!isRemoving && (
                  <button
                    onClick={() => handleRemoveClick(idx)}
                    className="ml-auto text-xs px-2 py-0.5 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                  >
                    Remove
                  </button>
                )}
              </div>

              {/* Inline remove confirmation */}
              {isRemoving && (
                <div className="flex items-center gap-2 flex-wrap pl-4">
                  <input
                    type="text"
                    placeholder="Reason (optional)"
                    value={reasons[idx] ?? ''}
                    onChange={e => setReasons(prev => ({ ...prev, [idx]: e.target.value }))}
                    className="flex-1 min-w-0 text-xs px-2 py-1 rounded border border-border/50 bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                    autoFocus
                  />
                  <button
                    onClick={() => handleConfirmRemove(idx)}
                    disabled={isSubmitting}
                    className="text-xs px-2 py-1 rounded bg-destructive/80 hover:bg-destructive text-destructive-foreground disabled:opacity-50 transition-colors"
                  >
                    {isSubmitting ? <Loader2 className="w-3 h-3 animate-spin" /> : 'Confirm'}
                  </button>
                  <button
                    onClick={handleCancel}
                    disabled={isSubmitting}
                    className="text-xs px-2 py-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground disabled:opacity-50 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              )}
            </li>
          )
        })}
      </ul>

      {/* Divider */}
      <div className="border-t border-amber-500/20 mb-3" />

      {/* Direction + Run */}
      <div className="space-y-2">
        <p className="text-xs text-muted-foreground">Give the agent direction to advance</p>
        <input
          type="text"
          placeholder='e.g. "skip auth-setup, use env vars"'
          value={direction}
          onChange={e => setDirection(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') handleRunWithDirection() }}
          className="w-full text-sm px-3 py-1.5 rounded-lg border border-border/50 bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <button
          onClick={handleRunWithDirection}
          disabled={!direction.trim() || isRunning}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {isRunning ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Running...
            </>
          ) : (
            <>
              <Play className="w-3.5 h-3.5" />
              Run with direction
            </>
          )}
        </button>
      </div>
    </div>
  )
}
