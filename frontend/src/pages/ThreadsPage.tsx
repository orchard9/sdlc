import { useState, useEffect, useCallback } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { MessageSquare } from 'lucide-react'
import { api } from '@/api/client'
import type { ThreadSummary, ThreadDetail, ThreadComment } from '@/lib/types'
import { ThreadListPane } from '@/components/threads/ThreadListPane'
import { ThreadDetailPane } from '@/components/threads/ThreadDetailPane'
import { NewThreadModal } from '@/components/threads/NewThreadModal'

export function ThreadsPage() {
  const { slug } = useParams<{ slug?: string }>()
  const navigate = useNavigate()

  const [threads, setThreads] = useState<ThreadSummary[]>([])
  const [loadingList, setLoadingList] = useState(true)
  const [detail, setDetail] = useState<ThreadDetail | null>(null)
  const [loadingDetail, setLoadingDetail] = useState(false)
  const [createOpen, setCreateOpen] = useState(false)

  // Load thread list on mount
  useEffect(() => {
    setLoadingList(true)
    api.listThreads()
      .then(setThreads)
      .catch(() => {/* silently handle — API not yet available */})
      .finally(() => setLoadingList(false))
  }, [])

  // Load thread detail when slug changes
  useEffect(() => {
    if (!slug) {
      setDetail(null)
      return
    }
    setLoadingDetail(true)
    api.getThread(slug)
      .then(setDetail)
      .catch(() => setDetail(null))
      .finally(() => setLoadingDetail(false))
  }, [slug])

  const handleSelect = useCallback((threadSlug: string) => {
    navigate(`/threads/${threadSlug}`)
  }, [navigate])

  const handleCommentAdded = useCallback((comment: ThreadComment) => {
    setDetail(prev => {
      if (!prev) return prev
      return {
        ...prev,
        comments: [...prev.comments, comment],
        comment_count: prev.comment_count + 1,
      }
    })
    // Also update the count in the list
    setThreads(prev =>
      prev.map(t =>
        t.slug === slug
          ? { ...t, comment_count: t.comment_count + 1 }
          : t
      )
    )
  }, [slug])

  const handleCreateThread = useCallback(async (data: { title: string; body?: string }) => {
    const newThread = await api.createThread(data)
    setThreads(prev => [newThread, ...prev])
    setCreateOpen(false)
    navigate(`/threads/${newThread.slug}`)
  }, [navigate])

  return (
    <div className="flex h-full overflow-hidden">
      {/* Left pane: thread list */}
      <div className="w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]">
        {loadingList ? (
          <div className="flex-1 flex items-center justify-center text-muted-foreground/40 text-sm">
            Loading…
          </div>
        ) : (
          <ThreadListPane
            threads={threads}
            selectedSlug={slug ?? null}
            onSelect={handleSelect}
            onNewThread={() => setCreateOpen(true)}
          />
        )}
      </div>

      {/* Right pane: thread detail */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {loadingDetail ? (
          <div className="flex-1 flex items-center justify-center text-muted-foreground/40 text-sm">
            Loading thread…
          </div>
        ) : detail ? (
          <ThreadDetailPane
            detail={detail}
            onCommentAdded={handleCommentAdded}
          />
        ) : (
          <EmptyDetailState onNewThread={() => setCreateOpen(true)} />
        )}
      </div>

      {/* Create thread modal */}
      <NewThreadModal
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onSubmit={handleCreateThread}
      />
    </div>
  )
}

function EmptyDetailState({ onNewThread }: { onNewThread: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground/40 gap-3 px-8">
      <MessageSquare className="w-10 h-10 opacity-30" />
      <div>
        <p className="text-sm font-medium mb-1">Select a thread to view it</p>
        <p className="text-xs">Or{' '}
          <button
            onClick={onNewThread}
            className="underline underline-offset-2 hover:text-muted-foreground/70 transition-colors"
          >
            create a new thread
          </button>
          {' '}to start collaborating
        </p>
      </div>
    </div>
  )
}
