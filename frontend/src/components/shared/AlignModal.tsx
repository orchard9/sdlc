import { useEffect, useRef, useState } from 'react'
import { Sparkles } from 'lucide-react'

interface AlignModalProps {
  open: boolean
  onClose: () => void
  onConfirm: (direction: string) => void
  title: string
}

export function AlignModal({ open, onClose, onConfirm, title }: AlignModalProps) {
  const [direction, setDirection] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (open) {
      setDirection('')
      setTimeout(() => textareaRef.current?.focus(), 0)
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

  const handleConfirm = () => {
    onConfirm(direction.trim())
  }

  if (!open) return null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/60"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md mx-4 bg-card border border-border rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        <div className="px-4 pt-4 pb-3 border-b border-border">
          <div className="flex items-center gap-2 mb-0.5">
            <Sparkles className="w-4 h-4 text-primary" />
            <span className="text-sm font-semibold">{title}</span>
          </div>
          <p className="text-xs text-muted-foreground">
            Describe what to focus on or emphasise. Leave blank for a general alignment.
          </p>
        </div>

        <div className="p-4 space-y-3">
          <textarea
            ref={textareaRef}
            value={direction}
            onChange={e => setDirection(e.target.value)}
            onKeyDown={e => {
              if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') handleConfirm()
            }}
            placeholder="e.g. Emphasise the agent runtime model and remove stale milestones"
            rows={4}
            className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring placeholder:text-muted-foreground resize-none"
          />
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">⌘↵ to confirm</span>
            <div className="flex items-center gap-2">
              <button
                onClick={onClose}
                className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                className="flex items-center gap-1.5 px-4 py-1.5 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
              >
                <Sparkles className="w-3.5 h-3.5" />
                Align
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
