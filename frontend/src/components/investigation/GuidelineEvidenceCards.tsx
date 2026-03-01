import { cn } from '@/lib/utils'
import type { EvidenceCounts } from '@/lib/types'

const PERSPECTIVES: { key: keyof EvidenceCounts; label: string; hint: string }[] = [
  { key: 'anti_patterns',  label: 'Anti-patterns',  hint: 'Instances found in the codebase' },
  { key: 'good_examples',  label: 'Good Examples',  hint: 'Exemplars to reference in the guideline' },
  { key: 'prior_art',      label: 'Prior Art',       hint: 'Community / ecosystem precedents' },
  { key: 'adjacent',       label: 'Adjacent',        hint: 'Related patterns and complementary rules' },
]

function EvidenceCard({
  label,
  hint,
  count,
}: {
  label: string
  hint: string
  count: number | undefined
}) {
  const pending = count === undefined || count === 0
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
        {count !== undefined && count > 0 && (
          <span className="shrink-0 inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium border bg-primary/10 text-primary border-primary/20">
            {count}
          </span>
        )}
      </div>
      {!pending && (
        <p className="mt-1 text-muted-foreground/50 leading-snug truncate">{hint}</p>
      )}
    </div>
  )
}

interface Props {
  evidenceCounts: EvidenceCounts | null
}

export function GuidelineEvidenceCards({ evidenceCounts }: Props) {
  return (
    <div className="space-y-1.5 px-3 py-3">
      {PERSPECTIVES.map(({ key, label, hint }) => (
        <EvidenceCard
          key={key}
          label={label}
          hint={hint}
          count={evidenceCounts?.[key]}
        />
      ))}
    </div>
  )
}
