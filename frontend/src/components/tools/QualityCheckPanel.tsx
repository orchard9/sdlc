import { useState } from 'react'
import { Check, X, ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { QualityCheckData, CheckResult } from '@/lib/types'

interface CheckRowProps {
  check: CheckResult
}

function CheckRow({ check }: CheckRowProps) {
  const [expanded, setExpanded] = useState(check.status === 'failed')
  const passed = check.status === 'passed'

  return (
    <div className={cn(
      'border rounded-lg overflow-hidden',
      passed ? 'border-emerald-500/20' : 'border-red-500/30',
    )}>
      <button
        onClick={() => setExpanded(prev => !prev)}
        className={cn(
          'w-full flex items-center gap-2.5 px-3 py-2 text-left transition-colors',
          passed ? 'bg-emerald-500/5 hover:bg-emerald-500/10' : 'bg-red-500/5 hover:bg-red-500/10',
        )}
      >
        <span className={cn(
          'shrink-0 w-4 h-4 rounded-full flex items-center justify-center',
          passed ? 'bg-emerald-500/20' : 'bg-red-500/20',
        )}>
          {passed
            ? <Check className="w-2.5 h-2.5 text-emerald-400" />
            : <X className="w-2.5 h-2.5 text-red-400" />
          }
        </span>
        <span className="flex-1 text-sm font-medium truncate">{check.name}</span>
        <span className="text-xs text-muted-foreground shrink-0">{check.duration_ms}ms</span>
        {check.output && (
          expanded
            ? <ChevronDown className="w-3.5 h-3.5 shrink-0 text-muted-foreground" />
            : <ChevronRight className="w-3.5 h-3.5 shrink-0 text-muted-foreground" />
        )}
      </button>
      {expanded && check.output && (
        <pre className="px-3 py-2 text-xs font-mono bg-card text-muted-foreground overflow-x-auto whitespace-pre-wrap leading-relaxed border-t border-border/50">
          {check.output}
        </pre>
      )}
    </div>
  )
}

interface QualityCheckPanelProps {
  data: QualityCheckData
}

export function QualityCheckPanel({ data }: QualityCheckPanelProps) {
  if (data.checks.length === 0) {
    return (
      <p className="text-sm text-muted-foreground italic">
        No checks ran. Add platform commands under <code className="font-mono text-xs">platform.commands</code> in <code className="font-mono text-xs">.sdlc/config.yaml</code>.
      </p>
    )
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-3 mb-3">
        {data.passed > 0 && (
          <span className="flex items-center gap-1 text-sm font-medium text-emerald-400">
            <Check className="w-4 h-4" />
            {data.passed} passed
          </span>
        )}
        {data.failed > 0 && (
          <span className="flex items-center gap-1 text-sm font-medium text-red-400">
            <X className="w-4 h-4" />
            {data.failed} failed
          </span>
        )}
      </div>
      {data.checks.map((check, i) => (
        <CheckRow key={`${check.name}-${i}`} check={check} />
      ))}
    </div>
  )
}
