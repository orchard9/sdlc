import { MessageSquare, Plus } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ThreadSummary, ThreadStatus } from '@/lib/types'

interface ThreadListPaneProps {
  threads: ThreadSummary[]
  selectedSlug: string | null
  onSelect: (slug: string) => void
  onNewThread: () => void
}

function StatusBadge({ status }: { status: ThreadStatus }) {
  if (status === 'open') {
    return (
      <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-primary/10 text-primary">
        open
      </span>
    )
  }
  if (status === 'synthesized') {
    return (
      <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-indigo-950/50 text-indigo-400">
        synthesized
      </span>
    )
  }
  return (
    <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-muted text-muted-foreground/60">
      → ponder
    </span>
  )
}

export function ThreadListPane({ threads, selectedSlug, onSelect, onNewThread }: ThreadListPaneProps) {
  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-3.5 pt-4 pb-3 border-b border-border shrink-0">
        <div className="flex items-center gap-2 text-[15px] font-semibold mb-2.5">
          <MessageSquare className="w-4 h-4 opacity-70" />
          Threads
        </div>
        <button
          onClick={onNewThread}
          className="w-full flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-opacity"
        >
          <Plus className="w-3 h-3" />
          New thread
        </button>
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto py-2 px-1.5">
        {threads.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground/40 px-4 py-8 gap-2">
            <MessageSquare className="w-8 h-8 opacity-30" />
            <p className="text-xs">No threads yet</p>
            <p className="text-[11px]">Create one to start collaborating</p>
          </div>
        ) : (
          threads.map(thread => (
            <button
              key={thread.slug}
              onClick={() => onSelect(thread.slug)}
              className={cn(
                'w-full text-left px-2.5 py-2.5 rounded-lg mb-0.5 transition-colors border',
                selectedSlug === thread.slug
                  ? 'bg-accent border-border/60'
                  : 'border-transparent hover:bg-accent/50'
              )}
            >
              <p className="text-[13px] font-medium truncate mb-1 leading-snug">
                {thread.title}
              </p>
              <div className="flex items-center gap-2 text-[11px] text-muted-foreground/70">
                <StatusBadge status={thread.status} />
                <span>{thread.comment_count} {thread.comment_count === 1 ? 'comment' : 'comments'}</span>
                {thread.author && <span>{thread.author}</span>}
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  )
}
