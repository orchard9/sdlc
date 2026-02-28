import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

interface Props {
  phases: string[]
  current: string
}

function toTitleCase(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1)
}

export function PhaseStrip({ phases, current }: Props) {
  // When current is "done", every phase is completed.
  const isDone = current === 'done'
  const currentIdx = isDone ? phases.length : phases.indexOf(current)
  // If current is not found (and not "done"), currentIdx === -1, treated as all upcoming.

  return (
    <div className="flex items-center gap-1 flex-wrap">
      {phases.map((phase, i) => {
        const isCompleted = currentIdx > i
        const isCurrent = !isDone && currentIdx === i

        return (
          <div key={phase} className="flex items-center gap-1">
            {/* Phase label */}
            <span
              className={cn(
                'flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium select-none',
                isCurrent && 'bg-primary/10 text-primary font-semibold',
                isCompleted && 'text-muted-foreground/60',
                !isCurrent && !isCompleted && 'text-muted-foreground/30',
              )}
            >
              {isCompleted && (
                <Check className="w-3 h-3 shrink-0" aria-hidden="true" />
              )}
              {toTitleCase(phase)}
            </span>

            {/* Separator — only between phases, not after the last */}
            {i < phases.length - 1 && (
              <span
                className={cn(
                  'text-xs select-none',
                  // Dim the separator if both adjacent phases are upcoming
                  i >= currentIdx - 1 && !isDone
                    ? 'text-muted-foreground/20'
                    : 'text-muted-foreground/40',
                )}
                aria-hidden="true"
              >
                →
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}
