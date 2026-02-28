import { useEffect, useState } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Loader2, Sparkles } from 'lucide-react'

interface AmaAnswerPanelProps {
  runKey: string
  onDone?: (finalText: string) => void
}

export function AmaAnswerPanel({ runKey, onDone }: AmaAnswerPanelProps) {
  const [text, setText] = useState('')
  const [done, setDone] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setText('')
    setDone(false)
    setError(null)

    let accumulated = ''
    const es = new EventSource(`/api/run/${encodeURIComponent(runKey)}/events`)

    es.addEventListener('agent', (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data)
        if (data.type === 'assistant' && data.text) {
          accumulated += data.text
          setText(prev => prev + data.text)
        } else if (data.type === 'result') {
          setDone(true)
          if (data.is_error) setError(data.text || 'Agent error')
          onDone?.(accumulated)
          es.close()
        } else if (data.type === 'error') {
          setError(data.message || 'Synthesis error')
          setDone(true)
          onDone?.(accumulated)
          es.close()
        }
      } catch {
        // malformed event — ignore
      }
    })

    es.onerror = () => {
      // 404 = run ended or not found — just mark done
      setDone(true)
      onDone?.(accumulated)
      es.close()
    }

    return () => es.close()
  }, [runKey])

  return (
    <div className="mt-4 space-y-2">
      <div className="flex items-center gap-2 border-t border-border/50 pt-4">
        <Sparkles className="w-3.5 h-3.5 text-primary shrink-0" />
        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
          Synthesis
        </span>
        {!done && (
          <Loader2 className="w-3 h-3 animate-spin text-muted-foreground ml-auto" />
        )}
      </div>

      {error ? (
        <p className="text-sm text-red-400">{error}</p>
      ) : text ? (
        <div className="text-sm text-foreground leading-relaxed [&_p]:mb-2 [&_p:last-child]:mb-0 [&_code]:text-xs [&_code]:font-mono [&_code]:bg-muted/60 [&_code]:border [&_code]:border-border/50 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_ul]:list-disc [&_ul]:pl-5 [&_ul]:space-y-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_ol]:space-y-1 [&_strong]:font-semibold">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {text}
          </ReactMarkdown>
        </div>
      ) : (
        !done && (
          <p className="text-sm text-muted-foreground italic">Synthesizing answer…</p>
        )
      )}
    </div>
  )
}
