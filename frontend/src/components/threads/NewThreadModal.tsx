import { useEffect, useRef, useState } from 'react'
import { MessageSquare, X } from 'lucide-react'

interface NewThreadModalProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: { title: string; body?: string }) => Promise<void>
}

export function NewThreadModal({ open, onClose, onSubmit }: NewThreadModalProps) {
  const [title, setTitle] = useState('')
  const [body, setBody] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const titleRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) {
      setTitle('')
      setBody('')
      setError(null)
      setSubmitting(false)
      setTimeout(() => titleRef.current?.focus(), 50)
    }
  }, [open])

  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const t = title.trim()
    if (!t) return
    setSubmitting(true)
    setError(null)
    try {
      await onSubmit({ title: t, body: body.trim() || undefined })
      setSubmitting(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create thread')
      setSubmitting(false)
    }
  }

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      onClick={(e) => { if (e.target === e.currentTarget) onClose() }}
    >
      <div className="w-[520px] max-w-[calc(100vw-32px)] bg-card border border-border rounded-xl shadow-2xl p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-5">
          <div className="flex items-center gap-2 text-base font-semibold">
            <MessageSquare className="w-4 h-4 text-muted-foreground" />
            New thread
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          {/* Title */}
          <div className="mb-4">
            <label className="block text-[11px] font-semibold uppercase tracking-widest text-muted-foreground/70 mb-1.5">
              Title
            </label>
            <input
              ref={titleRef}
              type="text"
              value={title}
              onChange={e => setTitle(e.target.value)}
              placeholder="What's on your mind?"
              className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <p className="mt-1 text-[11px] text-muted-foreground/45">
              Short, scannable. You can always edit later.
            </p>
          </div>

          {/* Core element */}
          <div className="mb-5">
            <label className="block text-[11px] font-semibold uppercase tracking-widest text-muted-foreground/70 mb-1.5">
              Core element{' '}
              <span className="normal-case font-normal opacity-50">(optional)</span>
            </label>
            <textarea
              value={body}
              onChange={e => setBody(e.target.value)}
              rows={4}
              placeholder="Describe the idea, issue, or question in detail. You can leave this blank and fill it in later."
              className="w-full resize-none rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:ring-1 focus:ring-ring leading-relaxed"
            />
            <p className="mt-1 text-[11px] text-muted-foreground/45">
              This becomes the living summary — the thing synthesis updates over time.
            </p>
          </div>

          {error && (
            <p className="mb-3 text-xs text-destructive">{error}</p>
          )}

          {/* Footer */}
          <div className="flex items-center justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-3.5 py-1.5 rounded-lg border border-border text-sm text-muted-foreground hover:bg-accent transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting || !title.trim()}
              className="px-4 py-1.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? 'Creating…' : 'Create thread'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
