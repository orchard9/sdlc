import { cn } from '@/lib/utils'

interface StatusBadgeProps {
  status: string
  className?: string
  testId?: string
}

const statusColors: Record<string, string> = {
  // Phases
  draft: 'bg-neutral-600 text-neutral-100',
  specified: 'bg-blue-600/80 text-blue-100',
  planned: 'bg-purple-600/80 text-purple-100',
  ready: 'bg-emerald-600/80 text-emerald-100',
  implementation: 'bg-amber-600/80 text-amber-100',
  review: 'bg-orange-600/80 text-orange-100',
  audit: 'bg-fuchsia-600/80 text-fuchsia-100',
  qa: 'bg-cyan-600/80 text-cyan-100',
  merge: 'bg-emerald-600/80 text-emerald-100',
  released: 'bg-emerald-700/80 text-emerald-100',
  // Statuses
  active: 'bg-emerald-600/80 text-emerald-100',
  verifying: 'bg-amber-600/80 text-amber-100',
  skipped: 'bg-neutral-600 text-neutral-300',
  // Artifact statuses
  missing: 'bg-neutral-700 text-neutral-400',
  approved: 'bg-emerald-600/80 text-emerald-100',
  rejected: 'bg-red-600/80 text-red-100',
  needs_fix: 'bg-amber-600/80 text-amber-100',
  passed: 'bg-emerald-600/80 text-emerald-100',
  failed: 'bg-red-600/80 text-red-100',
  waived: 'bg-neutral-600/80 text-neutral-300',
  // Task statuses
  pending: 'bg-neutral-600 text-neutral-200',
  in_progress: 'bg-blue-600/80 text-blue-100',
  completed: 'bg-emerald-600/80 text-emerald-100',
  blocked: 'bg-red-600/80 text-red-100',
  // Ponder statuses
  exploring: 'bg-violet-600/80 text-violet-100',
  converging: 'bg-amber-600/80 text-amber-100',
  committed: 'bg-emerald-600/80 text-emerald-100',
  parked: 'bg-neutral-600 text-neutral-300',
}

export function StatusBadge({ status, className, testId }: StatusBadgeProps) {
  const color = statusColors[status] ?? 'bg-neutral-600 text-neutral-200'
  return (
    <span
      data-testid={testId}
      className={cn('inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium', color, className)}
    >
      {status.replace(/_/g, ' ')}
    </span>
  )
}
