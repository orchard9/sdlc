import { useEffect, useRef, useState } from 'react'
import { X, Plus, Link, UploadCloud, FileText } from 'lucide-react'
import { api } from '@/api/client'
import { titleToSlug } from '../../lib/slug'
import { cn, formatBytes } from '@/lib/utils'

// Accepted file extensions for text preload
const ACCEPTED_EXTS = new Set([
  '.md', '.txt', '.html', '.svg', '.js', '.ts', '.tsx', '.jsx',
  '.rs', '.py', '.go', '.json', '.yaml', '.yml', '.toml', '.css', '.sh',
])

function isAccepted(file: File): boolean {
  const parts = file.name.split('.')
  if (parts.length < 2) return false
  const ext = '.' + parts[parts.length - 1].toLowerCase()
  return ACCEPTED_EXTS.has(ext)
}

interface NewIdeaModalProps {
  open: boolean
  onClose: () => void
  onCreated: (slug: string) => void
  initialTitle?: string
  initialSlug?: string
  initialBrief?: string
}

export function NewIdeaModal({
  open,
  onClose,
  onCreated,
  initialTitle,
  initialSlug,
  initialBrief,
}: NewIdeaModalProps) {
  const [title, setTitle] = useState(initialTitle ?? '')
  const [slug, setSlug] = useState(initialSlug ?? '')
  const [brief, setBrief] = useState(initialBrief ?? '')
  const [refs, setRefs] = useState<string[]>([''])
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  // Track whether the user has manually edited the slug (breaks auto-derive)
  const slugManuallyEdited = useRef(false)

  // File attachment state (T1)
  const [attachedFiles, setAttachedFiles] = useState<File[]>([])
  const [isDragOver, setIsDragOver] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const titleRef = useRef<HTMLInputElement>(null)

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      setTitle(initialTitle ?? '')
      setSlug(initialSlug ?? titleToSlug(initialTitle ?? '').slice(0, 40))
      setBrief(initialBrief ?? '')
      setRefs([''])
      setError(null)
      setSubmitting(false)
      slugManuallyEdited.current = !!initialSlug
      // Reset file state on re-open (T1)
      setAttachedFiles([])
      setIsDragOver(false)
      // Auto-focus the title input
      setTimeout(() => titleRef.current?.focus(), 0)
    }
  }, [open, initialTitle, initialSlug, initialBrief])

  // Escape to close
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  const handleTitleChange = (value: string) => {
    setTitle(value)
    if (!slugManuallyEdited.current) {
      setSlug(titleToSlug(value).slice(0, 40))
    }
  }

  const handleSlugChange = (value: string) => {
    slugManuallyEdited.current = true
    setSlug(value.toLowerCase().replace(/[^a-z0-9-]/g, '-').slice(0, 40))
  }

  const handleRefChange = (index: number, value: string) => {
    setRefs(prev => prev.map((r, i) => (i === index ? value : r)))
  }

  const handleAddRef = () => {
    setRefs(prev => [...prev, ''])
  }

  const handleRemoveRef = (index: number) => {
    setRefs(prev => {
      const next = prev.filter((_, i) => i !== index)
      return next.length === 0 ? [''] : next
    })
  }

  // T1: file attachment helpers
  const handleFilesAdded = (files: FileList | null) => {
    if (!files) return
    const accepted = Array.from(files).filter(isAccepted)
    setAttachedFiles(prev => {
      const existing = new Set(prev.map(f => f.name))
      return [...prev, ...accepted.filter(f => !existing.has(f.name))]
    })
  }

  const handleRemoveFile = (index: number) => {
    setAttachedFiles(prev => prev.filter((_, i) => i !== index))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!slug.trim() || !title.trim() || submitting) return

    setSubmitting(true)
    setError(null)

    try {
      await api.createPonderEntry({
        slug: slug.trim(),
        title: title.trim(),
        brief: brief.trim() || undefined,
      })

      const validRefs = refs.map(r => r.trim()).filter(Boolean)
      if (validRefs.length > 0) {
        const refMd = `# References\n\n${validRefs.map(r => `- ${r}`).join('\n')}\n`
        await api.capturePonderArtifact(slug.trim(), {
          filename: 'references.md',
          content: refMd,
        })
      }

      // T4: capture attached text files as scrapbook artifacts
      for (const file of attachedFiles) {
        const content = await file.text()
        await api.capturePonderArtifact(slug.trim(), { filename: file.name, content })
      }

      // T4: include preloaded file names in the chat seed message
      const fileNames = attachedFiles.map(f => f.name).join(', ')
      const seed = [
        title.trim(),
        brief.trim(),
        fileNames ? `Preloaded files: ${fileNames}` : '',
      ].filter(Boolean).join('\n\n')
      api.startPonderChat(slug.trim(), seed).catch(() => {})

      onCreated(slug.trim())
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create')
      setSubmitting(false)
    }
  }

  if (!open) return null

  const canSubmit = slug.trim().length > 0 && title.trim().length > 0 && !submitting

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      aria-modal="true"
      role="dialog"
      aria-label="New Idea"
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />

      {/* Card */}
      <div
        className="relative bg-card border border-border rounded-xl shadow-xl w-full max-w-xl mx-4 max-h-[85vh] flex flex-col"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="shrink-0 flex items-center justify-between px-5 py-4 border-b border-border">
          <span className="text-sm font-semibold">New Idea</span>
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
        <form
          onSubmit={handleSubmit}
          className="flex-1 overflow-y-auto px-5 py-5 space-y-4"
        >
          {/* Title */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">Title</label>
            <input
              ref={titleRef}
              type="text"
              value={title}
              onChange={e => handleTitleChange(e.target.value)}
              placeholder="What are you thinking about?"
              className="w-full px-3 py-2 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground"
            />
          </div>

          {/* Slug */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">Slug</label>
            <input
              type="text"
              value={slug}
              onChange={e => handleSlugChange(e.target.value)}
              placeholder="slug"
              className="w-full px-3 py-1.5 text-xs font-mono bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground text-muted-foreground"
            />
          </div>

          {/* Description */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">
              Description <span className="text-muted-foreground/50">(optional)</span>
            </label>
            <textarea
              value={brief}
              onChange={e => setBrief(e.target.value)}
              placeholder="Expand on the idea — what's the context, what's the problem, what are you hoping to explore?"
              rows={6}
              className="w-full px-3 py-2.5 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground resize-none"
            />
          </div>

          {/* References */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">
              References <span className="text-muted-foreground/50">(optional)</span>
            </label>
            <div className="space-y-2">
              {refs.map((ref, i) => (
                <div key={i} className="flex items-center gap-2">
                  <div className="flex-1 flex items-center gap-2 px-3 py-2 bg-muted/60 border border-border rounded-lg focus-within:border-primary/50 focus-within:ring-1 focus-within:ring-primary/20 transition-colors">
                    <Link className="w-3.5 h-3.5 text-muted-foreground/50 shrink-0" />
                    <input
                      type="url"
                      value={ref}
                      onChange={e => handleRefChange(i, e.target.value)}
                      placeholder="https://..."
                      className="flex-1 text-sm bg-transparent outline-none placeholder:text-muted-foreground"
                    />
                  </div>
                  {refs.length > 1 || ref.trim() ? (
                    <button
                      type="button"
                      onClick={() => handleRemoveRef(i)}
                      className="shrink-0 p-1.5 text-muted-foreground/50 hover:text-muted-foreground transition-colors"
                      aria-label="Remove reference"
                    >
                      <X className="w-3.5 h-3.5" />
                    </button>
                  ) : (
                    // Placeholder to keep alignment consistent
                    <div className="w-8 shrink-0" />
                  )}
                </div>
              ))}
              <button
                type="button"
                onClick={handleAddRef}
                className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors py-0.5"
              >
                <Plus className="w-3.5 h-3.5" />
                Add reference
              </button>
            </div>
          </div>

          {/* T2: Files section */}
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">
              Files <span className="text-muted-foreground/50">(optional)</span>
            </label>
            {/* Hidden file input */}
            <input
              ref={fileInputRef}
              type="file"
              multiple
              accept=".md,.txt,.html,.svg,.js,.ts,.tsx,.jsx,.rs,.py,.go,.json,.yaml,.yml,.toml,.css,.sh"
              className="hidden"
              onChange={e => {
                handleFilesAdded(e.target.files)
                // Reset so the same file can be re-selected after removal
                e.target.value = ''
              }}
            />
            {/* Drop zone */}
            <div
              role="button"
              tabIndex={0}
              aria-label="Attach files"
              onClick={() => fileInputRef.current?.click()}
              onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') fileInputRef.current?.click() }}
              onDragOver={e => { e.preventDefault(); setIsDragOver(true) }}
              onDragLeave={e => { if (!e.currentTarget.contains(e.relatedTarget as Node)) setIsDragOver(false) }}
              onDrop={e => {
                e.preventDefault()
                setIsDragOver(false)
                handleFilesAdded(e.dataTransfer.files)
              }}
              className={cn(
                'cursor-pointer flex flex-col items-center gap-1 py-4 px-3 border border-dashed rounded-lg transition-colors select-none',
                isDragOver
                  ? 'border-primary/60 bg-primary/5'
                  : 'border-border hover:border-primary/40 hover:bg-muted/30',
              )}
            >
              <UploadCloud className="w-4 h-4 text-muted-foreground/50" />
              <p className="text-xs text-muted-foreground">
                {isDragOver ? 'Release to attach' : 'Drop files here or click to browse'}
              </p>
              <p className="text-[10px] text-muted-foreground/50">
                .md .txt .html .svg .js .ts .tsx .jsx .rs .py .go .json .yaml .toml .css .sh
              </p>
            </div>
            {/* T3: File chips */}
            {attachedFiles.length > 0 && (
              <div className="space-y-1.5 pt-0.5">
                {attachedFiles.map((file, i) => {
                  const isLarge = file.size > 500 * 1024
                  return (
                    <div
                      key={i}
                      className="flex items-center gap-2 px-2.5 py-1.5 bg-muted/40 border border-border rounded-lg text-xs"
                    >
                      <FileText className="w-3.5 h-3.5 text-primary/70 shrink-0" />
                      <span className="flex-1 text-foreground font-medium truncate">{file.name}</span>
                      <span className="text-muted-foreground shrink-0">{formatBytes(file.size)}</span>
                      {isLarge && (
                        <span
                          className="text-amber-500 shrink-0 text-[10px]"
                          title="Large file — will be included but may use significant agent context"
                        >
                          ⚠
                        </span>
                      )}
                      <button
                        type="button"
                        onClick={() => handleRemoveFile(i)}
                        className="shrink-0 p-0.5 text-muted-foreground/50 hover:text-muted-foreground transition-colors"
                        aria-label={`Remove ${file.name}`}
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )
                })}
              </div>
            )}
          </div>

          {/* Error */}
          {error && (
            <p className="text-xs text-destructive">{error}</p>
          )}
        </form>

        {/* Footer */}
        <div className="shrink-0 flex items-center justify-end gap-3 px-5 py-4 border-t border-border bg-card">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={!canSubmit}
            onClick={handleSubmit}
            className="px-4 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {submitting ? 'Creating…' : 'Create Idea'}
          </button>
        </div>
      </div>
    </div>
  )
}
