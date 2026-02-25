import { Link } from 'react-router-dom'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import type { FeatureSummary } from '@/lib/types'
import { ArrowRight, AlertCircle } from 'lucide-react'

interface FeatureCardProps {
  feature: FeatureSummary
  position?: number
}

export function FeatureCard({ feature, position }: FeatureCardProps) {
  return (
    <Link
      to={`/features/${feature.slug}`}
      className="block bg-card border border-border rounded-xl p-4 hover:border-primary/40 transition-colors"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-1.5">
            {position != null && (
              <span className="text-[10px] font-mono font-semibold text-muted-foreground/60 shrink-0 tabular-nums">
                #{position}
              </span>
            )}
            <h3 className="text-sm font-medium truncate">{feature.title}</h3>
          </div>
          <p className="text-xs text-muted-foreground mt-0.5 font-mono">{feature.slug}</p>
        </div>
        <StatusBadge status={feature.phase} />
      </div>

      <PhaseProgressBar current={feature.phase} className="mt-3" />

      <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground">
        {feature.blocked && (
          <span className="flex items-center gap-1 text-destructive">
            <AlertCircle className="w-3 h-3" />
            Blocked
          </span>
        )}
        <span>{feature.task_summary}</span>
      </div>

      <div className="mt-2 flex items-center gap-1.5 text-xs text-primary">
        <ArrowRight className="w-3 h-3" />
        <span className="truncate">{feature.next_action.replace(/_/g, ' ')}</span>
      </div>
    </Link>
  )
}
