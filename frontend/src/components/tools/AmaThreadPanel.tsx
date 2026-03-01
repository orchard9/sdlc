import { useEffect, useRef, useState } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Telescope, Zap, Send, Sparkles, MessageSquare, Plus, Loader2, X } from 'lucide-react'
import { api } from '@/api/client'
import { cn } from '@/lib/utils'
import { AmaResultPanel } from '@/components/tools/AmaResultPanel'
import { AmaAnswerPanel } from '@/components/tools/AmaAnswerPanel'
import { ToolResultActions } from '@/components/tools/ToolResultActions'
import { ThreadToPonderModal } from '@/components/shared/ThreadToPonderModal'
import { FixRightAwayModal } from '@/components/shared/FixRightAwayModal'
import type { AmaThreadTurn, AmaThread, ToolMeta, AmaData } from '@/lib/types'

interface AmaThreadPanelProps {
  tool: ToolMeta
}

// ---------------------------------------------------------------------------
// Thread helpers
// ---------------------------------------------------------------------------

function buildThreadContext(turns: AmaThreadTurn[]): string {
  return turns
    .filter(t => t.synthesisText)
    .map((t, i) => `Q${i + 1}: ${t.question}\n${t.synthesisText}`)
    .join('\n\n')
}

function buildThreadMarkdown(turns: AmaThreadTurn[]): string {
  const lines: string[] = ['# AMA Thread\n']
  for (const [i, turn] of turns.entries()) {
    lines.push(`## Q${i + 1}: ${turn.question}\n`)
    if (turn.sources.length > 0) {
      lines.push('**Sources:**')
      for (const s of turn.sources) {
        lines.push(`- \`${s.path}:${s.lines[0]}–${s.lines[1]}\` (${Math.round(s.score * 100)}%)`)
      }
      lines.push('')
    }
    if (turn.synthesisText) {
      lines.push('**Synthesis:**\n')
      lines.push(turn.synthesisText)
      lines.push('')
    }
  }
  return lines.join('\n')
}

function buildFixDescription(turns: AmaThreadTurn[]): string {
  const last = turns[turns.length - 1]
  if (!last) return ''
  const allPaths = [...new Set(turns.flatMap(t => t.sources.map(s => s.path)))]
  const parts: string[] = []
  if (last.synthesisText) {
    parts.push(last.synthesisText.trim())
  }
  if (allPaths.length > 0) {
    parts.push('\n\nRelated files:\n' + allPaths.map(p => `- ${p}`).join('\n'))
  }
  return parts.join('')
}

// ---------------------------------------------------------------------------
// Turn card
// ---------------------------------------------------------------------------

interface TurnCardProps {
  turn: AmaThreadTurn
  turnIndex: number
  onSynthesisDone: (text: string) => void
}

function TurnCard({ turn, turnIndex, onSynthesisDone }: TurnCardProps) {
  return (
    <div className="space-y-3 border border-border/50 rounded-xl p-4 bg-muted/10">
      {/* Question */}
      <div className="flex items-start gap-2">
        <span className="text-[10px] font-mono text-muted-foreground/50 mt-0.5 shrink-0">
          Q{turnIndex + 1}
        </span>
        <p className="text-sm font-medium leading-snug">{turn.question}</p>
      </div>

      {/* Sources */}
      <AmaResultPanel data={{ sources: turn.sources }} />

      {/* Synthesis */}
      {turn.synthesisText !== null ? (
        // Already completed — show text directly (loaded from server or streaming done)
        <div className="mt-4 space-y-2">
          <div className="flex items-center gap-2 border-t border-border/50 pt-4">
            <Sparkles className="w-3.5 h-3.5 text-primary shrink-0" />
            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Synthesis
            </span>
          </div>
          <div className="text-sm text-foreground leading-relaxed [&_p]:mb-2 [&_p:last-child]:mb-0 [&_code]:text-xs [&_code]:font-mono [&_code]:bg-muted/60 [&_code]:border [&_code]:border-border/50 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_ul]:list-disc [&_ul]:pl-5 [&_ul]:space-y-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_ol]:space-y-1 [&_strong]:font-semibold">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{turn.synthesisText}</ReactMarkdown>
          </div>
        </div>
      ) : turn.synthesisRunKey ? (
        // Streaming — use AmaAnswerPanel
        <AmaAnswerPanel runKey={turn.synthesisRunKey} onDone={onSynthesisDone} />
      ) : null}
    </div>
  )
}

// ---------------------------------------------------------------------------
// AmaThreadPanel
// ---------------------------------------------------------------------------

