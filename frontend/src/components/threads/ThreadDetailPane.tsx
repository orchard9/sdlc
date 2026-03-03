import { useCallback, useRef, useState } from 'react'
import { Loader2, Send, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { api } from '@/api/client'
import type { ThreadDetail, ThreadComment, ThreadStatus } from '@/lib/types'
import { CoreElement } from './CoreElement'
import { CommentCard } from './CommentCard'

interface ThreadDetailPaneProps {
  detail: ThreadDetail
  onCommentAdded: (comment: ThreadComment) => void
  onDelete: () => void
  onStatusChange: (updated: ThreadDetail) => void
  onPromoted: (ponderSlug: string) => void
}

function StatusBadge({ status }: { status: ThreadStatus }) {
  if (status === 'open') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-primary/10 text-primary shrink-0">
        open
      </span>
    )
  }
  if (status === 'synthesized') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-indigo-950/50 text-indigo-400 shrink-0">
        synthesized
      </span>
    )
  }
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-muted text-muted-foreground/60 shrink-0">
      → ponder
    </span>
  )
}

function formatMeta(author: string, createdAt: string, commentCount: number): string {
  const d = new Date(createdAt)
  const dateStr = d.toLocaleString(undefined, { month: 'short', day: 'numeric' })
  const parts = [`opened by ${author}`, dateStr, `${commentCount} comment${commentCount !== 1 ? 's' : ''}`]
  return parts.join(' · ')
}

