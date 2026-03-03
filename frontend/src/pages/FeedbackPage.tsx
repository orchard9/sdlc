import { useState, useEffect, useRef, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { Trash2, Send, Loader2, MessageSquare, Pencil, Plus } from 'lucide-react'
import { api } from '@/api/client'
import type { FeedbackNote } from '@/lib/types'

export function FeedbackPage() {
  const navigate = useNavigate()
  const [notes, setNotes] = useState<FeedbackNote[]>([])
  const [draft, setDraft] = useState('')
  const [saving, setSaving] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [editingId, setEditingId] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const load = useCallback(async () => {
    try {
      const data = await api.getFeedback()
      setNotes(data.slice().reverse()) // newest first in UI
    } catch {
      // ignore
    }
  }, [])

  useEffect(() => {
    load()
    textareaRef.current?.focus()
  }, [load])

  async function saveNote() {
    const content = draft.trim()
    if (!content) return
    setSaving(true)
    setError(null)
    try {
      const note = await api.addFeedbackNote(content)
      setNotes(prev => [note, ...prev])
      setDraft('')
      textareaRef.current?.focus()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save note')
    } finally {
      setSaving(false)
    }
  }

  async function deleteNote(id: string) {
    setNotes(prev => prev.filter(n => n.id !== id))
    try {
      await api.deleteFeedbackNote(id)
    } catch {
      // Restore on error
      load()
    }
  }

  function onEdit(id: string, newContent: string) {
    setNotes(prev => prev.map(n => n.id === id ? { ...n, content: newContent, updated_at: new Date().toISOString() } : n))
  }

  function onEditError(id: string, originalContent: string) {
    setNotes(prev => prev.map(n => n.id === id ? { ...n, content: originalContent } : n))
  }

  function onEnrich(id: string, updatedNote: FeedbackNote) {
    setNotes(prev => prev.map(n => n.id === id ? updatedNote : n))
  }

  async function submitToPonder() {
    setSubmitting(true)
    setError(null)
    try {
      const result = await api.submitFeedbackToPonder()
      setNotes([])
      navigate(`/ponder/${result.slug}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to submit')
      setSubmitting(false)
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      saveNote()
    }
  }

  const hasNotes = notes.length > 0
  const hasDraft = draft.trim().length > 0

  return (
    <div className="max-w-3xl mx-auto p-4 sm:p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <div className="flex items-center gap-2.5 mb-1">
            <MessageSquare className="w-5 h-5 text-muted-foreground" />
            <h2 className="text-xl font-semibold">Feedback</h2>
          </div>
          <p className="text-sm text-muted-foreground">
            Write anything — ideas, issues, observations. Submit when ready to plan.
          </p>
        </div>
        {hasNotes && (
          <button
            onClick={submitToPonder}
            disabled={submitting}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
          >
            {submitting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Send className="w-4 h-4" />
            )}
            Submit to Ponder
            <span className="ml-1 text-xs opacity-75">({notes.length})</span>
          </button>
        )}
      </div>

      {/* Compose area */}
      <div className="mb-4">
        <div className="relative">
          <textarea
            ref={textareaRef}
            value={draft}
            onChange={e => setDraft(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="What's on your mind? ⌘↵ to save"
            rows={4}
            className="w-full resize-none rounded-lg border border-border bg-card px-4 py-3 text-sm placeholder:text-muted-foreground/50 focus:outline-none focus:ring-1 focus:ring-ring"
          />
          <div className="absolute bottom-3 right-3 flex items-center gap-2">
            <span className="text-[11px] text-muted-foreground/40">⌘↵</span>
            <button
              onClick={saveNote}
              disabled={saving || !hasDraft}
              className="px-3 py-1 rounded-md bg-accent text-accent-foreground text-xs font-medium hover:bg-accent/80 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {saving ? <Loader2 className="w-3 h-3 animate-spin" /> : 'Save'}
            </button>
          </div>
        </div>
        {error && (
          <p className="mt-1.5 text-xs text-destructive">{error}</p>
        )}
      </div>

      {/* Notes list */}
      <div className="space-y-2">
        {notes.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground/40 select-none">
            <p className="text-sm">No notes yet.</p>
            <p className="text-xs mt-1">Start typing above.</p>
          </div>
        )}
        {notes.map(note => (
          <NoteCard
            key={note.id}
            note={note}
            onDelete={deleteNote}
            onEdit={onEdit}
            onEditError={onEditError}
            onEnrich={onEnrich}
            editingId={editingId}
            setEditingId={setEditingId}
          />
        ))}
      </div>
    </div>
  )
}

function NoteCard({
  note,
  onDelete,
  onEdit,
  onEditError,
  onEnrich,
  editingId,
  setEditingId,
}: {
  note: FeedbackNote
  onDelete: (id: string) => void
  onEdit: (id: string, newContent: string) => void
  onEditError: (id: string, originalContent: string) => void
  onEnrich: (id: string, updatedNote: FeedbackNote) => void
  editingId: string | null
  setEditingId: (id: string | null) => void
}) {
  const [confirming, setConfirming] = useState(false)
  const [editDraft, setEditDraft] = useState(note.content)
  const [editError, setEditError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const editTextareaRef = useRef<HTMLTextAreaElement>(null)
  const [enriching, setEnriching] = useState(false)
  const [enrichDraft, setEnrichDraft] = useState('')
  const [enrichSaving, setEnrichSaving] = useState(false)
  const [enrichError, setEnrichError] = useState<string | null>(null)
  const enrichTextareaRef = useRef<HTMLTextAreaElement>(null)

  const isEditing = editingId === note.id

  // When another card opens edit mode, cancel ours without saving
  useEffect(() => {
    if (!isEditing) {
      setEditDraft(note.content)
      setEditError(null)
    }
  }, [isEditing, note.content])

  // Auto-focus textarea when edit mode opens
  useEffect(() => {
    if (isEditing) {
      editTextareaRef.current?.focus()
      // Move cursor to end
      const ta = editTextareaRef.current
      if (ta) {
        ta.setSelectionRange(ta.value.length, ta.value.length)
      }
    }
  }, [isEditing])

  function openEdit() {
    setEditDraft(note.content)
    setEditError(null)
    setEditingId(note.id)
  }

  function cancelEdit() {
    setEditDraft(note.content)
    setEditError(null)
    setEditingId(null)
  }

  async function saveEdit() {
    const trimmed = editDraft.trim()
    if (!trimmed) return
    const originalContent = note.content
    setSaving(true)
    setEditError(null)
    // Optimistic update
    onEdit(note.id, trimmed)
    setEditingId(null)
    try {
      await api.updateFeedbackNote(note.id, trimmed)
    } catch (e) {
      // Restore on error
      onEditError(note.id, originalContent)
      setEditDraft(originalContent)
      setEditError(e instanceof Error ? e.message : 'Failed to save')
      setEditingId(note.id)
    } finally {
      setSaving(false)
    }
  }

  function handleEditKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      saveEdit()
    } else if (e.key === 'Escape') {
      e.preventDefault()
      cancelEdit()
    }
  }

  function openEnrich() {
    setEnrichDraft('')
    setEnrichError(null)
    setEnriching(true)
    setTimeout(() => enrichTextareaRef.current?.focus(), 0)
  }

  function cancelEnrich() {
    setEnrichDraft('')
    setEnrichError(null)
    setEnriching(false)
  }

  async function saveEnrich() {
    const trimmed = enrichDraft.trim()
    if (!trimmed) return
    setEnrichSaving(true)
    setEnrichError(null)
    try {
      const updated = await api.enrichFeedbackNote(note.id, trimmed, 'user')
      onEnrich(note.id, updated)
      setEnrichDraft('')
      setEnriching(false)
    } catch (e) {
      setEnrichError(e instanceof Error ? e.message : 'Failed to save')
    } finally {
      setEnrichSaving(false)
    }
  }

  function handleEnrichKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      saveEnrich()
    } else if (e.key === 'Escape') {
      e.preventDefault()
      cancelEnrich()
    }
  }

  function handleDelete() {
    if (!confirming) {
      setConfirming(true)
      return
    }
    onDelete(note.id)
  }

  const date = new Date(note.created_at)
  const timeStr = date.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })

  const updatedStr = note.updated_at
    ? new Date(note.updated_at).toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: 'numeric',
        minute: '2-digit',
      })
    : null

  const isEditDraftEmpty = editDraft.trim().length === 0

  return (
    <div
      className="group relative rounded-lg border border-border bg-card px-4 py-3 text-sm"
      onMouseLeave={() => { if (!isEditing) setConfirming(false) }}
      onDoubleClick={() => { if (!isEditing) openEdit() }}
    >
      {isEditing ? (
        /* Edit mode */
        <>
          <textarea
            ref={editTextareaRef}
            value={editDraft}
            onChange={e => setEditDraft(e.target.value)}
            onKeyDown={handleEditKeyDown}
            rows={Math.max(3, editDraft.split('\n').length)}
            className="w-full resize-none rounded-md border border-ring bg-background px-3 py-2 text-sm leading-relaxed text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
          />
          {editError && (
            <p className="mt-1 text-xs text-destructive">{editError}</p>
          )}
          <div className="mt-2 flex items-center justify-end gap-2">
            <button
              onClick={cancelEdit}
              className="px-3 py-1 rounded-md text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={saveEdit}
              disabled={isEditDraftEmpty || saving}
              className="px-3 py-1 rounded-md bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {saving ? <Loader2 className="w-3 h-3 animate-spin" /> : 'Save'}
            </button>
          </div>
          <p className="mt-2 text-[11px] text-muted-foreground/40">
            {note.id} · {timeStr}{updatedStr ? ` · edited ${updatedStr}` : ''}
          </p>
        </>
      ) : (
        /* Display mode */
        <>
          <div className="flex items-start justify-between gap-3">
            <pre className="flex-1 whitespace-pre-wrap font-sans text-sm leading-relaxed text-foreground/90">
              {note.content}
            </pre>
            <div className="flex items-center gap-1 shrink-0 mt-0.5 opacity-0 group-hover:opacity-100">
              <button
                onClick={e => { e.stopPropagation(); openEnrich() }}
                title="Add context"
                className="p-1 rounded transition-colors text-muted-foreground hover:text-foreground hover:bg-accent"
              >
                <Plus className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={e => { e.stopPropagation(); openEdit() }}
                title="Edit note"
                className="p-1 rounded transition-colors text-muted-foreground hover:text-foreground hover:bg-accent"
              >
                <Pencil className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={handleDelete}
                title={confirming ? 'Click again to confirm' : 'Delete note'}
                className={`p-1 rounded transition-colors ${
                  confirming
                    ? 'text-destructive bg-destructive/10 opacity-100'
                    : 'text-muted-foreground hover:text-destructive hover:bg-destructive/10'
                }`}
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
          {/* Enrichment blocks */}
          {note.enrichments && note.enrichments.length > 0 && (
            <div className="mt-2 space-y-1.5 border-t border-border pt-2">
              {note.enrichments.map((e, i) => (
                <div key={i} className="flex gap-2 rounded-md bg-muted/40 px-2.5 py-1.5 border-l-2 border-primary/30">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5 mb-0.5">
                      <span className="inline-block rounded px-1.5 py-0.5 text-[10px] font-medium bg-primary/10 text-primary leading-none">
                        {e.source}
                      </span>
                      <span className="text-[10px] text-muted-foreground/50">
                        {new Date(e.added_at).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' })}
                      </span>
                    </div>
                    <p className="text-xs text-foreground/80 leading-relaxed whitespace-pre-wrap">{e.content}</p>
                  </div>
                </div>
              ))}
            </div>
          )}
          {/* Enrich textarea */}
          {enriching && (
            <div className="mt-2 border-t border-border pt-2">
              <textarea
                ref={enrichTextareaRef}
                value={enrichDraft}
                onChange={e => setEnrichDraft(e.target.value)}
                onKeyDown={handleEnrichKeyDown}
                placeholder="Add context… ⌘↵ to save, Esc to cancel"
                rows={2}
                className="w-full resize-none rounded-md border border-ring bg-background px-3 py-2 text-xs leading-relaxed text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              />
              {enrichError && <p className="mt-1 text-xs text-destructive">{enrichError}</p>}
              <div className="mt-1.5 flex items-center justify-end gap-2">
                <button
                  onClick={cancelEnrich}
                  className="px-3 py-1 rounded-md text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={saveEnrich}
                  disabled={enrichDraft.trim().length === 0 || enrichSaving}
                  className="px-3 py-1 rounded-md bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  {enrichSaving ? <Loader2 className="w-3 h-3 animate-spin" /> : 'Save'}
                </button>
              </div>
            </div>
          )}
          <p className="mt-2 text-[11px] text-muted-foreground/40">
            {note.id} · {timeStr}{updatedStr ? ` · edited ${updatedStr}` : ''}
          </p>
        </>
      )}
    </div>
  )
}
