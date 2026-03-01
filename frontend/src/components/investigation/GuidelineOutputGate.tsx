import { CheckCircle2, FileText } from 'lucide-react'
import type { InvestigationDetail } from '@/lib/types'

interface Props {
  investigation: InvestigationDetail
}

export function GuidelineOutputGate({ investigation }: Props) {
  const { publish_path, title, principles_count } = investigation

  if (publish_path) {
    return (
      <div className="px-3 py-3 space-y-2.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
            Published
          </span>
          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle2 className="w-3 h-3" />
            published
          </span>
        </div>
        {principles_count !== null && principles_count !== undefined && principles_count > 0 && (
          <p className="text-xs text-muted-foreground/60">
            {principles_count} {principles_count === 1 ? 'principle' : 'principles'} documented
          </p>
        )}
        <div className="flex items-center gap-1.5 text-xs font-mono text-muted-foreground/50 truncate">
          <FileText className="w-3 h-3 shrink-0" />
          <span className="truncate">{publish_path}</span>
        </div>
      </div>
    )
  }

  return (
    <div className="px-3 py-3 space-y-2.5">
      <div>
        <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
          Output
        </span>
        <p className="mt-1 text-xs text-foreground/80 leading-snug">
          Guideline distillation complete. Agent will publish to{' '}
          <span className="font-mono">.sdlc/guidelines/{investigation.slug}.md</span>.
        </p>
      </div>
      {principles_count !== null && principles_count !== undefined && principles_count > 0 && (
        <p className="text-xs text-muted-foreground/50">
          {principles_count} {principles_count === 1 ? 'principle' : 'principles'} ready
        </p>
      )}
      <p className="text-xs text-muted-foreground/40 italic">
        "{title}" — waiting for agent to publish
      </p>
    </div>
  )
}
