import { useCallback, useEffect, useState } from 'react'
import {
  ChevronRight,
  Copy,
  Download,
  Layers,
  Play,
  Plus,
  Search,
  Loader2,
  AlertCircle,
  CheckCircle2,
  Cpu,
} from 'lucide-react'
import { api } from '@/api/client'
import type {
  HubProjectEntry,
  FleetInstance,
  AvailableRepo,
  FleetAgentSummary,
  CreateRepoResponse,
} from '@/lib/types'
import { useHubSSE } from '@/hooks/useHubSSE'
import { cn } from '@/lib/utils'

// ---------------------------------------------------------------------------
// Status dot for fleet instances
// ---------------------------------------------------------------------------

function InstanceStatusDot({
  deploymentStatus,
  podHealthy,
}: {
  deploymentStatus: FleetInstance['deployment_status']
  podHealthy: boolean
}) {
  const label = podHealthy ? deploymentStatus : deploymentStatus === 'running' ? 'degraded' : deploymentStatus
  return (
    <span
      className={cn(
        'inline-block w-2.5 h-2.5 rounded-full flex-shrink-0',
        deploymentStatus === 'running' && podHealthy && 'bg-green-500',
        deploymentStatus === 'running' && !podHealthy && 'bg-yellow-400',
        deploymentStatus === 'pending' && 'bg-yellow-400',
        deploymentStatus === 'failed' && 'bg-red-500',
        deploymentStatus === 'unknown' && 'bg-zinc-500',
      )}
      title={label}
    />
  )
}

// Legacy status dot for heartbeat-based projects (fallback)
function StatusDot({ status }: { status: 'online' | 'stale' | 'offline' }) {
  return (
    <span
      className={cn(
        'inline-block w-2.5 h-2.5 rounded-full flex-shrink-0',
        status === 'online' && 'bg-green-500',
        status === 'stale' && 'bg-yellow-400',
        status === 'offline' && 'bg-zinc-500',
      )}
      title={status}
    />
  )
}

// ---------------------------------------------------------------------------
// Agent badge
// ---------------------------------------------------------------------------

function AgentBadge() {
  return (
    <span className="flex items-center gap-1.5 text-green-400 text-xs">
      <span className="inline-block w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
      agent running
    </span>
  )
}

// ---------------------------------------------------------------------------
// Agent summary bar
// ---------------------------------------------------------------------------

