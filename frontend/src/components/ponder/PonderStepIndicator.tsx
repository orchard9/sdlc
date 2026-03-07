import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PonderStatus } from '@/lib/types'

const STEPS = ['exploring', 'converging', 'committed'] as const

const STEP_LABELS: Record<string, string> = {
  exploring: 'Exploring',
  converging: 'Converging',
  committed: 'Committed',
}

const CURRENT_COLORS: Record<string, string> = {
  exploring: 'bg-violet-600/20 text-violet-400',
  converging: 'bg-amber-600/20 text-amber-400',
  committed: 'bg-emerald-600/20 text-emerald-400',
}

const DOT_FILLED: Record<string, string> = {
  exploring: 'bg-violet-500',
  converging: 'bg-amber-500',
  committed: 'bg-emerald-500',
}

const DOT_RING: Record<string, string> = {
  exploring: 'bg-violet-500 ring-2 ring-violet-500/40',
  converging: 'bg-amber-500 ring-2 ring-amber-500/40',
  committed: 'bg-emerald-500 ring-2 ring-emerald-500/40',
}

interface Props {
  status: PonderStatus
  compact?: boolean
}

export function PonderStepIndicator({ status, compact }: Props) {
  const isParked = status === 'parked'
  const currentIdx = isParked ? -1 : STEPS.indexOf(status as typeof STEPS[number])
  // committed means all steps are done
  const effectiveIdx = status === 'committed' ? STEPS.length : currentIdx

  if (compact) {
    return (
      <div className="flex items-center gap-1">
        {STEPS.map((step, i) => {
          const isCompleted = effectiveIdx > i
          const isCurrent = !isParked && effectiveIdx === i
          return (
            <div key={step} className="flex items-center gap-1">
              <span
                className={cn(
                  'block w-2 h-2 rounded-full',
                  isParked && 'bg-muted-foreground/20',
                  !isParked && isCompleted && DOT_FILLED[step],
                  !isParked && isCurrent && DOT_RING[step],
                  !isParked && !isCompleted && !isCurrent && 'border border-muted-foreground/30',
                )}
                title={STEP_LABELS[step]}
              />
              {i < STEPS.length - 1 && (
                <span className={cn(
                  'block w-2 h-px',
                  isCompleted && !isParked ? 'bg-muted-foreground/40' : 'bg-muted-foreground/15',
                )} />
              )}
            </div>
          )
        })}
      </div>
    )
  }

  return (
    <div className="flex items-center gap-1 flex-wrap">
      {STEPS.map((step, i) => {
        const isCompleted = effectiveIdx > i
        const isCurrent = !isParked && effectiveIdx === i
        return (
          <div key={step} className="flex items-center gap-1">
            <span
              className={cn(
                'flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium select-none',
                isParked && 'text-muted-foreground/25',
                !isParked && isCompleted && 'text-muted-foreground/60',
                !isParked && isCurrent && CURRENT_COLORS[step],
                !isParked && !isCompleted && !isCurrent && 'text-muted-foreground/30',
              )}
            >
              {isCompleted && !isParked && (
                <Check className="w-3 h-3 shrink-0" aria-hidden="true" />
              )}
              {STEP_LABELS[step]}
            </span>
            {i < STEPS.length - 1 && (
              <span
                className={cn(
                  'text-xs select-none',
                  isCompleted && !isParked ? 'text-muted-foreground/40' : 'text-muted-foreground/20',
                )}
                aria-hidden="true"
              >
                &rarr;
              </span>
            )}
          </div>
        )
      })}
      {isParked && (
        <span className="ml-1 px-2 py-0.5 rounded text-xs font-medium bg-neutral-600/30 text-neutral-400">
          Parked
        </span>
      )}
    </div>
  )
}
