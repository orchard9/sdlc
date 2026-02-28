import { useRef, useState } from 'react'
import { Telescope, Zap, RotateCcw, Send } from 'lucide-react'
import { api } from '@/api/client'
import { AmaResultPanel } from '@/components/tools/AmaResultPanel'
import { AmaAnswerPanel } from '@/components/tools/AmaAnswerPanel'
import { ToolResultActions } from '@/components/tools/ToolResultActions'
import { ThreadToPonderModal } from '@/components/shared/ThreadToPonderModal'
import { FixRightAwayModal } from '@/components/shared/FixRightAwayModal'
import type { AmaThreadTurn, ToolMeta, AmaData } from '@/lib/types'

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
      <AmaAnswerPanel runKey={turn.synthesisRunKey} onDone={onSynthesisDone} />
    </div>
  )
}

// ---------------------------------------------------------------------------
// AmaThreadPanel
// ---------------------------------------------------------------------------

export function AmaThreadPanel({ tool }: AmaThreadPanelProps) {
  const [turns, setTurns] = useState<AmaThreadTurn[]>([])
  const [question, setQuestion] = useState('')
  const [followUp, setFollowUp] = useState('')
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const [ponderOpen, setPonderOpen] = useState(false)
  const [fixOpen, setFixOpen] = useState(false)

  const followUpRef = useRef<HTMLInputElement>(null)

  const lastTurn = turns[turns.length - 1] ?? null
  const lastDone = lastTurn?.synthesisText !== null && lastTurn?.synthesisText !== undefined
    ? lastTurn.synthesisText !== null
    : false
  const threadReady = turns.length > 0 && lastTurn !== null && lastTurn.synthesisText !== null

  // Total unique source count across all turns
  const totalSources = new Set(turns.flatMap(t => t.sources.map(s => s.path))).size

  const updateTurnSynthesis = (index: number, text: string) => {
    setTurns(prev => prev.map((t, i) => i === index ? { ...t, synthesisText: text } : t))
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

      // Start synthesis immediately (fire-and-forget for the key)
      let synthesisRunKey = ''
      try {
        const answerResp = await api.answerAma(
          q.trim(),
          amaData.sources,
          {
            turnIndex,
            threadContext: priorContext || undefined,
          }
        )
        synthesisRunKey = answerResp.run_key
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

  const handleReset = () => {
    setTurns([])
    setQuestion('')
    setFollowUp('')
    setSearchError(null)
  }

  // Initial input: shown when no turns yet
  if (turns.length === 0) {
    return (
      <div className="space-y-3">
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
      </div>
    )
  }

  // Thread view: turns + follow-up + action bar
  return (
    <div className="space-y-4">
      {/* Thread header + reset */}
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground font-medium">
          {turns.length} question{turns.length !== 1 ? 's' : ''} · {totalSources} source{totalSources !== 1 ? 's' : ''}
        </span>
        <button
          onClick={handleReset}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
        >
          <RotateCcw className="w-3 h-3" />
          New thread
        </button>
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
    </div>
  )
}