function AgentSummaryBar({ summary }: { summary: FleetAgentSummary | null }) {
  if (!summary) return null

  return (
    <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
      <Cpu className="w-4 h-4" />
      {summary.total_active_runs > 0 ? (
        <span>
          {summary.total_active_runs} agent{summary.total_active_runs !== 1 ? 's' : ''} running
          across {summary.projects_with_agents} project{summary.projects_with_agents !== 1 ? 's' : ''}
        </span>
      ) : (
        <span>No active agents</span>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Fleet instance card
// ---------------------------------------------------------------------------

function FleetInstanceCard({ instance }: { instance: FleetInstance }) {
  const handleClick = () => {
    window.open(instance.url, '_blank')
  }

  return (
    <button
      onClick={handleClick}
      className="text-left w-full bg-card border border-border rounded-xl p-4 hover:border-zinc-600 transition-colors cursor-pointer flex flex-col gap-2"
    >
      <div className="flex items-center gap-2.5">
        <InstanceStatusDot deploymentStatus={instance.deployment_status} podHealthy={instance.pod_healthy} />
        <span className="font-semibold text-sm flex-1 truncate">{instance.slug}</span>
        <ChevronRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
      </div>
      <div className="text-xs text-muted-foreground truncate pl-5">{instance.url}</div>
      <div className="flex items-center gap-2 flex-wrap pl-5">
        {instance.active_milestone && (
          <span className="bg-muted/60 border border-border rounded px-1.5 py-0.5 text-xs text-muted-foreground">
            {instance.active_milestone}
          </span>
        )}
        {instance.feature_count !== null && instance.feature_count !== undefined && (
          <span className="text-xs text-muted-foreground">
            {instance.feature_count} {instance.feature_count === 1 ? 'feature' : 'features'}
          </span>
        )}
        {instance.agent_running && (
          <AgentBadge />
        )}
      </div>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Legacy project card (heartbeat fallback)
// ---------------------------------------------------------------------------

function ProjectCard({ project }: { project: HubProjectEntry }) {
  const handleClick = () => {
    window.open(project.url, '_blank')
  }

  return (
    <button
      onClick={handleClick}
      className="text-left w-full bg-card border border-border rounded-xl p-4 hover:border-zinc-600 transition-colors cursor-pointer flex flex-col gap-2"
    >
      <div className="flex items-center gap-2.5">
        <StatusDot status={project.status} />
        <span className="font-semibold text-sm flex-1 truncate">{project.name}</span>
        <ChevronRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
      </div>
      <div className="text-xs text-muted-foreground truncate pl-5">{project.url}</div>
      <div className="flex items-center gap-2 flex-wrap pl-5">
        {project.active_milestone && (
          <span className="bg-muted/60 border border-border rounded px-1.5 py-0.5 text-xs text-muted-foreground">
            {project.active_milestone}
          </span>
        )}
        {project.feature_count !== null && project.feature_count !== undefined && (
          <span className="text-xs text-muted-foreground">
            {project.feature_count} {project.feature_count === 1 ? 'feature' : 'features'}
          </span>
        )}
        {project.agent_running === true && <AgentBadge />}
      </div>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Available repo card
// ---------------------------------------------------------------------------

function AvailableRepoCard({
  repo,
  isProvisioning,
  error,
  onStart,
}: {
  repo: AvailableRepo
  isProvisioning: boolean
  error: string | null
  onStart: () => void
}) {
  return (
    <div className="bg-card border border-border rounded-xl p-4 flex flex-col gap-2">
      <div className="flex items-center gap-2.5">
        <span className="font-semibold text-sm flex-1 truncate">{repo.slug}</span>
      </div>
      {repo.description && (
        <div className="text-xs text-muted-foreground line-clamp-2">{repo.description}</div>
      )}
      {error && (
        <div className="flex items-center gap-1.5 text-xs text-red-400">
          <AlertCircle className="w-3.5 h-3.5 flex-shrink-0" />
          <span className="truncate">{error}</span>
        </div>
      )}
      <div className="mt-auto pt-1">
        <button
          onClick={onStart}
          disabled={isProvisioning}
          className={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
            isProvisioning
              ? 'bg-muted text-muted-foreground cursor-not-allowed'
              : 'bg-primary text-primary-foreground hover:bg-primary/90 cursor-pointer',
          )}
        >
          {isProvisioning ? (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Provisioning...
            </>
          ) : (
            <>
              <Play className="w-3.5 h-3.5" />
              Start
            </>
          )}
        </button>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Import section
// ---------------------------------------------------------------------------

type ImportState = 'idle' | 'importing' | 'provisioning' | 'done' | 'error'

function ImportSection() {
  const [url, setUrl] = useState('')
  const [pat, setPat] = useState('')
  const [state, setState] = useState<ImportState>('idle')
  const [error, setError] = useState<string | null>(null)

  const handleImport = async () => {
    if (!url.trim()) return
    setState('importing')
    setError(null)
    try {
      await api.importRepo(url.trim(), pat.trim() || undefined)
      setState('provisioning')
      // SSE will signal when provisioning is done, but show done state after a delay
      setTimeout(() => {
        setState('done')
        setUrl('')
        setPat('')
        setTimeout(() => setState('idle'), 3000)
      }, 5000)
    } catch (err) {
      setState('error')
      setError(err instanceof Error ? err.message : 'Import failed')
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-col sm:flex-row gap-3">
        <input
          type="text"
          value={url}
          onChange={e => setUrl(e.target.value)}
          placeholder="https://github.com/org/repo"
          disabled={state === 'importing' || state === 'provisioning'}
          className="flex-1 bg-muted/50 border border-border rounded-lg px-3 py-2 text-sm outline-none focus:border-primary transition-colors disabled:opacity-50"
        />
        <input
          type="password"
          value={pat}
          onChange={e => setPat(e.target.value)}
          placeholder="PAT (optional, for private repos)"
          disabled={state === 'importing' || state === 'provisioning'}
          className="sm:w-64 bg-muted/50 border border-border rounded-lg px-3 py-2 text-sm outline-none focus:border-primary transition-colors disabled:opacity-50"
        />
        <button
          onClick={handleImport}
          disabled={!url.trim() || state === 'importing' || state === 'provisioning'}
          className={cn(
            'flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
            state === 'importing' || state === 'provisioning'
              ? 'bg-muted text-muted-foreground cursor-not-allowed'
              : state === 'done'
                ? 'bg-green-600 text-white'
                : 'bg-primary text-primary-foreground hover:bg-primary/90 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed',
          )}
        >
          {state === 'importing' && (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Importing...
            </>
          )}
          {state === 'provisioning' && (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Provisioning...
            </>
          )}
          {state === 'done' && (
            <>
              <CheckCircle2 className="w-4 h-4" />
              Done
            </>
          )}
          {(state === 'idle' || state === 'error') && (
            <>
              <Download className="w-4 h-4" />
              Import
            </>
          )}
        </button>
      </div>
      {state === 'error' && error && (
        <div className="flex items-center gap-2 text-sm text-red-400">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Create repo section
// ---------------------------------------------------------------------------

type CreateRepoState = 'idle' | 'creating' | 'done' | 'error'

const NAME_RE = /^[a-z0-9][a-z0-9-]*$/

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(text).catch(() => {})
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <button
      onClick={handleCopy}
      className={cn(
        'flex-shrink-0 flex items-center gap-1 px-3 py-1.5 rounded-lg border text-xs font-medium transition-colors cursor-pointer',
        copied
          ? 'border-green-500 text-green-400'
          : 'border-border bg-muted/50 text-muted-foreground hover:text-foreground',
      )}
    >
      {copied ? (
        <>
          <CheckCircle2 className="w-3 h-3" />
          Copied!
        </>
      ) : (
        <>
          <Copy className="w-3 h-3" />
          Copy
        </>
      )}
    </button>
  )
}

function CreateRepoSection() {
  const [name, setName] = useState('')
  const [state, setState] = useState<CreateRepoState>('idle')
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<CreateRepoResponse | null>(null)

  const nameError =
    name.length > 0 && !NAME_RE.test(name)
      ? 'Lowercase letters, numbers, and hyphens only'
      : name.length > 100
        ? 'Name must be 100 characters or fewer'
        : null

  const handleCreate = async () => {
    if (!name.trim() || nameError) return
    setState('creating')
    setError(null)
    try {
      const data = await api.createRepo(name.trim())
      setResult(data)
      setState('done')
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to create repo'
      setError(
        msg.toLowerCase().includes('repo_exists') || msg.includes('already exists')
          ? `A repo named "${name}" already exists. Choose a different name.`
          : msg,
      )
      setState('error')
    }
  }

  const handleReset = () => {
    setName('')
    setState('idle')
    setError(null)
    setResult(null)
  }

  if (state === 'done' && result) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-2 text-sm font-semibold text-green-400">
          <CheckCircle2 className="w-4 h-4" />
          {result.repo_slug} created
        </div>

        <div className="space-y-3">
          <div>
            <p className="text-xs text-muted-foreground mb-1.5">Add remote:</p>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-muted/50 border border-border rounded-lg px-3 py-2 text-xs overflow-hidden text-ellipsis whitespace-nowrap">
                git remote add gitea {result.push_url}
              </code>
              <CopyButton text={`git remote add gitea ${result.push_url}`} />
            </div>
          </div>

          <div>
            <p className="text-xs text-muted-foreground mb-1.5">Push:</p>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-muted/50 border border-border rounded-lg px-3 py-2 text-xs">
                git push gitea main
              </code>
              <CopyButton text="git push gitea main" />
            </div>
          </div>

          <p className="text-xs text-muted-foreground">
            This is your deployment remote — push here to update your cluster instance.
          </p>
        </div>

        <button
          onClick={handleReset}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border bg-muted/40 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          Add another project
        </button>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1">
          <input
            type="text"
            value={name}
            onChange={e => { setName(e.target.value); if (state === 'error') setState('idle') }}
            onKeyDown={e => e.key === 'Enter' && handleCreate()}
            placeholder="project-name"
            disabled={state === 'creating'}
            className={cn(
              'w-full bg-muted/50 border rounded-lg px-3 py-2 text-sm outline-none transition-colors disabled:opacity-50',
              nameError ? 'border-red-500 focus:border-red-400' : 'border-border focus:border-primary',
            )}
          />
          {nameError && (
            <p className="text-xs text-red-400 mt-1">{nameError}</p>
          )}
        </div>
        <button
          onClick={handleCreate}
          disabled={!name.trim() || !!nameError || state === 'creating'}
          className={cn(
            'flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
            state === 'creating'
              ? 'bg-muted text-muted-foreground cursor-not-allowed'
              : 'bg-primary text-primary-foreground hover:bg-primary/90 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed',
          )}
        >
          {state === 'creating' ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Creating...
            </>
          ) : (
            <>
              <Plus className="w-4 h-4" />
              Create
            </>
          )}
        </button>
      </div>
      <p className="text-xs text-muted-foreground">lowercase letters, numbers, and hyphens</p>
      {state === 'error' && error && (
        <div className="flex items-center gap-2 text-sm text-red-400">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-6 max-w-md mx-auto w-full">
      <div className="text-center">
        <Layers className="w-12 h-12 text-muted-foreground opacity-30 mx-auto mb-3" />
        <div className="text-lg font-semibold">No projects yet</div>
        <p className="text-sm text-muted-foreground mt-1">
          Create a Gitea repo and push your local project to get started.
        </p>
      </div>
      <div className="w-full">
        <CreateRepoSection />
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Section header
// ---------------------------------------------------------------------------

function SectionHeader({ title, count }: { title: string; count: number }) {
  return (
    <div className="flex items-center gap-2 mb-3">
      <h2 className="text-lg font-semibold">{title}</h2>
      <span className="bg-muted border border-border rounded-full px-2.5 py-0.5 text-xs text-muted-foreground">
        {count}
      </span>
    </div>
  )
}

// ---------------------------------------------------------------------------
// HubPage — fleet control plane
// ---------------------------------------------------------------------------

export function HubPage() {
  // Legacy heartbeat data (fallback)
  const [projects, setProjects] = useState<HubProjectEntry[]>([])

  // Fleet data
  const [instances, setInstances] = useState<FleetInstance[]>([])
  const [available, setAvailable] = useState<AvailableRepo[]>([])
  const [agentSummary, setAgentSummary] = useState<FleetAgentSummary | null>(null)

  // UI state
  const [filter, setFilter] = useState('')
  const [loading, setLoading] = useState(true)
  const [fleetAvailable, setFleetAvailable] = useState(false)
  const [provisioningSlugs, setProvisioningSlugs] = useState<Set<string>>(new Set())
  const [provisionErrors, setProvisionErrors] = useState<Record<string, string>>({})

  // Initial load — fetch all data in parallel
  useEffect(() => {
    let cancelled = false

    async function load() {
      // Try fleet endpoints first
      const fleetPromise = api.getFleet().catch(() => null)
      const availablePromise = api.getAvailable().catch(() => null)
      const agentPromise = api.getAgentSummary().catch(() => null)
      // Always load heartbeat fallback
      const projectsPromise = api.getHubProjects().catch(() => [])

      const [fleetData, availData, agentData, projectData] = await Promise.all([
        fleetPromise,
        availablePromise,
        agentPromise,
        projectsPromise,
      ])

      if (cancelled) return

      if (fleetData) {
        setInstances(fleetData)
        setFleetAvailable(true)
      }
      if (availData) {
        setAvailable(availData)
      }
      if (agentData) {
        setAgentSummary(agentData)
      }
      setProjects(projectData)
      setLoading(false)
    }

    load()
    return () => { cancelled = true }
  }, [])

  // SSE callbacks — legacy
  const onProjectUpdated = useCallback((project: HubProjectEntry) => {
    setProjects(prev => {
      const idx = prev.findIndex(p => p.url === project.url)
      if (idx >= 0) {
        const next = [...prev]
        next[idx] = project
        return next
      }
      return [project, ...prev]
    })
  }, [])

  const onProjectRemoved = useCallback((url: string) => {
    setProjects(prev => prev.filter(p => p.url !== url))
  }, [])

  // SSE callbacks — fleet
  const onFleetUpdated = useCallback((instance: FleetInstance) => {
    setInstances(prev => {
      const idx = prev.findIndex(i => i.namespace === instance.namespace)
      if (idx >= 0) {
        const next = [...prev]
        next[idx] = instance
        return next
      }
      return [instance, ...prev]
    })
  }, [])

  const onFleetProvisioned = useCallback((instance: FleetInstance) => {
    // Add to instances
    setInstances(prev => {
      const idx = prev.findIndex(i => i.namespace === instance.namespace)
      if (idx >= 0) {
        const next = [...prev]
        next[idx] = instance
        return next
      }
      return [instance, ...prev]
    })
    // Remove from available
    setAvailable(prev => prev.filter(r => r.slug !== instance.slug))
    // Clear provisioning state
    setProvisioningSlugs(prev => {
      const next = new Set(prev)
      next.delete(instance.slug)
      return next
    })
  }, [])

  const onFleetAgentStatus = useCallback((summary: FleetAgentSummary) => {
    setAgentSummary(summary)
  }, [])

  const onRecompute = useCallback(
    (updater: (projects: HubProjectEntry[]) => HubProjectEntry[]) => {
      setProjects(updater)
    },
    [],
  )

  useHubSSE(
    { onProjectUpdated, onProjectRemoved, onFleetUpdated, onFleetProvisioned, onFleetAgentStatus },
    onRecompute,
  )

  // Provision handler
  const handleProvision = useCallback(async (slug: string) => {
    setProvisioningSlugs(prev => new Set(prev).add(slug))
    setProvisionErrors(prev => {
      const next = { ...prev }
      delete next[slug]
      return next
    })
    try {
      await api.provision(slug)
    } catch (err) {
      // Remove provisioning state and show error
      setProvisioningSlugs(prev => {
        const next = new Set(prev)
        next.delete(slug)
        return next
      })
      setProvisionErrors(prev => ({
        ...prev,
        [slug]: err instanceof Error ? err.message : 'Provisioning failed',
      }))
    }
  }, [])

  // Filter
  const lowerFilter = filter.toLowerCase()

  const visibleInstances = lowerFilter
    ? instances.filter(
        i =>
          i.slug.toLowerCase().includes(lowerFilter) ||
          i.url.toLowerCase().includes(lowerFilter),
      )
    : instances

  const visibleAvailable = lowerFilter
    ? available.filter(
        r =>
          r.slug.toLowerCase().includes(lowerFilter) ||
          (r.description || '').toLowerCase().includes(lowerFilter),
      )
    : available

  // Legacy fallback: use projects if fleet API not available
  const visibleProjects = lowerFilter
    ? projects.filter(
        p =>
          p.name.toLowerCase().includes(lowerFilter) ||
          p.url.toLowerCase().includes(lowerFilter),
      )
    : projects

  const totalCount = fleetAvailable
    ? instances.length + available.length
    : projects.length

  const visibleCount = fleetAvailable
    ? visibleInstances.length + visibleAvailable.length
    : visibleProjects.length

  const countLabel = lowerFilter
    ? `${visibleCount} of ${totalCount} project${totalCount !== 1 ? 's' : ''}`
    : `${totalCount} project${totalCount !== 1 ? 's' : ''}`

  const hasAnyContent = fleetAvailable
    ? instances.length > 0 || available.length > 0
    : projects.length > 0

  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="max-w-6xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center gap-3 mb-6">
          <h1 className="text-2xl font-bold">Projects</h1>
          <span className="bg-muted border border-border rounded-full px-3 py-0.5 text-xs text-muted-foreground">
            {totalCount}
          </span>
        </div>

        {/* Search — autofocused, search-first design */}
        <div className="mb-2 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none" />
          <input
            type="text"
            value={filter}
            onChange={e => setFilter(e.target.value)}
            placeholder="Search projects..."
            autoFocus
            className="w-full max-w-sm bg-muted/50 border border-border rounded-lg pl-9 pr-3 py-2 text-sm outline-none focus:border-primary transition-colors"
          />
        </div>

        {/* Agent summary bar */}
        {fleetAvailable && <AgentSummaryBar summary={agentSummary} />}

        {/* Count line */}
        {!loading && (
          <p className="text-xs text-muted-foreground mb-5">{countLabel}</p>
        )}

        {/* Content */}
        {loading ? (
          <div className="flex items-center justify-center py-20">
            <div className="w-6 h-6 border-2 border-border border-t-primary rounded-full animate-spin" />
          </div>
        ) : !hasAnyContent ? (
          <EmptyState />
        ) : fleetAvailable ? (
          <div className="space-y-8">
            {/* Running Instances */}
            {(visibleInstances.length > 0 || !lowerFilter) && (
              <section>
                <SectionHeader title="Running" count={visibleInstances.length} />
                {visibleInstances.length > 0 ? (
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {visibleInstances.map(instance => (
                      <FleetInstanceCard key={instance.namespace} instance={instance} />
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No instances running. Start one from available repos below.
                  </p>
                )}
              </section>
            )}

            {/* Available Repos */}
            {(visibleAvailable.length > 0 || !lowerFilter) && (
              <section>
                <SectionHeader title="Available" count={visibleAvailable.length} />
                <p className="text-xs text-muted-foreground mb-3">
                  Start deploys an sdlc workspace for this repo in the fleet.
                </p>
                {visibleAvailable.length > 0 ? (
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {visibleAvailable.map(repo => (
                      <AvailableRepoCard
                        key={repo.slug}
                        repo={repo}
                        isProvisioning={provisioningSlugs.has(repo.slug)}
                        error={provisionErrors[repo.slug] ?? null}
                        onStart={() => handleProvision(repo.slug)}
                      />
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    All repos have running instances.
                  </p>
                )}
              </section>
            )}

            {/* Import */}
            <section>
              <h2 className="text-lg font-semibold mb-3">Import External Repo</h2>
              <ImportSection />
            </section>

            {/* Add New Project */}
            <section>
              <h2 className="text-lg font-semibold mb-1">Add New Project</h2>
              <p className="text-xs text-muted-foreground mb-3">
                Create a new Gitea repo and push your local project to the cluster.
              </p>
              <CreateRepoSection />
            </section>
          </div>
        ) : (
          /* Legacy heartbeat-based view (fleet API not available) */
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {visibleProjects.map(project => (
              <ProjectCard key={project.url} project={project} />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
