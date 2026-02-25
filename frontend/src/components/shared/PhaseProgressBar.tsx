import { cn } from '@/lib/utils'
import { PHASES, phaseIndex } from '@/lib/phases'
import type { Phase } from '@/lib/types'

interface PhaseProgressBarProps {
  current: Phase
  className?: string
}

export function PhaseProgressBar({ current, className }: PhaseProgressBarProps) {
  const currentIdx = phaseIndex(current)

  return (
    <div className={cn('flex gap-0.5', className)}>
      {PHASES.map((phase, i) => (
        <div
          key={phase}
          className={cn(
            'h-1.5 flex-1 rounded-full transition-colors',
            i <= currentIdx ? 'bg-primary' : 'bg-muted'
          )}
          title={phase}
        />
      ))}
    </div>
  )
}
