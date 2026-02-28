import { useCallback, useEffect, useRef, useState } from 'react'
import { Send, Square, Loader2 } from 'lucide-react'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { OrientationStrip } from './OrientationStrip'
import { TeamRow } from './TeamRow'
import { SessionBlock } from './SessionBlock'
import type {
  PonderDetail,
  PonderRunState,
  PonderSseEvent,
  SessionContent,
  SessionMeta,
} from '@/lib/types'

// ---------------------------------------------------------------------------
// MCP call card — shown while agent is running
// ---------------------------------------------------------------------------

function McpCallCard({
  slug,
  message,
  session,
}: {
  slug: string
  message: string | null
  session: number
}) {
  return (
    <div className="my-4 border border-primary/20 rounded-lg overflow-hidden text-xs">
      <div className="flex items-center justify-between px-3 py-1.5 bg-primary/5 border-b border-primary/20">
        <span className="font-mono font-semibold text-primary/80">sdlc_ponder_chat</span>
        <span className="flex items-center gap-1.5 text-primary/60">
          <span className="w-1.5 h-1.5 rounded-full bg-primary/60 animate-pulse" />
          live
        </span>
      </div>
      <div className="px-3 py-2 space-y-1 font-mono text-muted-foreground/70">
        <div className="flex gap-4">
          <span className="w-16 shrink-0 text-muted-foreground/40">slug</span>
          <span>{slug}</span>
        </div>
        {message && (
          <div className="flex gap-4">
            <span className="w-16 shrink-0 text-muted-foreground/40">message</span>
            <span className="truncate max-w-xs">&ldquo;{message}&rdquo;</span>
          </div>
        )}
        <div className="flex gap-4">
          <span className="w-16 shrink-0 text-muted-foreground/40">session</span>
          <span>{session}</span>
        </div>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Working placeholder shown under the new session divider
// ---------------------------------------------------------------------------

function WorkingPlaceholder() {
  return (
    <div className="flex items-center gap-2 py-6 text-sm text-muted-foreground/50">
      <Loader2 className="w-3.5 h-3.5 animate-spin" />
      <span>agent working...</span>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Input bar
// ---------------------------------------------------------------------------

function InputBar({
  runState,
  onSend,
  onStop,
}: {
  runState: PonderRunState
  onSend: (message: string) => void
  onStop: () => void
}) {
  const [value, setValue] = useState('')
  const running = runState.status === 'running'

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (running || !value.trim() && runState.status !== 'idle') return
    onSend(value.trim())
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSubmit(e as unknown as React.FormEvent)
    }
  }

  if (running) {
    return (
      <div className="flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
        <div className="flex-1 px-3 py-2 text-xs text-muted-foreground/40 bg-muted/40 border border-border/50 rounded-lg">
          session in progress...
        </div>
        <button
          onClick={onStop}
          className="shrink-0 flex items-center gap-1.5 px-3 py-2 text-xs text-destructive/70 hover:text-destructive border border-destructive/30 hover:border-destructive/60 rounded-lg transition-colors"
          title="Stop session"
        >
          <Square className="w-3 h-3" />
          stop
        </button>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
      <textarea
        value={value}
        onChange={e => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Add a thought, constraint, or question..."
        rows={1}
        className="flex-1 px-3 py-2 text-sm bg-muted/40 border border-border/60 rounded-lg outline-none focus:border-primary/40 transition-colors placeholder:text-muted-foreground/40 resize-none leading-relaxed"
        style={{ minHeight: '2.5rem', maxHeight: '8rem' }}
        onInput={e => {
          const t = e.currentTarget
          t.style.height = 'auto'
          t.style.height = `${Math.min(t.scrollHeight, 128)}px`
        }}
      />
      <button
        type="submit"
        className="shrink-0 p-2 rounded-lg bg-primary/90 hover:bg-primary text-primary-foreground disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        title="Send"
      >
        <Send className="w-4 h-4" />
      </button>
    </form>
  )
}

// ---------------------------------------------------------------------------
// Main DialoguePanel
// ---------------------------------------------------------------------------

interface Props {
  entry: PonderDetail
  onRefresh: () => void
}

export function DialoguePanel({ entry, onRefresh }: Props) {
  const { slug } = entry
  const [sessions, setSessions] = useState<SessionContent[]>([])
  const [loading, setLoading] = useState(true)
  const [runState, setRunState] = useState<PonderRunState>({ status: 'idle' })
  const [pendingMessage, setPendingMessage] = useState<{
    text: string | null
    ownerName: string
    session: number
  } | null>(null)

  const streamRef = useRef<HTMLDivElement>(null)
  const userScrolledUp = useRef(false)

  // ------------------------------------------------------------------
  // Session loading
  // ------------------------------------------------------------------

  const loadSessions = useCallback(async () => {
    try {
      const metas: SessionMeta[] = await api.getPonderSessions(slug)
      const contents = await Promise.all(
        metas.map(m => api.getPonderSession(slug, m.session))
      )
      setSessions(contents)
    } catch {
      // non-fatal
    } finally {
      setLoading(false)
    }
  }, [slug])

  useEffect(() => {
    setLoading(true)
    setSessions([])
    setRunState({ status: 'idle' })
    setPendingMessage(null)
    loadSessions()
  }, [loadSessions])

  // ------------------------------------------------------------------
  // SSE — ponder-specific events + generic state updates
  // ------------------------------------------------------------------

  const handlePonderEvent = useCallback(
    (event: PonderSseEvent) => {
      if (event.slug !== slug) return
      if (event.type === 'ponder_run_started') {
        // run started is handled optimistically in handleSend
      } else if (event.type === 'ponder_run_completed') {
        setRunState({ status: 'idle' })
        setPendingMessage(null)
        loadSessions()
        onRefresh()
      } else if (event.type === 'ponder_run_stopped') {
        setRunState({ status: 'idle' })
        setPendingMessage(null)
      }
    },
    [slug, loadSessions, onRefresh],
  )

  const handleUpdate = useCallback(() => {
    // Only refresh sessions if not mid-run (avoid flash)
    if (runState.status === 'idle') {
      loadSessions()
      onRefresh()
    }
  }, [runState.status, loadSessions, onRefresh])

  useSSE(handleUpdate, handlePonderEvent)

  // ------------------------------------------------------------------
  // Auto-scroll
  // ------------------------------------------------------------------

  useEffect(() => {
    const el = streamRef.current
    if (!el || userScrolledUp.current) return
    el.scrollTop = el.scrollHeight
  }, [sessions, pendingMessage, runState])

  const handleScroll = useCallback(() => {
    const el = streamRef.current
    if (!el) return
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60
    userScrolledUp.current = !atBottom
  }, [])

  // ------------------------------------------------------------------
  // Send / Stop
  // ------------------------------------------------------------------

  const handleSend = useCallback(
    async (message: string) => {
      try {
        const res = await api.startPonderChat(slug, message || undefined)
        if (res.status === 'conflict') return
        setRunState({
          status: 'running',
          session: res.session,
          ownerName: res.owner_name,
          ownerMessage: message || null,
        })
        setPendingMessage({
          text: message || null,
          ownerName: res.owner_name,
          session: res.session,
        })
        userScrolledUp.current = false
      } catch {
        // silently fail — SSE will eventually resolve
      }
    },
    [slug],
  )

  const handleStop = useCallback(async () => {
    await api.stopPonderChat(slug).catch(() => {})
    setRunState({ status: 'idle' })
    setPendingMessage(null)
  }, [slug])

  // ------------------------------------------------------------------
  // Render
  // ------------------------------------------------------------------

  const isRunning = runState.status === 'running'
  const orientation = entry.orientation ?? (sessions.at(-1)?.orientation ?? null)

  return (
    <div className="h-full flex flex-col min-h-0">
      {/* Team row */}
      {entry.team.length > 0 && (
        <div className="shrink-0 px-5 pt-4 pb-3 border-b border-border/50">
          <TeamRow team={entry.team} />
        </div>
      )}

      {/* Orientation strip */}
      <div className="shrink-0 px-5 py-3 border-b border-border/50">
        <OrientationStrip orientation={orientation} />
      </div>

      {/* Stream */}
      <div
        ref={streamRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto px-5 py-4 min-h-0"
      >
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="w-4 h-4 animate-spin text-muted-foreground/40" />
          </div>
        ) : sessions.length === 0 && !pendingMessage ? (
          /* Empty state */
          <div className="flex flex-col items-center justify-center h-full text-center gap-3">
            <p className="text-sm text-muted-foreground/60">No sessions yet.</p>
            <p className="text-xs text-muted-foreground/40 max-w-xs">
              The agent will interview this idea, recruit thought partners, and write the
              dialogue here.
            </p>
            <p className="text-xs text-muted-foreground/30 mt-1">
              Add a seed thought below or just hit send.
            </p>
          </div>
        ) : (
          <>
            {sessions.map(s => (
              <SessionBlock
                key={s.session}
                session={s}
                ownerName={isRunning && runState.status === 'running' ? runState.ownerName : undefined}
                artifacts={entry.artifacts}
              />
            ))}

            {/* Pending session: optimistic owner message + MCP card + working indicator */}
            {pendingMessage && (
              <div>
                <div className="flex items-center gap-3 mb-5">
                  <div className="flex-1 h-px bg-border/40" />
                  <span className="text-xs text-muted-foreground/50 font-medium whitespace-nowrap">
                    Session {pendingMessage.session}  ·  just now
                  </span>
                  <div className="flex-1 h-px bg-border/40" />
                </div>

                {pendingMessage.text && (
                  <div className="my-3 border border-border/50 rounded-lg px-4 py-3 bg-muted/20">
                    <div className="flex items-baseline gap-2 mb-1.5">
                      <span className="text-sm font-bold text-primary">{pendingMessage.ownerName}</span>
                      <span className="text-xs text-muted-foreground/50">·</span>
                      <span className="text-xs font-medium text-primary/70">Owner</span>
                    </div>
                    <p className="text-sm text-foreground/80 leading-relaxed">{pendingMessage.text}</p>
                  </div>
                )}

                <McpCallCard
                  slug={slug}
                  message={pendingMessage.text}
                  session={pendingMessage.session}
                />

                <WorkingPlaceholder />
              </div>
            )}
          </>
        )}
      </div>

      {/* Input bar */}
      <InputBar
        runState={runState}
        onSend={handleSend}
        onStop={handleStop}
      />
    </div>
  )
}
