import { useEffect, type ReactNode } from 'react'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils'

interface FullscreenModalProps {
  open: boolean
  onClose: () => void
  title: string
  children: ReactNode
  className?: string
}

export function FullscreenModal({ open, onClose, title, children, className }: FullscreenModalProps) {
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-background">
      <div className="flex items-center justify-between px-6 py-3 border-b border-border shrink-0">
        <span className="text-sm font-semibold capitalize">{title.replace(/_/g, ' ')}</span>
        <button
          onClick={onClose}
          className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          aria-label="Close fullscreen"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
      <div className={cn('flex-1 overflow-y-auto px-8 py-6 max-w-4xl w-full mx-auto', className)}>
        {children}
      </div>
    </div>
  )
}
