import { cn } from '@/lib/utils'

interface StatusBadgeProps {
  status: string
  className?: string
}

function statusColor(status: string): string {
  switch (status) {
    case 'M': return 'text-amber-500'
    case 'A': return 'text-emerald-500'
    case 'D': return 'text-red-500'
    case 'R':
    case 'C': return 'text-blue-500'
    case '??': return 'text-muted-foreground'
    default: return 'text-muted-foreground'
  }
}

function statusLabel(status: string): string {
  if (status === '??') return '?'
  if (status === '!!') return '!'
  // For compound statuses like "AM", just take the first letter
  return status.charAt(0)
}

export function StatusBadge({ status, className }: StatusBadgeProps) {
  return (
    <span
      className={cn(
        'inline-block w-4 text-center font-mono text-xs font-semibold shrink-0',
        statusColor(status),
        className,
      )}
      title={statusTitle(status)}
    >
      {statusLabel(status)}
    </span>
  )
}

function statusTitle(status: string): string {
  switch (status) {
    case 'M': return 'Modified'
    case 'A': return 'Added'
    case 'D': return 'Deleted'
    case 'R': return 'Renamed'
    case 'C': return 'Copied'
    case '??': return 'Untracked'
    case '!!': return 'Ignored'
    default: return status
  }
}
