import { Link } from 'react-router-dom'
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PonderSummary, MilestoneSummary } from '@/lib/types'

// ---------------------------------------------------------------------------
// Stage definitions
// ---------------------------------------------------------------------------

const STAGES = [
  {
    label: 'Ponder',
    href: '/ponder',
    tooltip: 'Explore ideas before committing to a plan',
  },
  {
    label: 'Plan',
    href: '/ponder',
    tooltip: 'Review and refine the auto-generated milestone plan',
  },
  {
    label: 'Commit',
    href: '/milestones',
    tooltip: 'Commit the plan — creates features in wave order',
  },
  {
    label: 'Run Wave',
    href: '/milestones',
    tooltip: 'Start a wave — agents build features in parallel',
  },
  {
    label: 'Ship',
    href: '/milestones',
    tooltip: 'Features shipped — milestone complete',
  },
] as const

// ---------------------------------------------------------------------------
// Stage determination logic (greedy — highest reached stage wins)
// ---------------------------------------------------------------------------

function computeCurrentStage(
  ponders: PonderSummary[],
  milestones: MilestoneSummary[]
): number {
  // Stage 4 (Ship): at least one released milestone
  if (milestones.some(m => m.status === 'released')) return 4

  // Stage 3 (Run Wave): at least one active or verifying milestone
  if (milestones.some(m => m.status === 'active' || m.status === 'verifying')) return 3

  // Stage 2 (Commit): at least one milestone exists
  if (milestones.length > 0) return 2

  // Stage 1 (Plan): at least one ponder that is committed
  if (ponders.some(p => p.status === 'committed')) return 1

  // Stage 0 (Ponder): always — shown as the entry point for new projects
  return 0
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface PipelineIndicatorProps {
  ponders: PonderSummary[]
  milestones: MilestoneSummary[]
}

export function PipelineIndicator({ ponders, milestones }: PipelineIndicatorProps) {
  const currentStage = computeCurrentStage(ponders, milestones)

  return (
    <div className="flex items-center gap-1.5 py-2" role="navigation" aria-label="SDLC pipeline stages">
      {STAGES.map((stage, i) => {
        const isCompleted = i < currentStage
        const isCurrent = i === currentStage
        const isFuture = i > currentStage

        return (
          <div key={stage.label} className="flex items-center gap-1.5">
            <Link
              to={stage.href}
              title={stage.tooltip}
              className={cn(
                'flex items-center gap-1 rounded-full px-2.5 py-1 text-xs font-medium transition-opacity hover:opacity-80',
                isCurrent && 'bg-primary text-primary-foreground',
                isCompleted && 'bg-muted text-muted-foreground',
                isFuture && 'border border-border text-muted-foreground/60 hover:text-muted-foreground'
              )}
              aria-current={isCurrent ? 'step' : undefined}
            >
              {isCompleted && <Check className="w-3 h-3 shrink-0" />}
              {stage.label}
            </Link>
            {i < STAGES.length - 1 && (
              <span className="text-muted-foreground/40 text-xs select-none">→</span>
            )}
          </div>
        )
      })}
    </div>
  )
}
