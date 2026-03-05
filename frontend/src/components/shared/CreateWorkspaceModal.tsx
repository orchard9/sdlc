import { useEffect, useRef, useState } from 'react'
import { X, Plus, Link } from 'lucide-react'
import { titleToSlug } from '@/lib/slug'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface WorkspaceFieldConfig {
  /** Show a brief/description textarea (Ponder style) */
  showBrief?: boolean
  briefPlaceholder?: string
  /** Show reference URL inputs (Ponder only) */
  showReferences?: boolean
  /** Show a scope input (Evolve, Guideline) */
  showScope?: boolean
  scopePlaceholder?: string
  /** Show a context textarea (RootCause, Evolve, Guideline) */
  showContext?: boolean
  contextPlaceholder?: string
  /** Whether context is required for submit */
  requireContext?: boolean
}

export interface CreateWorkspaceValues {
  slug: string
  title: string
  brief?: string
  scope?: string
  context?: string
  references?: string[]
}

export interface CreateWorkspaceModalProps {
  open: boolean
  onClose: () => void
  onCreated: (slug: string) => void
  /** Modal header title, e.g. "New Idea" */
  title: string
  /** Submit button label. Defaults to "Create" */
  submitLabel?: string
  initialTitle?: string
  initialSlug?: string
  fields?: WorkspaceFieldConfig
  /** Called with form values on submit. Should throw on error. */
  onSubmit: (values: CreateWorkspaceValues) => Promise<void>
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function CreateWorkspaceModal({
  open,
  onClose,
  onCreated,
  title: modalTitle,
  submitLabel = 'Create',
  initialTitle,
  initialSlug,
  fields = {},
  onSubmit,
}: CreateWorkspaceModalProps) {
  const [title, setTitle] = useState(initialTitle ?? '')
  const [slug, setSlug] = useState(initialSlug ?? '')
  const [brief, setBrief] = useState('')
  const [scope, setScope] = useState('')
  const [context, setContext] = useState('')
  const [refs, setRefs] = useState<string[]>([''])
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const slugManuallyEdited = useRef(false)
  const titleRef = useRef<HTMLInputElement>(null)

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      setTitle(initialTitle ?? '')
      setSlug(initialSlug ?? titleToSlug(initialTitle ?? '').slice(0, 40))
      setBrief('')
      setScope('')
      setContext('')
      setRefs([''])
      setError(null)
      setSubmitting(false)
      slugManuallyEdited.current = !!initialSlug
      setTimeout(() => titleRef.current?.focus(), 0)
    }
  }, [open, initialTitle, initialSlug])

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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return

    setSubmitting(true)
    setError(null)

    const values: CreateWorkspaceValues = {
      slug: slug.trim(),
      title: title.trim(),
    }
    if (fields.showBrief) values.brief = brief.trim() || undefined
    if (fields.showScope) values.scope = scope.trim() || undefined
    if (fields.showContext) values.context = context.trim() || undefined
    if (fields.showReferences) {
      const validRefs = refs.map(r => r.trim()).filter(Boolean)
      if (validRefs.length > 0) values.references = validRefs
    }

    try {
      await onSubmit(values)
      onCreated(slug.trim())
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create')
      setSubmitting(false)
    }
  }

  if (!open) return null

  const requireContext = fields.requireContext ?? false
  const canSubmit =
    slug.trim().length > 0 &&
    title.trim().length > 0 &&
    (!requireContext || context.trim().length > 0) &&
    !submitting

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      aria-modal="true"
      role="dialog"
      aria-label={modalTitle}
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
          <span className="text-sm font-semibold">{modalTitle}</span>
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
              placeholder="Name this workspace"
              disabled={submitting}
              className="w-full px-3 py-2 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground disabled:opacity-60"
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
              disabled={submitting}
              className="w-full px-3 py-1.5 text-xs font-mono bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground text-muted-foreground disabled:opacity-60"
            />
          </div>

          {/* Scope (optional) */}
          {fields.showScope && (
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">
                Scope{' '}
                <span className="text-muted-foreground/50">(optional)</span>
              </label>
              <input
                type="text"
                value={scope}
                onChange={e => setScope(e.target.value)}
                placeholder={fields.scopePlaceholder ?? 'scope — files or modules this applies to'}
                disabled={submitting}
                className="w-full px-3 py-1.5 text-xs font-mono bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground disabled:opacity-60"
              />
            </div>
          )}

          {/* Brief / Description (optional) */}
          {fields.showBrief && (
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">
                Description{' '}
                <span className="text-muted-foreground/50">(optional)</span>
              </label>
              <textarea
                value={brief}
                onChange={e => setBrief(e.target.value)}
                placeholder={fields.briefPlaceholder ?? 'Expand on the idea…'}
                rows={6}
                disabled={submitting}
                className="w-full px-3 py-2.5 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground resize-none disabled:opacity-60"
              />
            </div>
          )}

          {/* Context */}
          {fields.showContext && (
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">
                Context
                {!requireContext && (
                  <span className="text-muted-foreground/50"> (optional)</span>
                )}
              </label>
              <textarea
                value={context}
                onChange={e => setContext(e.target.value)}
                placeholder={fields.contextPlaceholder ?? 'Provide context…'}
                rows={5}
                disabled={submitting}
                className="w-full px-3 py-2.5 text-sm bg-muted/60 border border-border rounded-lg outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-colors placeholder:text-muted-foreground resize-none disabled:opacity-60"
              />
            </div>
          )}

          {/* References */}
          {fields.showReferences && (
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">
                References{' '}
                <span className="text-muted-foreground/50">(optional)</span>
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
                        disabled={submitting}
                        className="flex-1 text-sm bg-transparent outline-none placeholder:text-muted-foreground disabled:opacity-60"
                      />
                    </div>
                    {refs.length > 1 || ref.trim() ? (
                      <button
                        type="button"
                        onClick={() => handleRemoveRef(i)}
                        disabled={submitting}
                        className="shrink-0 p-1.5 text-muted-foreground/50 hover:text-muted-foreground transition-colors disabled:opacity-40"
                        aria-label="Remove reference"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    ) : (
                      <div className="w-8 shrink-0" />
                    )}
                  </div>
                ))}
                <button
                  type="button"
                  onClick={handleAddRef}
                  disabled={submitting}
                  className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors py-0.5 disabled:opacity-40"
                >
                  <Plus className="w-3.5 h-3.5" />
                  Add reference
                </button>
              </div>
            </div>
          )}

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
            disabled={submitting}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors disabled:opacity-40"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={!canSubmit}
            onClick={handleSubmit}
            className="px-4 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {submitting ? 'Creating…' : submitLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
