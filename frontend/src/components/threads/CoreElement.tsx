import { FileText } from 'lucide-react'
import { cn } from '@/lib/utils'

interface CoreElementProps {
  body: string | null
  bodyVersion: number
  className?: string
}

export function CoreElement({ body, bodyVersion, className }: CoreElementProps) {
  return (
    <div className={cn('rounded-lg border border-border bg-card overflow-hidden', className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-border bg-muted/30">
        <div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-widest text-muted-foreground/70">
          <FileText className="w-3 h-3" />
          Core element
        </div>
        <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground/50">
          <span className="w-1.5 h-1.5 rounded-full bg-primary/60 inline-block" />
          v{bodyVersion} · {bodyVersion === 1 ? 'original' : 'synthesized'}
        </div>
      </div>

      {/* Body */}
      <div className="px-4 py-3.5">
        {body ? (
          <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed text-foreground/90">
            {body}
          </pre>
        ) : (
          <p className="text-sm text-muted-foreground/40 italic">
            No core element yet — add comments to start building the thread.
          </p>
        )}
      </div>

      {/* Version strip */}
      <div className="flex items-center gap-1.5 px-3.5 py-2 border-t border-border bg-muted/20">
        <span className="text-[10px] text-muted-foreground/40 mr-1">History:</span>
        {Array.from({ length: bodyVersion }, (_, i) => i + 1).map(v => (
          <span
            key={v}
            className={cn(
              'px-2 py-0.5 rounded-full text-[10px] font-medium',
              v === bodyVersion
                ? 'bg-primary/15 text-primary'
                : 'bg-muted text-muted-foreground/50'
            )}
          >
            v{v}
          </span>
        ))}
        {bodyVersion === 1 && (
          <span className="text-[10px] text-muted-foreground/30 ml-1">
            Next synthesis → v2
          </span>
        )}
      </div>
    </div>
  )
}
