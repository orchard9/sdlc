import { useEffect, useRef } from 'react'
import { cn } from '@/lib/utils'
import type { RunLine } from '@/hooks/useRunStream'

interface RunOutputProps {
  lines: RunLine[]
  running: boolean
  exitCode: number | null
  className?: string
}

export function RunOutput({ lines, running, exitCode, className }: RunOutputProps) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [lines.length])

  return (
    <div className={cn('bg-black/60 rounded-lg border border-border overflow-hidden', className)}>
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-card/50 text-xs">
        {running ? (
          <>
            <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
            <span className="text-muted-foreground">Running...</span>
          </>
        ) : exitCode !== null ? (
          <>
            <span className={cn('w-2 h-2 rounded-full', exitCode === 0 ? 'bg-emerald-400' : 'bg-red-400')} />
            <span className="text-muted-foreground">
              Exited with code {exitCode}
            </span>
          </>
        ) : (
          <span className="text-muted-foreground">Terminal</span>
        )}
      </div>
      <div className="p-3 max-h-96 overflow-y-auto font-mono text-xs leading-relaxed">
        {lines.map((line, i) => (
          <div
            key={i}
            className={cn(
              'whitespace-pre-wrap',
              line.type === 'stderr' ? 'text-red-400' : 'text-neutral-300'
            )}
          >
            {line.text}
          </div>
        ))}
        {lines.length === 0 && !running && (
          <span className="text-muted-foreground">No output yet</span>
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  )
}
