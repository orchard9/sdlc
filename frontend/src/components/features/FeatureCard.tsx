import { Link, useNavigate } from 'react-router-dom'
import { useState } from 'react'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { PhaseProgressBar } from '@/components/shared/PhaseProgressBar'
import type { FeatureSummary } from '@/lib/types'
import { ArrowRight, AlertCircle, FileText, Copy, Check } from 'lucide-react'

interface FeatureCardProps {
  feature: FeatureSummary
  position?: number
}

function deriveCommand(feature: FeatureSummary): string | null {
  if (
    feature.next_action === 'done' ||
    feature.next_action === 'wait_for_approval' ||
    feature.next_action === 'unblock_dependency'
  ) return null
  return `/sdlc-run ${feature.slug}`
}

export function FeatureCard({ feature, position }: FeatureCardProps) {
  const navigate = useNavigate()
  const [copied, setCopied] = useState(false)
  const cmd = deriveCommand(feature)

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation()
    if (!cmd) return
    navigator.clipboard.writeText(cmd).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <div
      className="bg-card border border-border rounded-xl p-4 hover:border-primary/40 transition-colors cursor-pointer"
      onClick={() => navigate(`/features/${feature.slug}`)}
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

      <div className="mt-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-1.5 text-xs text-primary min-w-0">
          <ArrowRight className="w-3 h-3 shrink-0" />
          <span className="truncate">
            {feature.next_action === 'create_spec' ? 'view spec' : feature.next_action.replace(/_/g, ' ')}
          </span>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <Link
            to={`/features/${feature.slug}#artifact-spec`}
            onClick={e => e.stopPropagation()}
            className="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            title="View spec"
          >
            <FileText className="w-3.5 h-3.5" />
          </Link>
          {cmd && (
            <button
              onClick={handleCopy}
              className="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
              title={`Copy: ${cmd}`}
            >
              {copied
                ? <Check className="w-3.5 h-3.5 text-green-400" />
                : <Copy className="w-3.5 h-3.5" />
              }
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
