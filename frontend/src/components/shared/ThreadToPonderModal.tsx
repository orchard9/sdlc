import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Telescope, ArrowLeft } from 'lucide-react'
import { api } from '@/api/client'

interface ThreadToPonderModalProps {
  open: boolean
  onClose: () => void
  defaultTitle: string
  artifactFilename: string
  artifactContent: string
  turnCount?: number
  sourceCount?: number
}

function toSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40)
}

type Step = 'form' | 'creating'

export function ThreadToPonderModal({
  open,
  onClose,
  defaultTitle,
  artifactFilename,
  artifactContent,
  turnCount,
  sourceCount,
}: ThreadToPonderModalProps) {
  const navigate = useNavigate()
  const [step, setStep] = useState<Step>('form')
  const [title, setTitle] = useState(defaultTitle)
  const [error, setError] = useState<string | null>(null)
  const titleRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) {
      setStep('form')
      setTitle(defaultTitle)
      setError(null)
      setTimeout(() => titleRef.current?.focus(), 0)
    }
  }, [open, defaultTitle])

  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  const slug = toSlug(title)

  const handleCreate = async () => {
    if (!title.trim() || !slug) return
    setError(null)
    setStep('creating')

    const tryCreate = async (s: string, t: string): Promise<void> => {
      try {
        await api.createPonderEntry({ slug: s, title: t })
      } catch (e) {
        // 409 conflict — try with -2 suffix once
        if (e instanceof Error && e.message.includes('409')) {
          await api.createPonderEntry({ slug: s + '-2', title: t })
          await api.capturePonderArtifact(s + '-2', { filename: artifactFilename, content: artifactContent })
          navigate('/ponder/' + s + '-2')
          onClose()
          return
        }
        throw e
      }
      await api.capturePonderArtifact(s, { filename: artifactFilename, content: artifactContent })
      navigate('/ponder/' + s)
      onClose()
    }

    try {
      await tryCreate(slug, title.trim())
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create ponder entry')
      setStep('form')
    }
  }

  if (!open) return null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Open in Ponder"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/60"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md mx-4 bg-card border border-border rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-4 pt-4 pb-3 border-b border-border">
          <div className="flex items-center gap-2 mb-0.5">
            <Telescope className="w-4 h-4 text-primary" />
            <span className="text-sm font-semibold">Open in Ponder</span>
          </div>
          <p className="text-xs text-muted-foreground">
            {step === 'form' && 'Save this thread as a ponder exploration.'}
            {step === 'creating' && 'Creating ponder entry…'}
          </p>
        </div>

        {step === 'form' && (
          <div className="p-4 space-y-4">
            {/* Summary */}
            {(turnCount !== undefined || sourceCount !== undefined) && (
              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                {turnCount !== undefined && (
                  <span>{turnCount} question{turnCount !== 1 ? 's' : ''}</span>
                )}
                {sourceCount !== undefined && (
                  <span>{sourceCount} source{sourceCount !== 1 ? 's' : ''}</span>
                )}
              </div>
            )}

            {/* Title input */}
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">Title</label>
              <input
                ref={titleRef}
                type="text"
                value={title}
                onChange={e => setTitle(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter') handleCreate() }}
                className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring"
              />
            </div>

            {/* Slug preview */}
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">Slug</label>
              <p className="text-xs font-mono text-muted-foreground/70 bg-muted/40 px-2 py-1.5 rounded border border-border/50">
                {slug || <span className="italic opacity-50">— enter a title —</span>}
              </p>
            </div>

            {error && <p className="text-xs text-destructive">{error}</p>}

            <div className="flex items-center justify-between">
              <button
                onClick={onClose}
                className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                <ArrowLeft className="w-3.5 h-3.5" />
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={!slug}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                <Telescope className="w-3.5 h-3.5" />
                Open in Ponder
              </button>
            </div>
          </div>
        )}

        {step === 'creating' && (
          <div className="p-8 flex flex-col items-center gap-3 text-muted-foreground">
            <span className="w-5 h-5 border-2 border-muted-foreground/30 border-t-primary rounded-full animate-spin" />
            <p className="text-sm">Creating ponder entry…</p>
          </div>
        )}
      </div>
    </div>
  )
}
