import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Code2 } from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { api } from '@/api/client'
import type { AmaSource } from '@/lib/types'

interface AskPonderModalProps {
  open: boolean
  onClose: () => void
}

type Step = 'input' | 'indexing' | 'answering' | 'answered'

function toThreadId(question: string): string {
  return (
    'ask-' +
    question
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, '')
      .trim()
      .replace(/\s+/g, '-')
      .slice(0, 44)
  )
}

export function AskPonderModal({ open, onClose }: AskPonderModalProps) {
  const navigate = useNavigate()

  const [step, setStep] = useState<Step>('input')
  const [question, setQuestion] = useState('')
  const [sources, setSources] = useState<AmaSource[]>([])
  const [runKey, setRunKey] = useState<string | null>(null)
  const [runId, setRunId] = useState<string | null>(null)
  const [answerText, setAnswerText] = useState('')
  const [, setStreamDone] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [openingThread, setOpeningThread] = useState(false)

  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Reset on open
  useEffect(() => {
    if (open) {
      setStep('input')
      setQuestion('')
      setSources([])
      setRunKey(null)
      setRunId(null)
      setAnswerText('')
      setStreamDone(false)
      setError(null)
      setOpeningThread(false)
      setTimeout(() => textareaRef.current?.focus(), 0)
    }
  }, [open])

  // Escape to close
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  // Stream answer when runKey is set
  useEffect(() => {
    if (!runKey || step !== 'answering') return

    let accumulated = ''
    const es = new EventSource(`/api/run/${encodeURIComponent(runKey)}/events`)

    es.addEventListener('agent', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data)
        if (data.type === 'assistant' && data.text) {
          accumulated += data.text
          setAnswerText(prev => prev + data.text)
        } else if (data.type === 'result' || data.type === 'error') {
          setStreamDone(true)
          setStep('answered')
          es.close()
        }
      } catch {
        // malformed — ignore
      }
    })

    es.onerror = () => {
      setStreamDone(true)
      setStep('answered')
      es.close()
    }

    return () => es.close()
  }, [runKey, step])

  const runSearch = async (q: string): Promise<AmaSource[]> => {
    const result = await api.runTool('ama', { question: q }) as { ok?: boolean; data?: { sources?: AmaSource[] }; error?: string }
    if (!result.ok) throw new Error(result.error ?? 'Search failed')
    return result.data?.sources ?? []
  }

  const handleAsk = async () => {
    const q = question.trim()
    if (!q) return
    setError(null)
    setAnswerText('')
    setStreamDone(false)
    setSources([])

    try {
      // Step 1: search the index
      let foundSources: AmaSource[]
      try {
        foundSources = await runSearch(q)
      } catch (searchErr) {
        const msg = searchErr instanceof Error ? searchErr.message : ''
        // Index not built yet — run setup then retry
        if (msg.toLowerCase().includes('index') || msg.toLowerCase().includes('not built') || msg.toLowerCase().includes('no such file')) {
          setStep('indexing')
          await api.setupTool('ama')
          foundSources = await runSearch(q)
        } else {
          throw searchErr
        }
      }

      setSources(foundSources)
      setStep('answering')

      // Step 2: synthesize answer
      const res = await api.answerAma(q, foundSources, { turnIndex: 0 })
      setRunKey(res.run_key)
      setRunId(res.run_id)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start answer')
      setStep('input')
    }
  }

  const handleAskAnother = () => {
    setStep('input')
    setAnswerText('')
    setSources([])
    setRunKey(null)
    setRunId(null)
    setStreamDone(false)
    setError(null)
    setTimeout(() => textareaRef.current?.focus(), 0)
  }

  const handleOpenAsThread = async () => {
    if (openingThread) return
    setOpeningThread(true)
    try {
      const threadId = toThreadId(question)
      await api.createAmaThread(threadId, question.trim())
      await api.addAmaThreadTurn(threadId, {
        question: question.trim(),
        sources,
        run_id: runId ?? undefined,
      })
      navigate(`/threads/${threadId}`)
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to open thread')
      setOpeningThread(false)
    }
  }

  if (!open) return null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Ask Code"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/60"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl mx-4 bg-card border border-border rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-4 pt-4 pb-3 border-b border-border">
          <div className="flex items-center gap-2 mb-0.5">
            <Code2 className="w-4 h-4 text-primary" />
            <span className="text-sm font-semibold">Ask Code</span>
            {step === 'indexing' && (
              <span className="ml-auto flex items-center gap-1.5 text-xs text-muted-foreground">
                <span className="w-1.5 h-1.5 bg-amber-500 rounded-full animate-pulse" />
                Building index…
              </span>
            )}
            {step === 'answering' && (
              <span className="ml-auto flex items-center gap-1.5 text-xs text-muted-foreground">
                <span className="w-1.5 h-1.5 bg-primary rounded-full animate-pulse" />
                Reading codebase…
              </span>
            )}
          </div>
          {step === 'input' && (
            <p className="text-xs text-muted-foreground">
              Ask how a feature works, what a file does, or how things connect.
            </p>
          )}
          {(step === 'indexing' || step === 'answering' || step === 'answered') && (
            <p className="text-xs text-muted-foreground truncate">
              "{question}"
            </p>
          )}
        </div>

        {/* Step: input */}
        {step === 'input' && (
          <div className="p-4 space-y-3">
            <textarea
              ref={textareaRef}
              value={question}
              onChange={e => setQuestion(e.target.value)}
              onKeyDown={e => {
                if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') handleAsk()
              }}
              placeholder={`How does Fix Right Away diagnose issues?\nHow are agent runs streamed?\nWhat controls sidebar collapse state?`}
              rows={3}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring placeholder:text-muted-foreground resize-none"
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">⌘↵ to ask</span>
              <button
                onClick={handleAsk}
                disabled={!question.trim()}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                <Code2 className="w-3.5 h-3.5" />
                Ask
              </button>
            </div>
          </div>
        )}

        {/* Step: indexing */}
        {step === 'indexing' && (
          <div className="p-4">
            <div className="flex items-center gap-3 text-muted-foreground py-4">
              <span className="w-4 h-4 border-2 border-muted-foreground/30 border-t-amber-500 rounded-full animate-spin shrink-0" />
              <div>
                <p className="text-sm">Building code index…</p>
                <p className="text-xs text-muted-foreground/60 mt-0.5">First time only — this takes a few seconds</p>
              </div>
            </div>
          </div>
        )}

        {/* Step: answering */}
        {step === 'answering' && (
          <div className="p-4 max-h-72 overflow-y-auto">
            {answerText ? (
              <div className="text-sm leading-relaxed [&_p]:mb-2 [&_p:last-child]:mb-0 [&_code]:text-xs [&_code]:font-mono [&_code]:bg-muted/60 [&_code]:border [&_code]:border-border/50 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_ul]:list-disc [&_ul]:pl-5 [&_ul]:space-y-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_ol]:space-y-1 [&_strong]:font-semibold">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{answerText}</ReactMarkdown>
              </div>
            ) : (
              <div className="flex items-center gap-3 text-muted-foreground py-4">
                <span className="w-4 h-4 border-2 border-muted-foreground/30 border-t-primary rounded-full animate-spin shrink-0" />
                <p className="text-sm">Reading codebase and synthesizing…</p>
              </div>
            )}
          </div>
        )}

        {/* Step: answered */}
        {step === 'answered' && (
          <>
            <div className="p-4 max-h-80 overflow-y-auto">
              <div className="text-sm leading-relaxed [&_p]:mb-2 [&_p:last-child]:mb-0 [&_code]:text-xs [&_code]:font-mono [&_code]:bg-muted/60 [&_code]:border [&_code]:border-border/50 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_ul]:list-disc [&_ul]:pl-5 [&_ul]:space-y-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_ol]:space-y-1 [&_strong]:font-semibold">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{answerText}</ReactMarkdown>
              </div>
            </div>
            <div className="px-4 pb-4 pt-2 border-t border-border flex items-center justify-between">
              <button
                onClick={handleAskAnother}
                className="text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                Ask another
              </button>
              <button
                onClick={handleOpenAsThread}
                disabled={openingThread}
                className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-border hover:border-border/80 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-40"
              >
                {openingThread ? (
                  <span className="w-3 h-3 border border-muted-foreground/30 border-t-foreground rounded-full animate-spin" />
                ) : null}
                Open as Thread
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
