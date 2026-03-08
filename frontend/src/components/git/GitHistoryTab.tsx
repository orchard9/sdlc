import { useGitLog } from '@/hooks/useGitLog'
import { relativeTime } from '@/lib/relativeTime'
import { GitCommitHorizontal, RefreshCw, AlertCircle } from 'lucide-react'

function SkeletonRows() {
  return (
    <div className="flex flex-col">
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="flex items-center gap-3 px-3 py-2 border-b border-border">
          <div className="w-14 h-3.5 bg-accent rounded animate-pulse" />
          <div className="flex-1 h-3.5 bg-accent rounded animate-pulse" />
          <div className="w-16 h-3.5 bg-accent rounded animate-pulse" />
          <div className="w-12 h-3.5 bg-accent rounded animate-pulse" />
        </div>
      ))}
    </div>
  )
}

export function GitHistoryTab() {
  const { commits, loading, error, hasMore, loadMore, refetch } = useGitLog()

  if (loading && commits.length === 0) {
    return <SkeletonRows />
  }

  if (error === 'not_a_git_repo') {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <AlertCircle className="w-10 h-10 mb-3 opacity-30" />
        <p className="text-sm">Not a git repository</p>
      </div>
    )
  }

  if (error === 'not_available') {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <AlertCircle className="w-10 h-10 mb-3 opacity-30" />
        <p className="text-sm">Commit history not available yet</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <AlertCircle className="w-10 h-10 mb-3 opacity-30" />
        <p className="text-sm mb-3">Failed to load commit history</p>
        <button
          onClick={refetch}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border text-xs text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
        >
          <RefreshCw className="w-3 h-3" />
          Retry
        </button>
      </div>
    )
  }

  if (commits.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <GitCommitHorizontal className="w-10 h-10 mb-3 opacity-30" />
        <p className="text-sm">No commits yet</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col">
      {commits.map((commit) => (
        <div
          key={commit.hash}
          className="flex items-center gap-3 px-3 py-2 border-b border-border hover:bg-accent/50 transition-colors group"
        >
          <span className="font-mono text-xs text-muted-foreground shrink-0 w-14">
            {commit.short_hash}
          </span>
          <span className="text-sm truncate flex-1 text-foreground">
            {commit.subject}
          </span>
          <span className="text-xs text-muted-foreground shrink-0 hidden sm:block">
            {commit.author_name}
          </span>
          <span className="text-xs text-muted-foreground shrink-0 w-14 text-right">
            {relativeTime(commit.date)}
          </span>
        </div>
      ))}
      {hasMore && (
        <div className="flex justify-center py-4">
          <button
            onClick={loadMore}
            className="px-4 py-1.5 rounded-md border border-border text-xs text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
          >
            Load more commits
          </button>
        </div>
      )}
    </div>
  )
}
