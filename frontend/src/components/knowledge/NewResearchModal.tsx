import { useEffect, useRef, useState } from 'react'
import { X, FlaskConical } from 'lucide-react'
import { api } from '@/api/client'

interface NewResearchModalProps {
  open: boolean
  entrySlug: string
  entryTitle: string
  onClose: () => void
  onStarted: () => void
}

export function NewResearchModal({
  open,
  entrySlug,
  entryTitle,
  onClose,
  onStarted,
}: NewResearchModalProps) {
  const [topic, setTopic] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  // Reset state and focus when modal opens
  useEffect(() => {
    if (open) {
      setTopic('')
      setError(null)
      setSubmitting(false)
      setTimeout(() => inputRef.current?.focus(), 0)
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (submitting) return

    setSubmitting(true)
    setError(null)

    try {
      const trimmed = topic.trim()
      await api.researchKnowledge(entrySlug, trimmed || undefined)
      onStarted()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start research')
      setSubmitting(false)
    }
  }

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      aria-modal="true"
      role="dialog"
      aria-label={`Research: ${entryTitle}`}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />

      {/* Card */}
      <div
        className="relative bg-card border border-border rounded-xl shadow-xl w-full max-w-sm mx-4 flex flex-col"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="shrink-0 flex items-center justify-between px-5 py-4 border-b border-border">
          <div className="flex items-center gap-2">
            <FlaskConical className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-semibold truncate max-w-[200px]" title={entryTitle}>
              Research: {entryTitle}
            </span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <form onSubmit={handleSubmit} className="px-5 py-5 space-y-4">
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">
              Topic hint <span className="text-muted-foreground/50">(optional)</span>
            </label>
            <input
              ref={inputRef}
              type="text"
              value={topic}
              onChange={e => setTopic(e.target.value)}
              placeholder="Leave blank to research the full entry topic"
              disabled={submitting}
              className="w-full px-3 py-2 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground disabled:opacity-50"
            />
          </div>

          {error && (
            <p className="text-xs text-destructive">{error}</p>
          )}
        </form>

        {/* Footer */}
        <div className="shrink-0 flex items-center justify-end gap-3 px-5 py-4 border-t border-border bg-card rounded-b-xl">
          <button
            type="button"
            onClick={onClose}
            disabled={submitting}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={submitting}
            onClick={handleSubmit}
            className="px-4 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {submitting ? 'Starting…' : 'Start Research'}
          </button>
        </div>
      </div>
    </div>
  )
}
