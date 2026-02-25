import { cn } from '@/lib/utils'

interface SkeletonProps {
  className?: string
  /** Number of skeleton lines to render. Defaults to 1. */
  lines?: number
  /** Optional width class (e.g. "w-1/2", "w-32"). Applied to each line. */
  width?: string
}

/** Reusable animated placeholder block for loading states. */
export function Skeleton({ className, lines = 1, width }: SkeletonProps) {
  if (lines === 1) {
    return (
      <div
        className={cn(
          'animate-pulse rounded-md bg-white/5 h-4',
          width,
          className,
        )}
      />
    )
  }

  return (
    <div className={cn('space-y-2.5', className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'animate-pulse rounded-md bg-white/5 h-4',
            // Make the last line shorter for a more natural look
            i === lines - 1 && !width ? 'w-2/3' : width,
          )}
        />
      ))}
    </div>
  )
}

/** Card-shaped skeleton matching FeatureCard layout. */
export function SkeletonCard({ className }: { className?: string }) {
  return (
    <div className={cn('bg-card border border-border rounded-xl p-4', className)}>
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 space-y-2">
          <Skeleton width="w-3/4" />
          <Skeleton width="w-1/3" className="h-3" />
        </div>
        <Skeleton width="w-16" className="h-5 rounded-md" />
      </div>
      <Skeleton className="mt-3 h-1.5 rounded-full" />
      <Skeleton width="w-1/2" className="mt-3 h-3" />
      <Skeleton width="w-1/3" className="mt-2 h-3" />
    </div>
  )
}

/** Milestone-row skeleton matching MilestonesPage card layout. */
export function SkeletonMilestone({ className }: { className?: string }) {
  return (
    <div className={cn('bg-card border border-border rounded-xl p-4', className)}>
      <div className="flex items-center gap-2 mb-2">
        <Skeleton width="w-1/3" />
        <Skeleton width="w-14" className="h-5 rounded-md" />
      </div>
      <Skeleton width="w-20" className="h-3" />
      <div className="flex gap-1.5 mt-2">
        <Skeleton width="w-16" className="h-5 rounded" />
        <Skeleton width="w-20" className="h-5 rounded" />
        <Skeleton width="w-14" className="h-5 rounded" />
      </div>
    </div>
  )
}

/** Skeleton mimicking the FeatureDetail page header + artifacts. */
export function SkeletonFeatureDetail({ className }: { className?: string }) {
  return (
    <div className={cn('max-w-4xl mx-auto', className)}>
      {/* Back link */}
      <Skeleton width="w-14" className="h-4 mb-4" />

      {/* Title row */}
      <div className="flex items-start justify-between gap-4 mb-4">
        <div className="flex-1 space-y-2">
          <Skeleton width="w-1/2" className="h-6" />
          <Skeleton width="w-1/4" className="h-3" />
          <Skeleton width="w-2/3" className="h-3" />
        </div>
        <Skeleton width="w-20" className="h-6 rounded-md" />
      </div>

      {/* Phase progress bar */}
      <Skeleton className="h-1.5 rounded-full mb-6" />

      {/* Next action card */}
      <div className="bg-card border border-border rounded-xl p-4 mb-6">
        <div className="flex items-center justify-between">
          <div className="space-y-2 flex-1">
            <Skeleton width="w-1/3" />
            <Skeleton width="w-1/2" className="h-3" />
          </div>
          <Skeleton width="w-16" className="h-8 rounded-lg" />
        </div>
      </div>

      {/* Artifacts section */}
      <div className="mb-6">
        <Skeleton width="w-16" className="h-4 mb-3" />
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="border border-border rounded-lg overflow-hidden">
              <div className="flex items-center justify-between px-3 py-2 bg-card/50 border-b border-border">
                <div className="flex items-center gap-2">
                  <Skeleton width="w-3.5" className="h-3.5 rounded" />
                  <Skeleton width="w-20" />
                  <Skeleton width="w-12" className="h-5 rounded-md" />
                </div>
              </div>
              <div className="p-4">
                <Skeleton lines={3} className="h-3" />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