export function ThreadDetailPane({ detail, onCommentAdded, onDelete, onStatusChange, onPromoted }: ThreadDetailPaneProps) {
  const [draft, setDraft] = useState('')
  const [composing, setComposing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Delete state
  const [deleteConfirm, setDeleteConfirm] = useState(false)
  const [deleting, setDeleting] = useState(false)
  const [deleteError, setDeleteError] = useState<string | null>(null)

  // Synthesize state
  const [synthesizing, setSynthesizing] = useState(false)
  const [synthesizeError, setSynthesizeError] = useState<string | null>(null)

  // Promote state
  const [promoting, setPromoting] = useState(false)
  const [promoteError, setPromoteError] = useState<string | null>(null)

  const sendComment = useCallback(async () => {
    const body = draft.trim()
    if (!body || composing) return
    setComposing(true)
    setError(null)
    try {
      const comment = await api.addThreadComment(detail.slug, { author: 'jordan', body })
      onCommentAdded(comment)
      setDraft('')
      textareaRef.current?.focus()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to send comment')
    } finally {
      setComposing(false)
    }
  }, [draft, composing, detail.slug, onCommentAdded])

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      sendComment()
    }
  }

  async function handleDelete() {
    if (!deleteConfirm) {
      setDeleteConfirm(true)
      return
    }
    setDeleting(true)
    setDeleteError(null)
    try {
      await api.deleteThread(detail.slug)
      onDelete()
    } catch (e) {
      setDeleteError(e instanceof Error ? e.message : 'Failed to delete thread')
      setDeleting(false)
      setDeleteConfirm(false)
    }
  }

  async function handleSynthesize() {
    setSynthesizing(true)
    setSynthesizeError(null)
    try {
      const updated = await api.patchThread(detail.slug, { status: 'synthesized' })
      onStatusChange(updated)
    } catch (e) {
      setSynthesizeError(e instanceof Error ? e.message : 'Failed to synthesize')
    } finally {
      setSynthesizing(false)
    }
  }

  async function handlePromote() {
    setPromoting(true)
    setPromoteError(null)
    try {
      const result = await api.promoteThreadToPonder(detail.slug)
      onPromoted(result.ponder_slug)
    } catch (e) {
      setPromoteError(e instanceof Error ? e.message : 'Failed to promote')
      setPromoting(false)
    }
  }

  const unincorporatedCount = detail.comments.filter(c => !c.incorporated).length

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Thread header */}
      <div className="px-6 py-4 border-b border-border shrink-0">
        <div className="flex items-start justify-between gap-3 mb-1">
          <div className="flex items-start gap-2.5 min-w-0">
            <h1 className="text-lg font-semibold leading-snug">{detail.title}</h1>
            <StatusBadge status={detail.status} />
          </div>
          {/* Action buttons */}
          <div className="flex items-center gap-1.5 shrink-0">
            <button
              onClick={handleSynthesize}
              disabled={synthesizing || detail.status !== 'open'}
              title={detail.status !== 'open' ? 'Already synthesized' : 'Mark thread as synthesized'}
              className="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-indigo-950/30 text-indigo-400 hover:bg-indigo-950/50 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {synthesizing && <Loader2 className="w-3 h-3 animate-spin" />}
              Synthesize
            </button>
            <button
              onClick={handlePromote}
              disabled={promoting || detail.status === 'promoted'}
              title={detail.status === 'promoted' ? 'Already promoted to ponder' : 'Promote thread to a ponder entry'}
              className="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-primary/10 text-primary hover:bg-primary/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {promoting && <Loader2 className="w-3 h-3 animate-spin" />}
              Promote to Ponder
            </button>
            {deleteConfirm ? (
              <div className="flex items-center gap-1">
                <button
                  onClick={handleDelete}
                  disabled={deleting}
                  className="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-destructive/20 text-destructive hover:bg-destructive/30 transition-colors disabled:opacity-40"
                >
                  {deleting ? <Loader2 className="w-3 h-3 animate-spin" /> : null}
                  Delete?
                </button>
                <button
                  onClick={() => setDeleteConfirm(false)}
                  disabled={deleting}
                  className="px-2.5 py-1 rounded-md text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <button
                onClick={handleDelete}
                title="Delete thread"
                className="p-1.5 rounded-md text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-colors"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        </div>
        <p className="text-xs text-muted-foreground/60">
          {formatMeta(detail.author, detail.created_at, detail.comment_count)}
        </p>
        {(synthesizeError || promoteError || deleteError) && (
          <p className="mt-1.5 text-xs text-destructive">
            {synthesizeError || promoteError || deleteError}
          </p>
        )}
      </div>

      {/* Scrollable body */}
      <div className="flex-1 overflow-y-auto px-6 py-5 flex flex-col gap-5">
        {/* Core element */}
        <CoreElement body={detail.body} bodyVersion={detail.body_version} />

        {/* Comments divider */}
        <div className="flex items-center gap-2.5 text-[11px] font-medium text-muted-foreground/40">
          <div className="flex-1 h-px bg-border" />
          {detail.comments.length} comment{detail.comments.length !== 1 ? 's' : ''}
          {unincorporatedCount > 0 && (
            <span className="text-primary/50">· {unincorporatedCount} pending synthesis</span>
          )}
          <div className="flex-1 h-px bg-border" />
        </div>

        {/* Comments */}
        {detail.comments.length === 0 ? (
          <p className="text-sm text-center text-muted-foreground/35 py-4">
            No comments yet. Add the first one below.
          </p>
        ) : (
          <div className="flex flex-col gap-2.5">
            {detail.comments.map(comment => (
              <CommentCard key={comment.id} comment={comment} />
            ))}
          </div>
        )}

        {/* Bottom spacer so compose doesn't cover last comment */}
        <div className="h-2" />
      </div>

      {/* Compose area */}
      <div className={cn('px-6 py-3.5 border-t border-border shrink-0 bg-muted/10')}>
        {/* Author row */}
        <div className="flex items-center gap-2 mb-2 text-xs text-muted-foreground/60">
          <div className="w-4 h-4 rounded-full bg-primary/20 text-primary flex items-center justify-center text-[9px] font-semibold">
            J
          </div>
          Replying as <strong className="text-foreground/80">jordan</strong>
        </div>

        {/* Input */}
        <div className="relative">
          <textarea
            ref={textareaRef}
            value={draft}
            onChange={e => setDraft(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add a comment… ⌘↵ to send"
            rows={3}
            className="w-full resize-none rounded-lg border border-border bg-card px-3 py-2.5 pr-24 text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:ring-1 focus:ring-ring leading-relaxed"
          />
          <div className="absolute bottom-2.5 right-2.5 flex items-center gap-1.5">
            <span className="text-[11px] text-muted-foreground/35">⌘↵</span>
            <button
              onClick={sendComment}
              disabled={composing || !draft.trim()}
              className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {composing ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : (
                <Send className="w-3 h-3" />
              )}
              Send
            </button>
          </div>
        </div>

        {error && (
          <p className="mt-1.5 text-xs text-destructive">{error}</p>
        )}
      </div>
    </div>
  )
}
