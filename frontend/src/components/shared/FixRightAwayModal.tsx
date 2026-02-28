import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Zap, AlertTriangle, ArrowLeft, FileCode } from 'lucide-react'
import { api } from '@/api/client'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import type { DiagnoseResult } from '@/lib/types'

interface FixRightAwayModalProps {
  open: boolean
  onClose: () => void
}

type Step = 'input' | 'diagnosing' | 'review' | 'creating'

function toFixSlug(title: string): string {
  return (
    'fix-' +
    title
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, '')
      .trim()
      .replace(/\s+/g, '-')
      .slice(0, 46)
  )
}

function ConfidenceBadge({ confidence }: { confidence: string }) {
  const map: Record<string, string> = {
    high: 'bg-green-500/15 text-green-400 border-green-500/30',
    medium: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
    low: 'bg-muted/60 text-muted-foreground border-border',
    none: 'bg-destructive/15 text-destructive border-destructive/30',
  }
  const cls = map[confidence] ?? map.low
  return (
    <span className={`text-xs px-1.5 py-0.5 rounded border font-mono ${cls}`}>
      {confidence}
    </span>
  )
}

export function FixRightAwayModal({ open, onClose }: FixRightAwayModalProps) {
  const navigate = useNavigate()
  const { setPanelOpen } = useAgentRuns()

  const [step, setStep] = useState<Step>('input')
  const [description, setDescription] = useState('')
  const [diagnosis, setDiagnosis] = useState<DiagnoseResult | null>(null)
  const [title, setTitle] = useState('')
  const [error, setError] = useState<string | null>(null)

  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const titleRef = useRef<HTMLInputElement>(null)

  // Reset on open
  useEffect(() => {
    if (open) {
      setStep('input')
      setDescription('')
      setDiagnosis(null)
      setTitle('')
      setError(null)
      setTimeout(() => textareaRef.current?.focus(), 0)
    }
  }, [open])

  // Focus title field when reaching review step
  useEffect(() => {
    if (step === 'review') {
      setTimeout(() => titleRef.current?.focus(), 0)
    }
  }, [step])

  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, onClose])

  const handleDiagnose = async () => {
    if (!description.trim()) return
    setError(null)
    setStep('diagnosing')
    try {
      const result = await api.diagnose(description)
      setDiagnosis(result)
      setTitle(toFixSlug(result.title))
      setStep('review')
    } catch (e) {
      // Diagnosis failed — fall back to raw description, skip to review
      const fallbackTitle = toFixSlug(description.trim().slice(0, 60))
      setDiagnosis({
        title: fallbackTitle.replace(/^fix-/, ''),
        problem_statement: description.trim(),
        root_cause: 'Diagnosis unavailable — proceeding with raw description.',
        files_affected: [],
        confidence: 'low',
      })
      setTitle(fallbackTitle)
      setStep('review')
      setError(e instanceof Error ? e.message : null)
    }
  }

  const handleFix = async () => {
    if (!title.trim()) return
    setError(null)
    setStep('creating')
    const slug = title.trim()
    const featureTitle = slug.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
    const context = diagnosis
      ? `Root cause: ${diagnosis.root_cause}\n\nProblem: ${diagnosis.problem_statement}\n\nOriginal report:\n${description}`
      : description
    try {
      await api.createFeature({ slug, title: featureTitle, description: context })
      await api.startRun(slug, context)
      setPanelOpen(true)
      navigate(`/features/${slug}`)
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create fix')
      setStep('review')
    }
  }

  if (!open) return null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Fix right away"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/60"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl mx-4 bg-card border border-border rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-4 pt-4 pb-3 border-b border-border">
          <div className="flex items-center gap-2 mb-0.5">
            <Zap className="w-4 h-4 text-primary" />
            <span className="text-sm font-semibold">Fix Right Away</span>
          </div>
          <p className="text-xs text-muted-foreground">
            {step === 'input' && 'Paste an error, stack trace, or describe what needs fixing.'}
            {step === 'diagnosing' && 'Reading files and diagnosing the issue…'}
            {step === 'review' && 'Review the diagnosis and confirm.'}
            {step === 'creating' && 'Creating feature and starting the agent…'}
          </p>
        </div>

        {/* Step: input */}
        {step === 'input' && (
          <div className="p-4 space-y-3">
            <textarea
              ref={textareaRef}
              value={description}
              onChange={e => setDescription(e.target.value)}
              onKeyDown={e => {
                if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') handleDiagnose()
              }}
              placeholder={`Paste a stack trace, error message, or describe the bug…`}
              rows={8}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring placeholder:text-muted-foreground resize-none font-mono"
            />
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">⌘↵ to diagnose</span>
              <button
                onClick={handleDiagnose}
                disabled={!description.trim()}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                <Zap className="w-3.5 h-3.5" />
                Diagnose
              </button>
            </div>
          </div>
        )}

        {/* Step: diagnosing */}
        {step === 'diagnosing' && (
          <div className="p-8 flex flex-col items-center gap-3 text-muted-foreground">
            <span className="w-5 h-5 border-2 border-muted-foreground/30 border-t-primary rounded-full animate-spin" />
            <p className="text-sm">Reading files and diagnosing…</p>
          </div>
        )}

        {/* Step: review */}
        {step === 'review' && diagnosis && (
          <div className="p-4 space-y-4">
            {/* Diagnosis card */}
            <div className="bg-muted/30 border border-border/60 rounded-lg p-3 space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Root cause</span>
                <ConfidenceBadge confidence={diagnosis.confidence} />
              </div>
              <p className="text-sm">{diagnosis.root_cause}</p>

              {diagnosis.confidence === 'none' && (
                <div className="flex items-start gap-2 mt-2 p-2 rounded bg-amber-500/10 border border-amber-500/20">
                  <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                  <p className="text-xs text-amber-300">
                    This doesn't look like a software issue. You can still proceed — a feature will be created with your description.
                  </p>
                </div>
              )}

              {diagnosis.problem_statement && diagnosis.problem_statement !== description && (
                <p className="text-xs text-muted-foreground border-t border-border/40 pt-2 mt-2">
                  {diagnosis.problem_statement}
                </p>
              )}

              {diagnosis.files_affected.length > 0 && (
                <div className="flex flex-wrap gap-1.5 pt-1">
                  {diagnosis.files_affected.map(f => (
                    <span key={f} className="flex items-center gap-1 text-xs font-mono bg-muted/60 text-muted-foreground px-1.5 py-0.5 rounded">
                      <FileCode className="w-3 h-3" />
                      {f}
                    </span>
                  ))}
                </div>
              )}
            </div>

            {/* Editable slug / feature title */}
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">Feature slug</label>
              <input
                ref={titleRef}
                type="text"
                value={title}
                onChange={e => setTitle(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-'))}
                onKeyDown={e => { if (e.key === 'Enter') handleFix() }}
                className="w-full px-3 py-2 text-sm font-mono bg-background border border-border rounded-lg outline-none focus:ring-1 focus:ring-ring"
              />
            </div>

            {error && <p className="text-xs text-destructive">{error}</p>}

            <div className="flex items-center justify-between">
              <button
                onClick={() => setStep('input')}
                className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                <ArrowLeft className="w-3.5 h-3.5" />
                Edit
              </button>
              <button
                onClick={handleFix}
                disabled={!title.trim()}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                <Zap className="w-3.5 h-3.5" />
                Fix it
              </button>
            </div>
          </div>
        )}

        {/* Step: creating */}
        {step === 'creating' && (
          <div className="p-8 flex flex-col items-center gap-3 text-muted-foreground">
            <span className="w-5 h-5 border-2 border-muted-foreground/30 border-t-primary rounded-full animate-spin" />
            <p className="text-sm">Starting agent…</p>
          </div>
        )}
      </div>
    </div>
  )
}
