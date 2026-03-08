import { cn } from '@/lib/utils'
import type { GitStatus } from '@/hooks/useGitStatus'

interface GitDetailsPopoverProps {
  status: GitStatus
  collapsed: boolean
}

function severityDotClass(severity: 'green' | 'yellow' | 'red'): string {
  switch (severity) {
    case 'green': return 'bg-emerald-500'
    case 'yellow': return 'bg-amber-500'
    case 'red': return 'bg-red-500'
  }
}

function severityLabelClass(severity: 'green' | 'yellow' | 'red'): string {
  switch (severity) {
    case 'green': return 'text-emerald-500'
    case 'yellow': return 'text-amber-500'
    case 'red': return 'text-red-500'
  }
}

function severityExplanation(status: GitStatus): string {
  if (status.has_conflicts) return 'Merge conflicts'
  if (status.behind > 10) return 'Far behind upstream'
  if (status.dirty_count > 0) return 'Uncommitted changes'
  if (status.behind > 0) return 'Behind upstream'
  if (status.untracked_count > 5) return 'Many untracked files'
  if (status.ahead > 0) return 'Ahead of upstream'
  return 'Working tree clean'
}

function guidanceText(status: GitStatus): string {
  if (status.has_conflicts) return 'Resolve conflicts before continuing.'
  if (status.behind > 10) return 'Pull from upstream to catch up.'
  if (status.dirty_count > 0) return 'Stage and commit when ready.'
  if (status.behind > 0) return 'Pull from upstream to stay current.'
  if (status.ahead > 0) return 'Push to share your commits.'
  return 'All clear. Nothing to commit.'
}

interface StatusRowProps {
  dotClass: string
  count: number
  label: string
}

function StatusRow({ dotClass, count, label }: StatusRowProps) {
  if (count === 0) return null
  return (
    <div className="flex items-center gap-2 py-0.5 text-xs text-muted-foreground">
      <span className={cn('w-1.5 h-1.5 rounded-full shrink-0', dotClass)} />
      <span className="font-medium text-foreground min-w-[14px]">{count}</span>
      <span>{label}</span>
    </div>
  )
}

export function GitDetailsPopover({ status, collapsed }: GitDetailsPopoverProps) {
  const trackingParts: string[] = []
  if (status.ahead > 0) trackingParts.push(`${status.ahead} ahead`)
  if (status.behind > 0) trackingParts.push(`${status.behind} behind`)

  const hasStatusRows =
    status.dirty_count > 0 ||
    status.staged_count > 0 ||
    status.untracked_count > 0 ||
    status.conflict_count > 0

  return (
    <div
      className={cn(
        'absolute z-50 w-[260px] bg-popover border border-border rounded-lg shadow-lg',
        'animate-in fade-in-0 zoom-in-95 duration-150',
        collapsed
          ? 'left-full top-0 ml-2'
          : 'bottom-full left-0 mb-2',
      )}
    >
      {/* Branch & tracking */}
      <div className="px-3 py-2.5">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-medium text-foreground">{status.branch}</span>
          {trackingParts.length > 0 && (
            <span className="text-xs text-muted-foreground">{trackingParts.join(', ')}</span>
          )}
        </div>
      </div>

      {/* Status counts */}
      {hasStatusRows && (
        <div className="px-3 py-2 border-t border-border/50">
          <StatusRow dotClass="bg-red-500" count={status.conflict_count} label="conflicts" />
          <StatusRow dotClass="bg-amber-500" count={status.dirty_count} label="modified" />
          <StatusRow dotClass="bg-emerald-500" count={status.staged_count} label="staged" />
          <StatusRow dotClass="bg-muted-foreground/50" count={status.untracked_count} label="untracked" />
        </div>
      )}

      {/* Severity */}
      <div className="px-3 py-2 border-t border-border/50">
        <div className="flex items-center gap-1.5 text-xs">
          <span className={cn('w-1.5 h-1.5 rounded-full', severityDotClass(status.severity))} />
          <span className={cn('font-semibold uppercase tracking-wider text-[10px]', severityLabelClass(status.severity))}>
            {status.severity}
          </span>
          <span className="text-muted-foreground">{severityExplanation(status)}</span>
        </div>
      </div>

      {/* Guidance */}
      <div className="px-3 py-2 border-t border-border/50">
        <p className="text-xs text-muted-foreground italic">{guidanceText(status)}</p>
      </div>
    </div>
  )
}
