import { cn } from '@/lib/utils'
import type { ThreadComment } from '@/lib/types'

interface CommentCardProps {
  comment: ThreadComment
}

function isAgent(author: string): boolean {
  return author.startsWith('agent:')
}

function avatarInitial(author: string): string {
  const name = author.startsWith('agent:') ? author.slice(6) : author
  return name.charAt(0).toUpperCase()
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })
}

export function CommentCard({ comment }: CommentCardProps) {
  const agent = isAgent(comment.author)
  const initial = avatarInitial(comment.author)

  return (
    <div
      className={cn(
        'rounded-lg border bg-card px-3.5 py-3 transition-opacity',
        comment.incorporated
          ? 'opacity-50 border-dashed border-border'
          : 'border-border'
      )}
    >
      <div className="flex items-center gap-2 mb-2">
        {/* Avatar */}
        <div
          className={cn(
            'w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-semibold shrink-0',
            agent
              ? 'bg-indigo-950/60 text-indigo-400'
              : 'bg-primary/20 text-primary'
          )}
        >
          {initial}
        </div>

        {/* Author */}
        <span
          className={cn(
            'text-xs font-semibold',
            agent ? 'text-indigo-400' : 'text-foreground'
          )}
        >
          {comment.author}
        </span>

        {/* Time */}
        <span className="text-[11px] text-muted-foreground/50">
          {formatTime(comment.created_at)}
        </span>

        {/* Incorporated badge */}
        {comment.incorporated && (
          <span className="ml-auto text-[10px] px-2 py-0.5 rounded-full bg-primary/10 text-primary/70">
            absorbed
          </span>
        )}
      </div>

      {/* Body */}
      <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed text-foreground/85">
        {comment.body}
      </pre>
    </div>
  )
}
