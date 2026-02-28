import { cn } from '@/lib/utils'
import type { InvestigationArtifact } from '@/lib/types'

interface Props {
  artifacts: InvestigationArtifact[]
  confidence: number | null
}

function confidenceBarColor(confidence: number): string {
  if (confidence >= 70) return 'bg-emerald-500'
  if (confidence >= 40) return 'bg-amber-500'
  return 'bg-red-500'
}

function confidenceTextColor(confidence: number): string {
  if (confidence >= 70) return 'text-emerald-400'
  if (confidence >= 40) return 'text-amber-400'
  return 'text-red-400'
}

export function SynthesisCard({ artifacts, confidence }: Props) {
  const synthesisArtifact = artifacts.find(a => a.filename === 'synthesis.md')

  const preview =
    synthesisArtifact?.content != null && synthesisArtifact.content.length > 0
      ? synthesisArtifact.content.length > 200
        ? synthesisArtifact.content.slice(0, 200) + '...'
        : synthesisArtifact.content
      : null

  return (
    <div className="px-3 py-3 space-y-2.5">
      {/* Header */}
      <span className="text-xs font-semibold text-muted-foreground/60 uppercase tracking-wider">
        Synthesis
      </span>

      {/* Confidence bar */}
      {confidence !== null && confidence !== undefined ? (
        <div className="space-y-1">
          <div className="flex items-center justify-between text-xs">
            <span className="text-muted-foreground/60">Hypothesis forming...</span>
            <span className={cn('tabular-nums font-medium', confidenceTextColor(confidence))}>
              {confidence}%
            </span>
          </div>
          <div className="h-1 rounded-full bg-muted/60 overflow-hidden">
            <div
              className={cn('h-full rounded-full transition-all', confidenceBarColor(confidence))}
              style={{ width: `${Math.min(100, Math.max(0, confidence))}%` }}
            />
          </div>
        </div>
      ) : (
        <p className="text-xs text-muted-foreground/50 italic">
          Hypothesis forming...
        </p>
      )}

      {/* Synthesis artifact preview */}
      {synthesisArtifact ? (
        <div className="space-y-1">
          <p className="text-xs font-mono text-muted-foreground/50">synthesis.md</p>
          {preview ? (
            <p className="text-xs text-foreground/70 leading-relaxed font-mono whitespace-pre-wrap break-words">
              {preview}
            </p>
          ) : (
            <p className="text-xs text-muted-foreground/40 italic">Empty file</p>
          )}
        </div>
      ) : (
        <p className="text-xs text-muted-foreground/40 italic leading-snug">
          Agent is synthesizing findings...
        </p>
      )}
    </div>
  )
}
