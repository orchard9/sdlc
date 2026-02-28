import { useState, useEffect, useRef, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { Trash2, Send, Loader2 } from 'lucide-react'
import { api } from '@/api/client'
import type { FeedbackNote } from '@/lib/types'

export function FeedbackPage() {
  const navigate = useNavigate()
  const [notes, setNotes] = useState<FeedbackNote[]>([])
  const [draft, setDraft] = useState('')
  const [saving, setSaving] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
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
    <div className="h-full flex flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border shrink-0">
        <div>
          <h1 className="text-lg font-semibold">Feedback</h1>
          <p className="text-xs text-muted-foreground mt-0.5">
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
      <div className="px-6 pt-4 pb-3 shrink-0">
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
      <div className="flex-1 overflow-y-auto px-6 pb-6 space-y-2">
        {notes.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground/40 select-none">
            <p className="text-sm">No notes yet.</p>
            <p className="text-xs mt-1">Start typing above.</p>
          </div>
        )}
        {notes.map(note => (
          <NoteCard key={note.id} note={note} onDelete={deleteNote} />
        ))}
      </div>
    </div>
  )
}

function NoteCard({
  note,
  onDelete,
}: {
  note: FeedbackNote
  onDelete: (id: string) => void
}) {
  const [confirming, setConfirming] = useState(false)

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

  return (
    <div
      className="group relative rounded-lg border border-border bg-card px-4 py-3 text-sm"
      onMouseLeave={() => setConfirming(false)}
    >
      <div className="flex items-start justify-between gap-3">
        <pre className="flex-1 whitespace-pre-wrap font-sans text-sm leading-relaxed text-foreground/90">
          {note.content}
        </pre>
        <button
          onClick={handleDelete}
          title={confirming ? 'Click again to confirm' : 'Delete note'}
          className={`shrink-0 mt-0.5 p-1 rounded transition-colors opacity-0 group-hover:opacity-100 ${
            confirming
              ? 'text-destructive bg-destructive/10 opacity-100'
              : 'text-muted-foreground hover:text-destructive hover:bg-destructive/10'
          }`}
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
      <p className="mt-2 text-[11px] text-muted-foreground/40">
        {note.id} · {timeStr}
      </p>
    </div>
  )
}
