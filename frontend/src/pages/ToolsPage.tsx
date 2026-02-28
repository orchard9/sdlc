import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { ToolCard } from '@/components/tools/ToolCard'
import { AmaResultPanel } from '@/components/tools/AmaResultPanel'
import { QualityCheckPanel } from '@/components/tools/QualityCheckPanel'
import { Loader2, AlertTriangle, Play, Wrench } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ToolMeta, ToolResult, AmaData, QualityCheckData } from '@/lib/types'

// ---------------------------------------------------------------------------
// Tool run panel (right side)
// ---------------------------------------------------------------------------

interface ToolRunPanelProps {
  tool: ToolMeta
}

function ToolRunPanel({ tool }: ToolRunPanelProps) {
  const [question, setQuestion] = useState('')
  const [scope, setScope] = useState('')
  const [jsonInput, setJsonInput] = useState('{}')
  const [running, setRunning] = useState(false)
  const [settingUp, setSettingUp] = useState(false)
  const [result, setResult] = useState<ToolResult | null>(null)
  const [setupResult, setSetupResult] = useState<ToolResult | null>(null)
  const [setupDone, setSetupDone] = useState(false)

  // Reset state when the selected tool changes
  useEffect(() => {
    setQuestion('')
    setScope('')
    setJsonInput('{}')
    setResult(null)
    setSetupResult(null)
    setSetupDone(false)
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
      if (tool.name === 'ama') {
        input = { question: question.trim() }
      } else if (tool.name === 'quality-check') {
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

  const canRun = tool.name === 'ama'
    ? question.trim().length > 0
    : true

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
        {/* Setup banner */}
        {tool.requires_setup && !setupDone && (
          <div className="flex items-start gap-3 px-4 py-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
            <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-amber-300">Setup required</p>
              {tool.setup_description && (
                <p className="text-xs text-amber-400/70 mt-0.5">{tool.setup_description}</p>
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
            <button
              onClick={handleSetup}
              disabled={settingUp}
              className="shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 rounded-lg border border-amber-500/30 transition-colors disabled:opacity-50"
            >
              {settingUp && <Loader2 className="w-3 h-3 animate-spin" />}
              {settingUp ? 'Running…' : 'Run Setup'}
            </button>
          </div>
        )}

        {/* Input area */}
        <div className="space-y-2">
          {tool.name === 'ama' && (
            <>
              <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wider">Question</label>
              <input
                type="text"
                value={question}
                onChange={e => setQuestion(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter' && canRun && !running) handleRun() }}
                placeholder="e.g. Where is JWT validation?"
                className="w-full px-3 py-2 text-sm bg-muted/40 border border-border rounded-lg outline-none focus:border-primary/50 transition-colors placeholder:text-muted-foreground/50"
              />
            </>
          )}
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
          {tool.name !== 'ama' && tool.name !== 'quality-check' && (
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

        {/* Run button */}
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

        {/* Results */}
        {result && (
          <div className="mt-2">
            {!result.ok ? (
              <div className="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-sm text-red-300">{result.error}</p>
              </div>
            ) : tool.name === 'ama' ? (
              <AmaResultPanel data={result.data as AmaData} />
            ) : tool.name === 'quality-check' ? (
              <QualityCheckPanel data={result.data as QualityCheckData} />
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
  )
}
