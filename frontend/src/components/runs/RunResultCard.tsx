import { CheckCircle2, XCircle, DollarSign, Hash } from 'lucide-react'
import type { PairedRunResult } from '@/lib/types'

interface RunResultCardProps {
  event: PairedRunResult
}

export function RunResultCard({ event }: RunResultCardProps) {
  const borderColor = event.isError ? 'border-red-500' : 'border-green-500'
  const bgColor = event.isError ? 'bg-red-500/5' : 'bg-green-500/5'

  return (
    <div className={`border-l-2 ${borderColor} ${bgColor} pl-3 py-2 rounded-r space-y-1`}>
      <div className="flex items-center gap-2 flex-wrap">
        {event.isError
          ? <XCircle className="w-3.5 h-3.5 text-red-400 shrink-0" />
          : <CheckCircle2 className="w-3.5 h-3.5 text-green-400 shrink-0" />
        }
        <span className={`text-xs font-medium ${event.isError ? 'text-red-400' : 'text-green-400'}`}>
          {event.isError ? 'Run failed' : 'Run completed'}
        </span>
        {event.cost_usd != null && (
          <span className="flex items-center gap-0.5 text-[10px] text-muted-foreground/70">
            <DollarSign className="w-2.5 h-2.5" />
            {event.cost_usd.toFixed(4)}
          </span>
        )}
        {event.turns != null && (
          <span className="flex items-center gap-0.5 text-[10px] text-muted-foreground/70">
            <Hash className="w-2.5 h-2.5" />
            {event.turns} turn{event.turns !== 1 ? 's' : ''}
          </span>
        )}
      </div>
      {event.text && event.text.trim().length > 0 && (
        <p className="text-xs text-muted-foreground/80 whitespace-pre-wrap leading-relaxed pl-5 line-clamp-4">
          {event.text}
        </p>
      )}
    </div>
  )
}
