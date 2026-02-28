import type { PonderOrientation } from '@/lib/types'

interface Props {
  orientation: PonderOrientation | null
}

export function OrientationStrip({ orientation }: Props) {
  if (!orientation || (!orientation.current && !orientation.next && !orientation.commit)) {
    return (
      <div className="border border-border/40 rounded-lg px-4 py-3 text-xs text-muted-foreground/50 italic">
        No orientation yet — run a session to get started.
      </div>
    )
  }

  return (
    <div className="border border-border/60 rounded-lg divide-y divide-border/40 text-xs">
      <div className="flex items-start gap-3 px-4 py-2.5">
        <span className="shrink-0 font-semibold text-muted-foreground/60 uppercase tracking-wider w-28">
          WHERE WE ARE
        </span>
        <span className="text-foreground/80">{orientation.current || '—'}</span>
      </div>
      <div className="flex items-start gap-3 px-4 py-2.5">
        <span className="shrink-0 font-semibold text-primary/70 uppercase tracking-wider w-28">
          → NEXT MOVE
        </span>
        <span className="text-foreground/80">{orientation.next || '—'}</span>
      </div>
      <div className="flex items-start gap-3 px-4 py-2.5">
        <span className="shrink-0 font-semibold text-muted-foreground/60 uppercase tracking-wider w-28">
          COMMIT WHEN
        </span>
        <span className="text-foreground/80">{orientation.commit || '—'}</span>
      </div>
    </div>
  )
}
