import { cn } from '@/lib/utils'
import { parseAreaArtifact } from '@/lib/parseInvestigation'
import type { InvestigationArtifact } from '@/lib/types'
import type { AreaArtifactMeta } from '@/lib/types'

interface Props {
  artifacts: InvestigationArtifact[]
}

const AREAS: { prefix: string; label: string }[] = [
  { prefix: 'area-1', label: 'Code Paths' },
  { prefix: 'area-2', label: 'Bottlenecks' },
  { prefix: 'area-3', label: 'Data Flow' },
  { prefix: 'area-4', label: 'Auth Chain' },
  { prefix: 'area-5', label: 'Environment' },
]

const statusStyles: Record<AreaArtifactMeta['status'], string> = {
  pending:       '',
  investigating: 'bg-amber-500/10 text-amber-400 border border-amber-500/20',
  finding:       'bg-blue-500/10 text-blue-400 border border-blue-500/20',
  hypothesis:    'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20',
}

const statusLabels: Record<AreaArtifactMeta['status'], string> = {
  pending:       'pending',
  investigating: 'investigating',
  finding:       'finding',
  hypothesis:    'hypothesis',
}

interface AreaCardProps {
  label: string
  meta: AreaArtifactMeta | null
}

function AreaCard({ label, meta }: AreaCardProps) {
  const hasMeta = meta !== null
  const status = meta?.status ?? 'pending'
  const isPending = status === 'pending'

  return (
    <div
      className={cn(
        'rounded-lg border px-3 py-2.5 text-xs',
        hasMeta && !isPending
          ? 'border-border/50 bg-card'
          : 'border-border/30 bg-card/40',
      )}
    >
      <div className="flex items-center justify-between gap-2 min-w-0">
        {/* Left: status badge + area name */}
        <div className="flex items-center gap-2 min-w-0">
          {!isPending && hasMeta ? (
            <span
              className={cn(
                'shrink-0 inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium',
                statusStyles[status],
              )}
            >
              {statusLabels[status]}
            </span>
          ) : null}
          <span
            className={cn(
              'font-medium truncate',
              hasMeta && !isPending ? 'text-foreground/80' : 'text-muted-foreground/40',
            )}
          >
            {label}
          </span>
        </div>

        {/* Right: confidence */}
        {meta?.confidence !== undefined && (
          <span className="shrink-0 text-muted-foreground/50 tabular-nums">
            {meta.confidence}%
          </span>
        )}
      </div>

      {/* Finding preview */}
      {meta?.finding && (
        <p className="mt-1 text-muted-foreground/70 truncate leading-snug">
          {meta.finding}
        </p>
      )}
    </div>
  )
}

export function AreaCards({ artifacts }: Props) {
  return (
    <div className="space-y-1.5 px-3 py-3">
      {AREAS.map(({ prefix, label }) => {
        const artifact = artifacts.find(a => a.filename.startsWith(prefix + '-'))
        const meta =
          artifact && artifact.content != null
            ? parseAreaArtifact(artifact.filename, artifact.content)
            : null

        return (
          <AreaCard key={prefix} label={label} meta={meta} />
        )
      })}
    </div>
  )
}
