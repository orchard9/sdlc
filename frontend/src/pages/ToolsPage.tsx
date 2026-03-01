import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '@/api/client'
import { ToolCard } from '@/components/tools/ToolCard'
import { AmaThreadPanel } from '@/components/tools/AmaThreadPanel'
import { AmaAnswerPanel } from '@/components/tools/AmaAnswerPanel'
import { QualityCheckPanel } from '@/components/tools/QualityCheckPanel'
import { ToolUsageSection } from '@/components/tools/ToolUsageSection'
import { Loader2, AlertTriangle, Play, Plus, Wrench, RefreshCw, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ToolMeta, ToolResult, QualityCheckData, CheckResult } from '@/lib/types'

// ---------------------------------------------------------------------------
// CreateToolModal
// ---------------------------------------------------------------------------

interface CreateToolModalProps {
  onClose: () => void
  onCreated: () => void
}

const TOOL_NAME_RE = /^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/

function CreateToolModal({ onClose, onCreated }: CreateToolModalProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const nameRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    nameRef.current?.focus()
  }, [])

  const nameInvalid = name.length > 0 && !TOOL_NAME_RE.test(name)
  const canSubmit = name.length > 0 && description.trim().length > 0 && !nameInvalid && !submitting

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    setSubmitting(true)
    setError(null)
    try {
      await api.createTool({ name: name.trim(), description: description.trim() })
      onCreated()
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create tool')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md bg-card border border-border rounded-xl shadow-2xl p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold">New Tool</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-muted/60 text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Name */}
          <div className="space-y-1.5">
            <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Name <span className="normal-case text-muted-foreground/50">(slug — lowercase letters, numbers, hyphens)</span>
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
            {nameInvalid && (
              <p className="text-xs text-red-400">Must start and end with a letter or digit</p>
            )}
          </div>

          {/* Description */}
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

          {/* Error */}
          {error && (
            <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
              <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
              <p className="text-sm text-red-300">{error}</p>
            </div>
          )}

          {/* Actions */}
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
              {submitting ? 'Creating…' : 'Create Tool'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Tool run panel (right side)
// ---------------------------------------------------------------------------

interface ToolRunPanelProps {
  tool: ToolMeta
}

function ToolRunPanel({ tool }: ToolRunPanelProps) {
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

  // Reset state when the selected tool changes
  useEffect(() => {
    setScope('')
    setJsonInput('{}')
    setResult(null)
    setSetupResult(null)
    setSetupDone(false)
    setReconfigureRunKey(null)
    setFixRunKey(null)
  }, [tool.name])

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
      const res = await api.runTool(tool.name, input)
      setResult(res)
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
            <div className="flex items-center gap-2">
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
            </div>

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

  const load = useCallback(() => {
    api.listTools()
      .then(data => {
        setTools(data)
        if (data.length > 0 && !selectedName) {
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
          <ToolRunPanel tool={selectedTool} />
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
