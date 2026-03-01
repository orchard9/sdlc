import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '@/api/client'
import { ToolCard } from '@/components/tools/ToolCard'
import { AmaThreadPanel } from '@/components/tools/AmaThreadPanel'
import { AmaAnswerPanel } from '@/components/tools/AmaAnswerPanel'
import { QualityCheckPanel } from '@/components/tools/QualityCheckPanel'
import { ToolUsageSection } from '@/components/tools/ToolUsageSection'
import { Loader2, AlertTriangle, Play, Plus, Wrench, RefreshCw, X, Copy, Sparkles, History, Trash2, Zap, ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ToolMeta, ToolResult, QualityCheckData, CheckResult, ToolInteractionRecord, ResultAction } from '@/lib/types'

// ---------------------------------------------------------------------------
// CreateToolModal — Plan-Act Pattern
// ---------------------------------------------------------------------------

interface CreateToolModalProps {
  onClose: () => void
  onCreated: () => void
}

const TOOL_NAME_RE = /^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/

type CreateStep = 'form' | 'planning' | 'adjusting' | 'building'

function CreateToolModal({ onClose, onCreated }: CreateToolModalProps) {
  const [step, setStep] = useState<CreateStep>('form')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [requirements, setRequirements] = useState('')
  const [planRunKey, setPlanRunKey] = useState<string | null>(null)
  const [planText, setPlanText] = useState('')
  const [adjustments, setAdjustments] = useState('')
  const [buildRunKey, setBuildRunKey] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const nameRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (step === 'form') nameRef.current?.focus()
  }, [step])

  const nameInvalid = name.length > 0 && !TOOL_NAME_RE.test(name)
  const canPlan = name.length > 0 && description.trim().length > 0 && !nameInvalid && !submitting

  const handlePlan = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canPlan) return
    setSubmitting(true)
    setError(null)
    try {
      const resp = await api.planTool({
        name: name.trim(),
        description: description.trim(),
        requirements: requirements.trim() || undefined,
      })
      setPlanRunKey(resp.run_key)
      setStep('planning')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start planning')
    } finally {
      setSubmitting(false)
    }
  }

  const handleBuild = async () => {
    setSubmitting(true)
    setError(null)
    const fullPlan = adjustments.trim()
      ? `${planText}\n\n## Adjustments\n\n${adjustments.trim()}`
      : planText
    try {
      const resp = await api.buildTool({ name: name.trim(), plan: fullPlan })
      setBuildRunKey(resp.run_key)
      setStep('building')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start build')
    } finally {
      setSubmitting(false)
    }
  }

  const stepLabel: Record<CreateStep, string> = {
    form: 'New Tool',
    planning: 'Planning…',
    adjusting: 'Review Plan',
    building: 'Building…',
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-2xl bg-card border border-border rounded-xl shadow-2xl p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold">{stepLabel[step]}</h2>
          {step !== 'building' && (
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>

        {/* Step: form */}
        {step === 'form' && (
          <form onSubmit={handlePlan} className="space-y-4">
            <div className="space-y-1.5">
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Name <span className="normal-case text-muted-foreground/50">(slug — lowercase, numbers, hyphens)</span>
              </label>
              <input
                ref={nameRef}
                type="text"
                value={name}
                onChange={e => setName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
                placeholder="my-tool"
                className={cn(
                  'w-full px-3 py-2 text-sm bg-muted/40 border rounded-lg outline-none transition-colors placeholder:text-muted-foreground/50',
                  nameInvalid
                    ? 'border-red-500/50 focus:border-red-500'
                    : 'border-border focus:border-primary/50',
                )}
              />
              {nameInvalid && <p className="text-xs text-red-400">Must start and end with a letter or digit</p>}
            </div>

            <div className="space-y-1.5">
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Description
              </label>
              <input
                type="text"
                value={description}
                onChange={e => setDescription(e.target.value)}
                placeholder="What does this tool do?"
                className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50"
              />
            </div>

            <div className="space-y-1.5">
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Requirements <span className="normal-case text-muted-foreground/50">(optional)</span>
              </label>
              <textarea
                value={requirements}
                onChange={e => setRequirements(e.target.value)}
                placeholder="Any specific inputs, outputs, or constraints…"
                rows={2}
                className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50 resize-none"
              />
            </div>

            {error && (
              <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-sm text-red-300">{error}</p>
              </div>
            )}

            <div className="flex items-center justify-end gap-2 pt-1">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={!canPlan}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                {submitting ? 'Starting…' : 'Plan with AI'}
              </button>
            </div>
          </form>
        )}

        {/* Step: planning — stream plan output */}
        {step === 'planning' && planRunKey && (
          <AmaAnswerPanel
            runKey={planRunKey}
            onDone={text => {
              setPlanText(text)
              setStep('adjusting')
            }}
          />
        )}

        {/* Step: adjusting — review plan, optionally tweak, then build */}
        {step === 'adjusting' && (
          <div className="space-y-4">
            <p className="text-xs text-muted-foreground">
              Review the plan below. Add adjustments if needed, then click <strong>Build It</strong>.
            </p>

            <div className="space-y-1.5">
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Adjustments <span className="normal-case text-muted-foreground/50">(optional)</span>
              </label>
              <textarea
                value={adjustments}
                onChange={e => setAdjustments(e.target.value)}
                placeholder="Change the input schema to… / Add support for… / Remove…"
                rows={3}
                className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50 resize-none"
              />
            </div>

            {error && (
              <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-sm text-red-300">{error}</p>
              </div>
            )}

            <div className="flex items-center justify-end gap-2 pt-1">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleBuild}
                disabled={submitting}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                {submitting ? 'Starting…' : 'Build It'}
              </button>
            </div>
          </div>
        )}

        {/* Step: building — stream build output */}
        {step === 'building' && buildRunKey && (
          <AmaAnswerPanel
            runKey={buildRunKey}
            onDone={() => {
              onCreated()
              onClose()
            }}
          />
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// CloneToolModal — copy a built-in tool to a new user-owned name
// ---------------------------------------------------------------------------

interface CloneToolModalProps {
  sourceName: string
  onClose: () => void
  onCloned: (newName: string) => void
}

function CloneToolModal({ sourceName, onClose, onCloned }: CloneToolModalProps) {
  const [newName, setNewName] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => { inputRef.current?.focus() }, [])

  const nameInvalid = newName.length > 0 && !TOOL_NAME_RE.test(newName)
  const canSubmit = newName.length > 0 && !nameInvalid && !submitting

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    setSubmitting(true)
    setError(null)
    try {
      await api.cloneTool(sourceName, { new_name: newName.trim() })
      onCloned(newName.trim())
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Clone failed')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md bg-card border border-border rounded-xl shadow-2xl p-6">
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold">Clone "{sourceName}"</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Creates an editable copy of <code className="font-mono text-xs">{sourceName}</code> that you can modify without it being overwritten on re-init.
          </p>

          <div className="space-y-1.5">
            <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
              New name <span className="normal-case text-muted-foreground/50">(slug)</span>
            </label>
            <input
              ref={inputRef}
              type="text"
              value={newName}
              onChange={e => setNewName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
              placeholder={`my-${sourceName}`}
              className={cn(
                'w-full px-3 py-2 text-sm bg-muted/40 border rounded-lg outline-none transition-colors placeholder:text-muted-foreground/50',
                nameInvalid
                  ? 'border-red-500/50 focus:border-red-500'
                  : 'border-border focus:border-primary/50',
              )}
            />
            {nameInvalid && <p className="text-xs text-red-400">Must start and end with a letter or digit</p>}
          </div>

          {error && (
            <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
              <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
              <p className="text-sm text-red-300">{error}</p>
            </div>
          )}

          <div className="flex items-center justify-end gap-2 pt-1">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!canSubmit}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              {submitting ? 'Cloning…' : 'Clone'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// EvolveModal — describe a change, stream agent output
// ---------------------------------------------------------------------------

interface EvolveModalProps {
  toolName: string
  onClose: () => void
  onEvolved: () => void
}

type EvolveStep = 'form' | 'evolving'

function EvolveModal({ toolName, onClose, onEvolved }: EvolveModalProps) {
  const [step, setStep] = useState<EvolveStep>('form')
  const [changeRequest, setChangeRequest] = useState('')
  const [evolveRunKey, setEvolveRunKey] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (step === 'form') textareaRef.current?.focus()
  }, [step])

  const canSubmit = changeRequest.trim().length > 0 && !submitting

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    setSubmitting(true)
    setError(null)
    try {
      const resp = await api.evolveTool(toolName, changeRequest.trim())
      setEvolveRunKey(resp.run_key)
      setStep('evolving')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start evolve')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-2xl bg-card border border-border rounded-xl shadow-2xl p-6">
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold">
            {step === 'form' ? `Evolve "${toolName}"` : 'Evolving…'}
          </h2>
          {step !== 'evolving' && (
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>

        {step === 'form' && (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-1.5">
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                What would you like to change?
              </label>
              <textarea
                ref={textareaRef}
                value={changeRequest}
                onChange={e => setChangeRequest(e.target.value)}
                placeholder="Add a --dry-run flag that prints what would change without executing… / Change the output schema to include a summary field… / Fix the error handling when the API times out…"
                rows={4}
                className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50 resize-none"
              />
            </div>

            {error && (
              <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-sm text-red-300">{error}</p>
              </div>
            )}

            <div className="flex items-center justify-end gap-2 pt-1">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={!canSubmit}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                {submitting ? 'Starting…' : 'Evolve'}
              </button>
            </div>
          </form>
        )}

        {step === 'evolving' && evolveRunKey && (
          <AmaAnswerPanel
            runKey={evolveRunKey}
            onDone={() => {
              onEvolved()
              onClose()
            }}
          />
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

function formatRelative(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime()
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return 'just now'
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h ago`
  const days = Math.floor(hrs / 24)
  return `${days}d ago`
}

function evalCondition(condition: string | undefined, result: unknown): boolean {
  if (!condition) return true
  try {
    const match = condition.match(/^\$\.?([\w.]+)\s*(==|!=|>=|<=|>|<)\s*(.+)$/)
    if (!match) return true
    const [, pathStr, op, rhsStr] = match
    const parts = pathStr.split('.')
    let val: unknown = result
    for (const part of parts) {
      if (val === null || val === undefined || typeof val !== 'object') return true
      val = (val as Record<string, unknown>)[part]
    }
    let rhs: unknown
    if (rhsStr === 'null') rhs = null
    else if (rhsStr === 'true') rhs = true
    else if (rhsStr === 'false') rhs = false
    else if (/^-?\d+(\.\d+)?$/.test(rhsStr)) rhs = parseFloat(rhsStr)
    else rhs = rhsStr.replace(/^["']|["']$/g, '')
    switch (op) {
      case '==': return val === rhs
      case '!=': return val !== rhs
      case '>': return typeof val === 'number' && typeof rhs === 'number' && val > rhs
      case '<': return typeof val === 'number' && typeof rhs === 'number' && val < rhs
      case '>=': return typeof val === 'number' && typeof rhs === 'number' && val >= rhs
      case '<=': return typeof val === 'number' && typeof rhs === 'number' && val <= rhs
      default: return true
    }
  } catch {
    return true
  }
}

// ---------------------------------------------------------------------------
// InteractionRow — one recorded run in the History tab
// ---------------------------------------------------------------------------

interface InteractionRowProps {
  record: ToolInteractionRecord
  onDelete: (id: string) => void
}

function InteractionRow({ record, onDelete }: InteractionRowProps) {
  const [expanded, setExpanded] = useState(false)

  const inputSummary = (() => {
    if (!record.input || typeof record.input !== 'object') return '{}'
    const keys = Object.keys(record.input as Record<string, unknown>)
    if (keys.length === 0) return '{}'
    const firstKey = keys[0]
    const firstVal = (record.input as Record<string, unknown>)[firstKey]
    if (typeof firstVal === 'string' && firstVal.length > 0) {
      return `${firstKey}: "${firstVal.slice(0, 30)}"`
    }
    return `${keys.length} field${keys.length !== 1 ? 's' : ''}`
  })()

  const durationMs = record.created_at && record.completed_at
    ? Math.round(new Date(record.completed_at).getTime() - new Date(record.created_at).getTime())
    : null

  return (
    <div className="rounded-lg border border-border/50 bg-muted/5 overflow-hidden">
      <div
        className="flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-muted/20 transition-colors"
        onClick={() => setExpanded(e => !e)}
      >
        <span className={cn(
          'text-[10px] font-mono px-1.5 py-0.5 rounded border shrink-0',
          record.status === 'completed'
            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
            : record.status === 'failed'
              ? 'bg-red-500/10 border-red-500/30 text-red-400'
              : 'bg-amber-500/10 border-amber-500/30 text-amber-400',
        )}>
          {record.status}
        </span>
        <span className="flex-1 text-xs font-mono text-muted-foreground truncate">{inputSummary}</span>
        <span className="text-[10px] text-muted-foreground/50 shrink-0">{formatRelative(record.created_at)}</span>
        {durationMs !== null && (
          <span className="text-[10px] font-mono text-muted-foreground/40 shrink-0">{durationMs}ms</span>
        )}
        {expanded
          ? <ChevronDown className="w-3 h-3 text-muted-foreground/50 shrink-0" />
          : <ChevronRight className="w-3 h-3 text-muted-foreground/50 shrink-0" />
        }
        <button
          onClick={e => { e.stopPropagation(); onDelete(record.id) }}
          className="p-0.5 rounded hover:bg-red-500/20 text-muted-foreground/40 hover:text-red-400 transition-colors shrink-0"
          title="Delete record"
        >
          <Trash2 className="w-3 h-3" />
        </button>
      </div>
      {expanded && (
        <pre className="px-3 py-2 text-xs font-mono bg-muted/20 border-t border-border/30 overflow-x-auto whitespace-pre-wrap text-muted-foreground max-h-64 overflow-y-auto">
          {JSON.stringify(record.result ?? record.input, null, 2)}
        </pre>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Tool run panel (right side)
// ---------------------------------------------------------------------------

interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (selectName?: string) => void
}

function ToolRunPanel({ tool, onReload }: ToolRunPanelProps) {
  const [scope, setScope] = useState('')
  const [jsonInput, setJsonInput] = useState('{}')
  const [running, setRunning] = useState(false)
  const [settingUp, setSettingUp] = useState(false)
  const [reconfiguring, setReconfiguring] = useState(false)
  const [fixing, setFixing] = useState(false)
  const [result, setResult] = useState<ToolResult | null>(null)
  const [setupResult, setSetupResult] = useState<ToolResult | null>(null)
  const [setupDone, setSetupDone] = useState(false)
  const [reconfigureRunKey, setReconfigureRunKey] = useState<string | null>(null)
  const [fixRunKey, setFixRunKey] = useState<string | null>(null)
  const [showCloneModal, setShowCloneModal] = useState(false)
  const [showEvolveModal, setShowEvolveModal] = useState(false)
  const [activeTab, setActiveTab] = useState<'run' | 'history'>('run')
  const [interactions, setInteractions] = useState<ToolInteractionRecord[]>([])
  const [historyLoading, setHistoryLoading] = useState(false)
  const [actRunKey, setActRunKey] = useState<string | null>(null)
  const [lastInput, setLastInput] = useState<unknown>(null)

  // Reset state when the selected tool changes
  useEffect(() => {
    setScope('')
    setJsonInput('{}')
    setResult(null)
    setSetupResult(null)
    setSetupDone(false)
    setReconfigureRunKey(null)
    setFixRunKey(null)
    setShowCloneModal(false)
    setShowEvolveModal(false)
    setActiveTab('run')
    setInteractions([])
    setActRunKey(null)
    setLastInput(null)
  }, [tool.name])

  const loadHistory = async () => {
    setHistoryLoading(true)
    try {
      const records = await api.listToolInteractions(tool.name)
      setInteractions(records)
    } catch {
      // non-fatal
    } finally {
      setHistoryLoading(false)
    }
  }

  const handleSetup = async () => {
    setSettingUp(true)
    setSetupResult(null)
    try {
      const res = await api.setupTool(tool.name)
      setSetupResult(res)
      if (res.ok) setSetupDone(true)
    } catch (e) {
      setSetupResult({ ok: false, error: e instanceof Error ? e.message : 'Setup failed' })
    } finally {
      setSettingUp(false)
    }
  }

  const handleRun = async () => {
    setRunning(true)
    setResult(null)
    setActRunKey(null)
    try {
      let input: unknown
      if (tool.name === 'quality-check') {
        input = scope.trim() ? { scope: scope.trim() } : {}
      } else {
        try {
          input = JSON.parse(jsonInput || '{}')
        } catch {
          setResult({ ok: false, error: 'Invalid JSON input' })
          setRunning(false)
          return
        }
      }
      setLastInput(input)
      const res = await api.runTool(tool.name, input)
      setResult(res)
      // Refresh history in the background after each run
      api.listToolInteractions(tool.name).then(records => setInteractions(records)).catch(() => {})
    } catch (e) {
      setResult({ ok: false, error: e instanceof Error ? e.message : 'Run failed' })
    } finally {
      setRunning(false)
    }
  }

  const handleReconfigure = async () => {
    setReconfiguring(true)
    setReconfigureRunKey(null)
    try {
      const resp = await api.reconfigureQualityGates()
      setReconfigureRunKey(resp.run_key)
    } catch {
      // non-fatal — show nothing
    } finally {
      setReconfiguring(false)
    }
  }

  const handleFixIssues = async (failedChecks: CheckResult[]) => {
    setFixing(true)
    setFixRunKey(null)
    try {
      const resp = await api.fixQualityIssues(failedChecks)
      setFixRunKey(resp.run_key)
    } catch {
      // non-fatal — show nothing
    } finally {
      setFixing(false)
    }
  }

  const handleActTool = async (actionIndex: number, action: ResultAction) => {
    if (action.confirm && !window.confirm(action.confirm)) return
    setActRunKey(null)
    try {
      const resp = await api.actTool(tool.name, {
        action_index: actionIndex,
        result: result as unknown,
        input: lastInput,
      })
      setActRunKey(resp.run_key)
    } catch {
      // non-fatal
    }
  }

  const canRun = true

  return (
    <div className="flex flex-col h-full min-h-0 overflow-y-auto">
      {/* Header */}
      <div className="shrink-0 px-5 pt-5 pb-4 border-b border-border/50">
        <div className="flex items-start gap-3">
          <div className="flex-1 min-w-0">
            <h2 className="text-base font-semibold">{tool.display_name}</h2>
            <p className="text-sm text-muted-foreground mt-0.5">{tool.description}</p>
          </div>
          <span className="shrink-0 text-xs font-mono bg-muted/60 border border-border/50 rounded px-2 py-0.5 text-muted-foreground">
            v{tool.version}
          </span>
        </div>
      </div>

      <div className="flex-1 px-5 py-4 space-y-4">
        {/* Usage: agent hint + CLI command + full reference */}
        <ToolUsageSection tool={tool} />

        {/* Setup banner */}
        {tool.requires_setup && !tool.setup_done && !setupDone && (
          <div className="flex items-start gap-3 px-4 py-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
            <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-amber-300">Setup required</p>
              {tool.setup_description && (
                <p className="text-xs text-amber-400/70 mt-0.5">{tool.setup_description}</p>
              )}
              {tool.name === 'quality-check' && (
                <p className="text-xs text-amber-400/60 mt-1">
                  Click <span className="font-medium text-amber-400/80">Reconfigure</span> below — detects your stack and installs a two-phase hook: auto-fix first (gofmt, prettier, rustfmt, ruff), then verify.
                </p>
              )}
              {setupResult && (
                <p className={cn(
                  'text-xs mt-1.5 font-mono',
                  setupResult.ok ? 'text-emerald-400' : 'text-red-400',
                )}>
                  {setupResult.ok
                    ? (() => {
                        const d = setupResult.data as Record<string, unknown> | undefined
                        if (d && 'files_indexed' in d) {
                          return `✓ ${d.files_indexed} files indexed, ${d.total_chunks ?? 0} chunks`
                        }
                        return '✓ Setup complete'
                      })()
                    : `Error: ${setupResult.error}`
                  }
                </p>
              )}
            </div>
            {tool.name !== 'quality-check' && (
              <button
                onClick={handleSetup}
                disabled={settingUp}
                className="shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 rounded-lg border border-amber-500/30 transition-colors disabled:opacity-50"
              >
                {settingUp && <Loader2 className="w-3 h-3 animate-spin" />}
                {settingUp ? 'Running…' : 'Run Setup'}
              </button>
            )}
          </div>
        )}

        {/* AMA: threaded question panel */}
        {tool.name === 'ama' ? (
          <AmaThreadPanel tool={tool} />
        ) : (
          <>
            {/* Tab switcher */}
            <div className="flex items-center gap-1">
              <button
                onClick={() => setActiveTab('run')}
                className={cn(
                  'px-3 py-1.5 text-xs font-medium rounded-lg transition-colors',
                  activeTab === 'run'
                    ? 'bg-muted text-foreground'
                    : 'text-muted-foreground hover:text-foreground',
                )}
              >
                Run
              </button>
              <button
                onClick={() => {
                  setActiveTab('history')
                  if (interactions.length === 0) loadHistory()
                }}
                className={cn(
                  'flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-colors',
                  activeTab === 'history'
                    ? 'bg-muted text-foreground'
                    : 'text-muted-foreground hover:text-foreground',
                )}
              >
                <History className="w-3 h-3" />
                History
                {interactions.length > 0 && (
                  <span className="ml-0.5 text-[10px] font-mono bg-muted-foreground/20 px-1.5 py-0.5 rounded-full">
                    {interactions.length}
                  </span>
                )}
              </button>
            </div>

            {/* Run tab */}
            {activeTab === 'run' && (
              <>
                {/* Input area */}
                <div className="space-y-2">
                  {tool.name === 'quality-check' && (
                    <>
                      <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">Scope <span className="normal-case text-muted-foreground/50">(optional — filter by name)</span></label>
                      <input
                        type="text"
                        value={scope}
                        onChange={e => setScope(e.target.value)}
                        placeholder="e.g. test"
                        className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50"
                      />
                    </>
                  )}
                  {tool.name !== 'quality-check' && (
                    <>
                      <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">JSON Input</label>
                      <textarea
                        value={jsonInput}
                        onChange={e => setJsonInput(e.target.value)}
                        rows={4}
                        className="w-full px-3 py-2 text-sm font-mono bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50 resize-none"
                      />
                    </>
                  )}
                </div>

                {/* Run button (+ Reconfigure for quality-check) */}
                <div className="flex items-center gap-2 flex-wrap">
                  <button
                    onClick={handleRun}
                    disabled={running || !canRun}
                    className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {running
                      ? <Loader2 className="w-4 h-4 animate-spin" />
                      : <Play className="w-4 h-4" />
                    }
                    {running ? 'Running…' : 'Run'}
                  </button>
                  {tool.name === 'quality-check' && (
                    <button
                      onClick={handleReconfigure}
                      disabled={reconfiguring}
                      title="Detect project stack and reconfigure quality gates"
                      className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                      {reconfiguring
                        ? <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        : <RefreshCw className="w-3.5 h-3.5" />
                      }
                      {reconfiguring ? 'Reconfiguring…' : 'Reconfigure'}
                    </button>
                  )}
                  {tool.built_in && (
                    <button
                      onClick={() => setShowCloneModal(true)}
                      title="Clone to an editable copy"
                      className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
                    >
                      <Copy className="w-3.5 h-3.5" />
                      Clone
                    </button>
                  )}
                  {!tool.built_in && (
                    <button
                      onClick={() => setShowEvolveModal(true)}
                      title="Evolve this tool with AI"
                      className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors"
                    >
                      <Sparkles className="w-3.5 h-3.5" />
                      Evolve
                    </button>
                  )}
                </div>

                {/* Clone modal */}
                {showCloneModal && (
                  <CloneToolModal
                    sourceName={tool.name}
                    onClose={() => setShowCloneModal(false)}
                    onCloned={(newName) => {
                      setShowCloneModal(false)
                      onReload(newName)
                    }}
                  />
                )}

                {/* Evolve modal */}
                {showEvolveModal && (
                  <EvolveModal
                    toolName={tool.name}
                    onClose={() => setShowEvolveModal(false)}
                    onEvolved={() => onReload(tool.name)}
                  />
                )}

                {/* Reconfigure streaming output */}
                {reconfigureRunKey && (
                  <AmaAnswerPanel runKey={reconfigureRunKey} />
                )}

                {/* Fix Issues streaming output */}
                {fixRunKey && (
                  <AmaAnswerPanel runKey={fixRunKey} />
                )}

                {/* Results */}
                {result && (
                  <div className="mt-2">
                    {tool.name === 'quality-check' && result.data ? (
                      // Show structured check results even when ok:false (checks ran, some failed)
                      <>
                        <QualityCheckPanel
                          data={result.data as QualityCheckData}
                          onFixIssues={handleFixIssues}
                          fixing={fixing}
                        />
                        {fixRunKey && null /* streaming panel shown above */}
                      </>
                    ) : !result.ok ? (
                      <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
                        <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                        <p className="text-sm text-red-300">{result.error}</p>
                      </div>
                    ) : (
                      <pre className="text-xs font-mono bg-muted/30 border border-border rounded-lg p-3 overflow-x-auto whitespace-pre-wrap text-muted-foreground">
                        {JSON.stringify(result, null, 2)}
                      </pre>
                    )}
                    {result.ok && result.duration_ms !== undefined && (
                      <p className="text-xs text-muted-foreground/50 mt-2">{result.duration_ms}ms</p>
                    )}
                  </div>
                )}

                {/* Result action buttons — driven by tool's result_actions declaration */}
                {result?.ok && tool.result_actions && tool.result_actions.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {tool.result_actions.map((action, i) =>
                      evalCondition(action.condition, result) ? (
                        <button
                          key={i}
                          onClick={() => handleActTool(i, action)}
                          disabled={!!actRunKey}
                          className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded-lg border border-border/50 transition-colors disabled:opacity-50"
                        >
                          <Zap className="w-3.5 h-3.5" />
                          {action.label}
                        </button>
                      ) : null
                    )}
                  </div>
                )}

                {/* Result action streaming output */}
                {actRunKey && <AmaAnswerPanel runKey={actRunKey} />}
              </>
            )}

            {/* History tab */}
            {activeTab === 'history' && (
              <div className="space-y-2">
                {historyLoading ? (
                  <div className="flex items-center justify-center py-8">
                    <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
                  </div>
                ) : interactions.length === 0 ? (
                  <div className="text-center py-8">
                    <p className="text-sm text-muted-foreground">No runs recorded yet.</p>
                    <p className="text-xs text-muted-foreground/60 mt-1">Run the tool to record history.</p>
                  </div>
                ) : (
                  interactions.map(record => (
                    <InteractionRow
                      key={record.id}
                      record={record}
                      onDelete={async (id) => {
                        await api.deleteToolInteraction(tool.name, id).catch(() => {})
                        setInteractions(prev => prev.filter(r => r.id !== id))
                      }}
                    />
                  ))
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function ToolsPage() {
  const [tools, setTools] = useState<ToolMeta[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedName, setSelectedName] = useState<string | null>(null)
  const [showCreateModal, setShowCreateModal] = useState(false)

  const load = useCallback((selectAfterLoad?: string) => {
    api.listTools()
      .then(data => {
        setTools(data)
        if (selectAfterLoad) {
          setSelectedName(selectAfterLoad)
        } else if (data.length > 0 && !selectedName) {
          setSelectedName(data[0].name)
        }
      })
      .catch(err => setError(err.message))
      .finally(() => setLoading(false))
  }, [selectedName])

  useEffect(() => { load() }, [load])

  const selectedTool = tools.find(t => t.name === selectedName) ?? null

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    )
  }

  return (
    <>
      {showCreateModal && (
        <CreateToolModal
          onClose={() => setShowCreateModal(false)}
          onCreated={() => {
            setLoading(true)
            load()
          }}
        />

      )}

    <div className="h-full flex overflow-hidden">
      {/* Left pane: tool list */}
      <div className={cn(
        'w-64 shrink-0 border-r border-border flex flex-col bg-card',
        selectedTool ? 'hidden md:flex' : 'flex',
      )}>
        <div className="px-3 pt-4 pb-3 border-b border-border/50">
          <div className="flex items-center gap-2">
            <Wrench className="w-4 h-4 text-muted-foreground" />
            <h2 className="text-base font-semibold">Tools</h2>
            <span className="ml-auto text-xs text-muted-foreground">{tools.length}</span>
            <button
              onClick={() => setShowCreateModal(true)}
              title="Scaffold a new tool"
              className="p-1 rounded hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto px-2 py-2 space-y-0.5">
          {tools.length === 0 ? (
            <div className="px-3 py-4 text-center">
              <p className="text-xs text-muted-foreground">No tools installed.</p>
              <p className="text-xs text-muted-foreground/60 mt-1">Run <code className="font-mono">sdlc update</code></p>
            </div>
          ) : (
            tools.map(tool => (
              <ToolCard
                key={tool.name}
                tool={tool}
                selected={tool.name === selectedName}
                onSelect={() => setSelectedName(tool.name)}
              />
            ))
          )}
        </div>
      </div>

      {/* Right pane: tool detail + run UI */}
      <div className={cn(
        'flex-1 min-w-0',
        !selectedTool ? 'hidden md:flex items-center justify-center' : 'flex flex-col',
      )}>
        {selectedTool ? (
          <ToolRunPanel tool={selectedTool} onReload={(name) => { setLoading(true); load(name) }} />
        ) : (
          <div className="text-center">
            <Wrench className="w-8 h-8 text-muted-foreground/30 mx-auto mb-2" />
            <p className="text-sm text-muted-foreground">Select a tool to run it</p>
          </div>
        )}
      </div>
    </div>
    </>
  )
}
