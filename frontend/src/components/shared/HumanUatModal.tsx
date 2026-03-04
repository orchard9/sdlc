import { useEffect, useState } from 'react'
import { Loader2 } from 'lucide-react'
import { api } from '@/api/client'

type Verdict = 'pass' | 'pass_with_tasks' | 'failed'

interface HumanUatModalProps {
  open: boolean
  onClose: () => void
  mode: 'milestone' | 'feature'
  slug: string
}

const VERDICTS: { value: Verdict; label: string }[] = [
  { value: 'pass', label: 'Pass' },
  { value: 'pass_with_tasks', label: 'Pass with Tasks' },
  { value: 'failed', label: 'Fail' },
]

function notesRequired(verdict: Verdict | null): boolean {
  return verdict === 'pass_with_tasks' || verdict === 'failed'
}

export function HumanUatModal({ open, onClose, mode, slug }: HumanUatModalProps) {
  const [checklist, setChecklist] = useState<string | null>(null)
  const [checklistLoading, setChecklistLoading] = useState(false)
  const [verdict, setVerdict] = useState<Verdict | null>(null)
  const [notes, setNotes] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

  // Reset form and fetch checklist when modal opens
  useEffect(() => {
    if (!open) return
    setVerdict(null)
    setNotes('')
    setSubmitError(null)
    setChecklist(null)
    setChecklistLoading(true)

    const fetchChecklist = async () => {
      try {
        if (mode === 'feature') {
          const artifact = await api.getArtifact(slug, 'qa_plan')
          setChecklist(artifact.content ?? null)
        } else {
          const result = await api.getMilestoneAcceptanceTest(slug)
          setChecklist(result.content ?? null)
        }
      } catch {
        setChecklist(null)
      } finally {
        setChecklistLoading(false)
      }
    }

    fetchChecklist()
  }, [open, mode, slug])

  // Keyboard: Escape closes modal
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !submitting) onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose, submitting])

  if (!open) return null

  const required = notesRequired(verdict)
  const canSubmit = verdict !== null && (!required || notes.trim().length > 0)

  const handleSubmit = async () => {
    if (!canSubmit || submitting) return
    setSubmitting(true)
    setSubmitError(null)
    try {
      if (mode === 'milestone') {
        await api.submitHumanMilestoneUat(slug, { verdict: verdict!, notes })
      } else {
        await api.submitHumanFeatureQa(slug, { verdict: verdict!, notes })
      }
      onClose()
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : 'Submission failed. Please try again.')
    } finally {
      setSubmitting(false)
    }
  }

  const title = mode === 'milestone' ? 'Submit UAT Results' : 'Submit QA Results'

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      onClick={() => { if (!submitting) onClose() }}
    >
      <div
        className="w-full max-w-lg mx-4 bg-card border border-border rounded-2xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <span className="text-sm font-semibold">{title}</span>
          <button
            onClick={() => { if (!submitting) onClose() }}
            disabled={submitting}
            aria-label="Close"
            className="w-6 h-6 flex items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-40"
          >
            ✕
          </button>
        </div>

        {/* Body */}
        <div className="px-5 py-4 space-y-5">
          {/* Checklist */}
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
              {mode === 'milestone' ? 'Acceptance Test' : 'QA Plan'}
            </p>
            {checklistLoading ? (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Loading checklist…
              </div>
            ) : checklist ? (
              <pre className="bg-muted border border-border rounded-lg px-3 py-2.5 text-xs text-muted-foreground whitespace-pre-wrap max-h-36 overflow-y-auto leading-relaxed font-mono">
                {checklist}
              </pre>
            ) : (
              <p className="bg-muted border border-border rounded-lg px-3 py-2.5 text-xs text-muted-foreground italic">
                No checklist available — proceed with manual assessment.
              </p>
            )}
          </div>

          {/* Verdict */}
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
              Verdict
            </p>
            <div className="flex gap-2 flex-wrap">
              {VERDICTS.map(({ value, label }) => (
                <button
                  key={value}
                  type="button"
                  disabled={submitting}
                  onClick={() => setVerdict(value)}
                  className={[
                    'px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors',
                    verdict === value
                      ? value === 'pass'
                        ? 'border-green-500/50 bg-green-500/15 text-green-400'
                        : value === 'failed'
                          ? 'border-red-500/50 bg-red-500/15 text-red-400'
                          : 'border-primary/50 bg-primary/15 text-primary'
                      : 'border-border bg-muted text-muted-foreground hover:text-foreground hover:bg-muted/80',
                    submitting ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer',
                  ].join(' ')}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          {/* Notes */}
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2 flex items-center gap-1.5">
              Notes
              {required && (
                <span className="text-amber-400 font-normal normal-case tracking-normal">required</span>
              )}
              {!required && verdict !== null && (
                <span className="text-muted-foreground/60 font-normal normal-case tracking-normal">optional</span>
              )}
            </p>
            <textarea
              value={notes}
              onChange={e => setNotes(e.target.value)}
              disabled={submitting}
              placeholder="Describe what you tested and what you found…"
              rows={3}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring placeholder:text-muted-foreground resize-none disabled:opacity-50"
            />
          </div>

          {/* Error */}
          {submitError && (
            <p className="text-xs text-destructive bg-destructive/10 border border-destructive/20 rounded-lg px-3 py-2">
              {submitError}
            </p>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-border">
          <button
            onClick={() => { if (!submitting) onClose() }}
            disabled={submitting}
            className="px-4 py-1.5 text-sm text-muted-foreground hover:text-foreground border border-border rounded-lg hover:bg-muted transition-colors disabled:opacity-40"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!canSubmit || submitting}
            className="inline-flex items-center gap-1.5 px-4 py-1.5 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? (
              <>
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Submitting…
              </>
            ) : 'Submit Results'}
          </button>
        </div>
      </div>
    </div>
  )
}