export function AmaThreadPanel({ tool }: AmaThreadPanelProps) {
  const [turns, setTurns] = useState<AmaThreadTurn[]>([])
  const [threadId, setThreadId] = useState<string | null>(null)
  const [threads, setThreads] = useState<AmaThread[]>([])
  const [threadsLoading, setThreadsLoading] = useState(true)
  const [question, setQuestion] = useState('')
  const [followUp, setFollowUp] = useState('')
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const [ponderOpen, setPonderOpen] = useState(false)
  const [fixOpen, setFixOpen] = useState(false)

  const followUpRef = useRef<HTMLInputElement>(null)

  const selectThread = async (id: string) => {
    setThreadId(id)
    setSearchError(null)
    try {
      const detail = await api.getAmaThread(id)
      const loadedTurns: AmaThreadTurn[] = detail.turns.map(t => ({
        question: t.question,
        sources: t.sources,
        synthesisRunKey: t.run_id ?? '',
        synthesisText: t.synthesis ?? null,
        timestamp: t.created_at,
      }))
      setTurns(loadedTurns)
    } catch {
      // non-fatal
    } finally {
      setThreadsLoading(false)
    }
  }

  // Load thread list on mount
  useEffect(() => {
    api.listAmaThreads()
      .then(list => {
        setThreads(list)
        if (list.length > 0) {
          selectThread(list[0].id)
        } else {
          setThreadsLoading(false)
        }
      })
      .catch(() => setThreadsLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const refreshThreadList = () => {
    api.listAmaThreads().then(list => setThreads(list)).catch(() => {})
  }

  const lastTurn = turns[turns.length - 1] ?? null
  const lastDone = lastTurn?.synthesisText !== null && lastTurn?.synthesisText !== undefined
  const threadReady = turns.length > 0 && lastDone

  const totalSources = new Set(turns.flatMap(t => t.sources.map(s => s.path))).size

  const updateTurnSynthesis = (index: number, text: string) => {
    setTurns(prev => prev.map((t, i) => i === index ? { ...t, synthesisText: text } : t))
    // Persist synthesis to server
    if (threadId) {
      api.updateAmaThreadTurnSynthesis(threadId, index, text).catch(() => {})
    }
  }

  const askQuestion = async (q: string) => {
    if (!q.trim() || searching) return
    setSearchError(null)
    setSearching(true)

    try {
      const res = await api.runTool(tool.name, { question: q.trim() })
      if (!res.ok) {
        setSearchError(res.error ?? 'Search failed')
        setSearching(false)
        return
      }

      const amaData = res.data as AmaData
      const turnIndex = turns.length
      const priorContext = buildThreadContext(turns)

      let synthesisRunKey = ''
      let newThreadId = threadId
      try {
        const answerResp = await api.answerAma(
          q.trim(),
          amaData.sources,
          {
            turnIndex,
            threadContext: priorContext || undefined,
            threadId: threadId ?? undefined,
          }
        )
        synthesisRunKey = answerResp.run_key
        // answer_ama creates thread transparently and returns thread_id
        if (answerResp.thread_id && !newThreadId) {
          newThreadId = answerResp.thread_id
          setThreadId(answerResp.thread_id)
          refreshThreadList()
        }
      } catch {
        // synthesis failed to start — non-fatal
      }

      const newTurn: AmaThreadTurn = {
        question: q.trim(),
        sources: amaData.sources,
        synthesisRunKey,
        synthesisText: null,
        timestamp: new Date().toISOString(),
      }

      setTurns(prev => [...prev, newTurn])
    } catch (e) {
      setSearchError(e instanceof Error ? e.message : 'Search failed')
    } finally {
      setSearching(false)
    }
  }

  const handleInitialAsk = () => {
    const q = question.trim()
    if (!q) return
    setQuestion('')
    askQuestion(q)
  }

  const handleFollowUp = () => {
    const q = followUp.trim()
    if (!q) return
    setFollowUp('')
    askQuestion(q)
  }

  const handleNewThread = () => {
    setTurns([])
    setThreadId(null)
    setQuestion('')
    setFollowUp('')
    setSearchError(null)
  }

  const handleDeleteThread = async (id: string) => {
    try {
      await api.deleteAmaThread(id)
      const newList = threads.filter(t => t.id !== id)
      setThreads(newList)
      if (id === threadId) {
        if (newList.length > 0) {
          selectThread(newList[0].id)
        } else {
          handleNewThread()
        }
      }
    } catch {
      // non-fatal
    }
  }

  if (threadsLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex gap-4 min-h-[320px]">
      {/* Left pane: thread list */}
      <div className="w-[180px] shrink-0 flex flex-col gap-1 border-r border-border/50 pr-3">
        <button
          onClick={handleNewThread}
          className="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-muted/40 rounded-lg transition-colors"
        >
          <Plus className="w-3 h-3" />
          New Thread
        </button>

        {threads.length === 0 && (
          <p className="text-[10px] text-muted-foreground/50 px-2 mt-2">No threads yet</p>
        )}

        {threads.map(t => (
          <div
            key={t.id}
            className={cn(
              'group relative flex items-start gap-1.5 px-2 py-1.5 rounded-lg cursor-pointer transition-colors',
              t.id === threadId
                ? 'bg-primary/10 text-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-muted/40',
            )}
            onClick={() => { if (t.id !== threadId) selectThread(t.id) }}
          >
            <MessageSquare className="w-3 h-3 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-[11px] font-medium leading-snug line-clamp-2">{t.title}</p>
              <p className="text-[10px] text-muted-foreground/60 mt-0.5">
                {t.turn_count} turn{t.turn_count !== 1 ? 's' : ''}
              </p>
            </div>
            <button
              onClick={e => { e.stopPropagation(); handleDeleteThread(t.id) }}
              className="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-red-500/20 hover:text-red-400 transition-all shrink-0 mt-0.5"
              title="Delete thread"
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        ))}
      </div>

      {/* Right pane: active thread or initial question input */}
      <div className="flex-1 min-w-0 space-y-4">
        {threadId === null ? (
          // No thread selected — show initial question input
          <>
            <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Question
            </label>
            <input
              type="text"
              value={question}
              onChange={e => setQuestion(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter' && !searching) handleInitialAsk() }}
              placeholder="e.g. Where is JWT validation?"
              disabled={searching}
              className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50 disabled:opacity-60"
            />
            {searchError && <p className="text-xs text-destructive">{searchError}</p>}
            <button
              onClick={handleInitialAsk}
              disabled={!question.trim() || searching}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {searching
                ? <span className="w-4 h-4 border-2 border-primary-foreground/30 border-t-primary-foreground rounded-full animate-spin" />
                : <Send className="w-4 h-4" />
              }
              {searching ? 'Searching…' : 'Ask'}
            </button>
          </>
        ) : (
          // Thread view: turns + follow-up + action bar
          <>
            {/* Thread header */}
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground font-medium">
                {turns.length} question{turns.length !== 1 ? 's' : ''} · {totalSources} source{totalSources !== 1 ? 's' : ''}
              </span>
            </div>

            {/* Turn cards */}
            {turns.map((turn, i) => (
              <TurnCard
                key={turn.timestamp + i}
                turn={turn}
                turnIndex={i}
                onSynthesisDone={text => updateTurnSynthesis(i, text)}
              />
            ))}

            {/* Follow-up input — shown when last synthesis is done */}
            {lastDone && !searching && (
              <div className="flex items-center gap-2">
                <input
                  ref={followUpRef}
                  type="text"
                  value={followUp}
                  onChange={e => setFollowUp(e.target.value)}
                  onKeyDown={e => { if (e.key === 'Enter' && !searching) handleFollowUp() }}
                  placeholder="Follow up…"
                  className="flex-1 px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50"
                />
                <button
                  onClick={handleFollowUp}
                  disabled={!followUp.trim() || searching}
                  className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors shrink-0"
                >
                  <Send className="w-3.5 h-3.5" />
                  Ask
                </button>
              </div>
            )}

            {/* Searching spinner for follow-up */}
            {searching && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span className="w-3.5 h-3.5 border-2 border-muted-foreground/30 border-t-primary rounded-full animate-spin" />
                Searching…
              </div>
            )}

            {searchError && <p className="text-xs text-destructive">{searchError}</p>}

            {/* Thread action bar */}
            {threadReady && (
              <ToolResultActions
                actions={[
                  {
                    id: 'ponder',
                    icon: Telescope,
                    label: 'Open in Ponder',
                    onClick: () => setPonderOpen(true),
                  },
                  {
                    id: 'fix',
                    icon: Zap,
                    label: 'Fix Right Away',
                    onClick: () => setFixOpen(true),
                  },
                ]}
              />
            )}

            {/* Modals */}
            <ThreadToPonderModal
              open={ponderOpen}
              onClose={() => setPonderOpen(false)}
              defaultTitle={turns[0]?.question.slice(0, 40) ?? 'AMA Thread'}
              artifactFilename="ama-thread.md"
              artifactContent={buildThreadMarkdown(turns)}
              turnCount={turns.length}
              sourceCount={totalSources}
            />

            <FixRightAwayModal
              open={fixOpen}
              onClose={() => setFixOpen(false)}
              initialDescription={buildFixDescription(turns)}
            />
          </>
        )}
      </div>
    </div>
  )
}
