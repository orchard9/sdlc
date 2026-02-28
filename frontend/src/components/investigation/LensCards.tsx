import { cn } from '@/lib/utils'
import type { LensScores } from '@/lib/types'

type Maturity = 'low' | 'medium' | 'high' | 'excellent'

const LENSES: { key: keyof LensScores; label: string; question: string }[] = [
  { key: 'pit_of_success',    label: 'Pit of Success',     question: 'Do defaults lead to good outcomes?' },
  { key: 'coupling',          label: 'Coupling',            question: 'Are related things together?' },
  { key: 'growth_readiness',  label: 'Growth Readiness',    question: 'Will this scale to 10×?' },
  { key: 'self_documenting',  label: 'Self-Documenting',    question: 'Can you understand it from the code?' },
  { key: 'failure_modes',     label: 'Failure Modes',       question: 'What happens when it breaks?' },
]

const MATURITY_STYLES: Record<Maturity, string> = {
  low:       'bg-red-500/10 text-red-400 border-red-500/20',
  medium:    'bg-amber-500/10 text-amber-400 border-amber-500/20',
  high:      'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
  excellent: 'bg-blue-500/10 text-blue-400 border-blue-500/20',
}

function normalizeMaturity(raw: string | undefined): Maturity | null {
  if (!raw) return null
  const s = raw.toLowerCase().trim()
  if (s === 'low') return 'low'
  if (s === 'medium') return 'medium'
  if (s === 'high') return 'high'
  if (s === 'excellent') return 'excellent'
  return null
}

function LensCard({ label, question, maturity }: { label: string; question: string; maturity: Maturity | null }) {
  const pending = maturity === null
  return (
    <div className={cn(
      'rounded-lg border px-3 py-2.5 text-xs',
      pending ? 'border-border/30 bg-card/40' : 'border-border/50 bg-card',
    )}>
      <div className="flex items-center justify-between gap-2 min-w-0">
        <span className={cn(
          'font-medium truncate',
          pending ? 'text-muted-foreground/40' : 'text-foreground/80',
        )}>
          {label}
        </span>
        {maturity && (
          <span className={cn(
            'shrink-0 inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium border',
            MATURITY_STYLES[maturity],
          )}>
            {maturity}
          </span>
        )}
      </div>
      {!pending && (
        <p className="mt-1 text-muted-foreground/50 leading-snug truncate">{question}</p>
      )}
    </div>
  )
}

interface Props {
  lensScores: LensScores | null
}

export function LensCards({ lensScores }: Props) {
  return (
    <div className="space-y-1.5 px-3 py-3">
      {LENSES.map(({ key, label, question }) => (
        <LensCard
          key={key}
          label={label}
          question={question}
          maturity={normalizeMaturity(lensScores?.[key])}
        />
      ))}
    </div>
  )
}
