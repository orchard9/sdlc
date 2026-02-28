import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { cn } from '@/lib/utils'
import { Loader2, AlertCircle, Bot, ChevronDown, ChevronRight, Cpu, Wrench } from 'lucide-react'
import type { AgentDefinition } from '@/lib/types'

// ---------------------------------------------------------------------------
// Model badge color
// ---------------------------------------------------------------------------

function modelColor(model: string): string {
  if (model.includes('opus')) return 'bg-purple-500/10 text-purple-400 border-purple-500/20'
  if (model.includes('sonnet')) return 'bg-blue-500/10 text-blue-400 border-blue-500/20'
  if (model.includes('haiku')) return 'bg-green-500/10 text-green-400 border-green-500/20'
  return 'bg-muted text-muted-foreground border-border'
}

// ---------------------------------------------------------------------------
// Agent card
// ---------------------------------------------------------------------------

function AgentCard({ agent }: { agent: AgentDefinition }) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="border border-border rounded-lg bg-card overflow-hidden">
      {/* Header row */}
      <button
        onClick={() => setExpanded(v => !v)}
        className="w-full flex items-start gap-3 px-4 py-4 text-left hover:bg-accent/30 transition-colors"
      >
        <div className="mt-0.5 w-8 h-8 rounded-lg bg-accent flex items-center justify-center shrink-0">
          <Bot className="w-4 h-4 text-muted-foreground" />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-medium text-sm text-foreground">{agent.name}</span>
            {agent.model && (
              <span className={cn('text-xs px-1.5 py-0.5 rounded border font-mono', modelColor(agent.model))}>
                {agent.model}
              </span>
            )}
          </div>
          {agent.description && (
            <p className="mt-1 text-xs text-muted-foreground line-clamp-2 leading-relaxed">
              {agent.description}
            </p>
          )}
          {agent.tools.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {agent.tools.map(tool => (
                <span
                  key={tool}
                  className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground border border-border/50 font-mono"
                >
                  {tool}
                </span>
              ))}
            </div>
          )}
        </div>

        <div className="shrink-0 mt-1 text-muted-foreground">
          {expanded
            ? <ChevronDown className="w-4 h-4" />
            : <ChevronRight className="w-4 h-4" />}
        </div>
      </button>

      {/* Expanded: full system prompt */}
      {expanded && agent.content && (
        <div className="border-t border-border px-4 py-4">
          <div className="flex items-center gap-1.5 mb-3 text-xs text-muted-foreground">
            <Wrench className="w-3 h-3" />
            <span>System prompt</span>
          </div>
          <pre className="text-xs text-muted-foreground whitespace-pre-wrap font-mono leading-relaxed bg-muted/40 border border-border/50 rounded p-3 max-h-96 overflow-y-auto">
            {agent.content}
          </pre>
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// AgentsPage
// ---------------------------------------------------------------------------

export function AgentsPage() {
  const [agents, setAgents] = useState<AgentDefinition[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    try {
      const data = await api.getAgents()
      setAgents(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load agents')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])
  useSSE(load)

  return (
    <div className="max-w-3xl mx-auto px-6 py-8 space-y-6">
      {/* Header */}
      <div>
        <div className="flex items-center gap-2 mb-1">
          <Bot className="w-5 h-5 text-muted-foreground" />
          <h1 className="text-xl font-semibold">Agents</h1>
        </div>
        <p className="text-sm text-muted-foreground">
          Claude agents available in <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">~/.claude/agents/</code>. Use <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">/sdlc-recruit</code> to add thought partners.
        </p>
      </div>

      {/* Loading */}
      {loading && (
        <div className="flex items-center gap-2 text-muted-foreground text-sm">
          <Loader2 className="w-4 h-4 animate-spin" />
          Loading agents…
        </div>
      )}

      {/* Error */}
      {error && !loading && (
        <div className="flex items-center gap-2 text-destructive text-sm border border-destructive/20 bg-destructive/5 rounded-lg px-4 py-3">
          <AlertCircle className="w-4 h-4 shrink-0" />
          {error}
        </div>
      )}

      {/* Empty state */}
      {!loading && !error && agents.length === 0 && (
        <div className="border border-dashed border-border rounded-lg px-6 py-10 text-center">
          <Cpu className="w-8 h-8 text-muted-foreground/30 mx-auto mb-3" />
          <p className="text-sm text-muted-foreground mb-1">No agents installed</p>
          <p className="text-xs text-muted-foreground/60">
            Run <code className="font-mono bg-muted px-1.5 py-0.5 rounded">sdlc init</code> to install default agents, or{' '}
            <code className="font-mono bg-muted px-1.5 py-0.5 rounded">/sdlc-recruit</code> to add a thought partner.
          </p>
        </div>
      )}

      {/* Agent list */}
      {!loading && !error && agents.length > 0 && (
        <div className="space-y-3">
          <p className="text-xs text-muted-foreground">{agents.length} agent{agents.length !== 1 ? 's' : ''}</p>
          {agents.map(agent => (
            <AgentCard key={agent.name} agent={agent} />
          ))}
        </div>
      )}
    </div>
  )
}
