import { useDeferredValue, useEffect, useState } from 'react'
import {
  AlertTriangle,
  ArrowUpRight,
  Layers3,
  Plus,
  Search,
  Workflow,
} from 'lucide-react'
import { api } from '@/api/client'
import type {
  AvailableRepo,
  FleetInstance,
  HubActivityEntry,
  HubAttentionItem,
  HubProjectEntry,
  HubSummary,
  ProvisionEntry,
} from '@/lib/types'
import { useHubSSE } from '@/hooks/useHubSSE'
import { cn } from '@/lib/utils'
import { KpiStrip } from '@/components/hub/KpiStrip'
import { AttentionZone } from '@/components/hub/AttentionZone'
import { FleetTable } from '@/components/hub/FleetTable'
import { ActivityFeed } from '@/components/hub/ActivityFeed'
import { AddProjectModal } from '@/components/hub/AddProjectModal'

type ToggleKey = 'attention' | 'agents' | 'unhealthy' | 'available'

const FILTERS: Array<{ key: ToggleKey; label: string }> = [
  { key: 'attention', label: 'Needs Attention' },
  { key: 'agents', label: 'Running Agents' },
  { key: 'unhealthy', label: 'Unhealthy' },
  { key: 'available', label: 'Available Only' },
]

async function loadHubData() {
  const [projectData, fleetData, availableData, activityData] = await Promise.all([
    api.getHubProjects().catch(() => []),
    api.getFleet().catch(() => []),
    api.getAvailable().catch(() => []),
    api.getHubActivity().catch(() => []),
  ])

  return { projectData, fleetData, availableData, activityData }
}

function classify(instance: FleetInstance): 'online' | 'degraded' | 'provisioning' | 'failed' {
  if (instance.deployment_status === 'failed' || instance.provision_status === 'failed') return 'failed'
  if (instance.deployment_status === 'pending' || instance.provision_status === 'requested' || instance.provision_status === 'provisioning') return 'provisioning'
  if (instance.attention_reasons.length > 0) return 'degraded'
  return 'online'
}

function buildSummary(instances: FleetInstance[]): HubSummary {
  let online = 0
  let degraded = 0
  let provisioning = 0
  let failed = 0
  let activeAgents = 0

  for (const instance of instances) {
    const bucket = classify(instance)
    if (bucket === 'online') online += 1
    else if (bucket === 'degraded') degraded += 1
    else if (bucket === 'provisioning') provisioning += 1
    else failed += 1

    if (instance.agent_running) activeAgents += 1
  }

  return {
    total_projects: instances.length,
    online,
    degraded,
    provisioning,
    failed,
    active_agents: activeAgents,
    attention_count: buildAttention(instances).length,
  }
}

function buildAttention(instances: FleetInstance[]): HubAttentionItem[] {
  return instances
    .filter(instance => classify(instance) !== 'online')
    .map(instance => {
      const bucket = classify(instance)
      return {
        id: `attention:${instance.slug}`,
        severity: bucket === 'failed' ? 'error' : 'warning',
        title:
          bucket === 'failed'
            ? `${instance.slug} needs intervention`
            : bucket === 'provisioning'
              ? `${instance.slug} is provisioning`
              : `${instance.slug} is degraded`,
        detail: instance.attention_reasons.join(' • ') || 'Requires review',
        slug: instance.slug,
        url: instance.url,
      }
    })
}

function placeholderFromProvision(provision: ProvisionEntry): FleetInstance {
  return {
    slug: provision.slug,
    namespace: `sdlc-${provision.slug}`,
    url: provision.url,
    deployment_status: provision.status === 'failed' ? 'failed' : provision.status === 'ready' ? 'running' : 'pending',
    pod_healthy: provision.status === 'ready',
    active_milestone: null,
    feature_count: null,
    agent_running: null,
    created_at: provision.created_at,
    last_heartbeat_at: null,
    heartbeat_status: null,
    provision_status: provision.status,
    attention_reasons:
      provision.status === 'failed'
        ? ['Latest provision attempt failed']
        : provision.status === 'ready'
          ? []
          : ['Provisioning is still in progress'],
  }
}

function AvailableRepoCard({
  repo,
  onStart,
}: {
  repo: AvailableRepo
  onStart: (slug: string) => void
}) {
  return (
    <div className="rounded-xl border border-border bg-background/40 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="font-medium tracking-tight">{repo.slug}</div>
          <div className="mt-1 text-sm text-muted-foreground line-clamp-2">
            {repo.description || 'No description yet.'}
          </div>
        </div>
        <button
          onClick={() => onStart(repo.slug)}
          disabled={!repo.can_provision}
          className="inline-flex items-center gap-2 rounded-lg bg-primary px-3 py-2 text-xs font-medium text-primary-foreground disabled:opacity-50"
        >
          Start
          <ArrowUpRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  )
}

export function HubPage() {
  const [projects, setProjects] = useState<HubProjectEntry[]>([])
  const [instances, setInstances] = useState<FleetInstance[]>([])
  const [available, setAvailable] = useState<AvailableRepo[]>([])
  const [activity, setActivity] = useState<HubActivityEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null)
  const [addModalOpen, setAddModalOpen] = useState(false)
  const [search, setSearch] = useState('')
  const [toggles, setToggles] = useState<Record<ToggleKey, boolean>>({
    attention: false,
    agents: false,
    unhealthy: false,
    available: false,
  })

  const deferredSearch = useDeferredValue(search.trim().toLowerCase())

  useEffect(() => {
    let cancelled = false
    loadHubData()
      .then(data => {
        if (cancelled) return
        setProjects(data.projectData)
        setInstances(data.fleetData)
        setAvailable(data.availableData)
        setActivity(data.activityData)
        setLoading(false)
      })
      .catch(() => {
        if (!cancelled) setLoading(false)
      })

    return () => {
      cancelled = true
    }
  }, [])

  useHubSSE(
    {
      onProjectUpdated: project => {
        setProjects(prev => {
          const index = prev.findIndex(item => item.url === project.url)
          if (index === -1) return [project, ...prev]
          const next = [...prev]
          next[index] = project
          return next
        })
        setInstances(prev =>
          prev.map(instance =>
            instance.url === project.url || instance.slug === project.name
              ? {
                  ...instance,
                  active_milestone: project.active_milestone,
                  feature_count: project.feature_count,
                  agent_running: project.agent_running,
                  last_heartbeat_at: project.last_seen,
                  heartbeat_status: project.status,
                }
              : instance,
          ),
        )
      },
      onProjectRemoved: url => {
        setProjects(prev => prev.filter(project => project.url !== url))
        setInstances(prev =>
          prev
            .flatMap(instance => {
              if (instance.url !== url) return [instance]
              if (instance.deployment_status === 'unknown') return []
              return [{
                ...instance,
                heartbeat_status: 'offline',
                last_heartbeat_at: null,
                attention_reasons: Array.from(new Set([...instance.attention_reasons, 'Heartbeat is offline'])),
              }]
            }),
        )
      },
      onFleetUpdated: instance => {
        setInstances(prev => {
          const index = prev.findIndex(item => item.slug === instance.slug)
          if (index === -1) return [...prev, instance].sort((a, b) => a.slug.localeCompare(b.slug))
          const next = [...prev]
          next[index] = instance
          return next
        })
      },
      onFleetProvisioned: instance => {
        setInstances(prev => {
          const index = prev.findIndex(item => item.slug === instance.slug)
          if (index === -1) return [...prev, instance].sort((a, b) => a.slug.localeCompare(b.slug))
          const next = [...prev]
          next[index] = instance
          return next
        })
        setAvailable(prev => prev.filter(repo => repo.slug !== instance.slug))
      },
      onProvisionUpdated: provision => {
        setInstances(prev => {
          const index = prev.findIndex(instance => instance.slug === provision.slug)
          if (index === -1) {
            return [...prev, placeholderFromProvision(provision)].sort((a, b) => a.slug.localeCompare(b.slug))
          }
          const next = [...prev]
          next[index] = {
            ...next[index],
            provision_status: provision.status,
            deployment_status:
              provision.status === 'failed'
                ? 'failed'
                : provision.status === 'ready'
                  ? next[index].deployment_status === 'unknown' ? 'running' : next[index].deployment_status
                  : 'pending',
            attention_reasons:
              provision.status === 'failed'
                ? ['Latest provision attempt failed']
                : provision.status === 'ready'
                  ? next[index].attention_reasons.filter(reason => !reason.toLowerCase().includes('provision'))
                  : Array.from(new Set([...next[index].attention_reasons, 'Provisioning is still in progress'])),
          }
          return next
        })
      },
      onActivityAppended: entry => {
        setActivity(prev => [entry, ...prev.filter(item => item.id !== entry.id)].slice(0, 30))
      },
    },
    updater => setProjects(updater),
  )

  const summary = buildSummary(instances)
  const attentionItems = buildAttention(instances)

  const visibleInstances = instances.filter(instance => {
    if (toggles.available) return false
    if (toggles.attention && classify(instance) === 'online') return false
    if (toggles.agents && !instance.agent_running) return false
    if (toggles.unhealthy && !(instance.deployment_status === 'failed' || !instance.pod_healthy || instance.heartbeat_status === 'stale' || instance.heartbeat_status === 'offline')) return false

    if (!deferredSearch) return true

    return [
      instance.slug,
      instance.url,
      instance.namespace,
      instance.active_milestone || '',
      ...(instance.attention_reasons || []),
    ]
      .join(' ')
      .toLowerCase()
      .includes(deferredSearch)
  })

  const visibleAvailable = available.filter(repo => {
    if (!deferredSearch) return true
    return [repo.slug, repo.description || '', repo.full_name].join(' ').toLowerCase().includes(deferredSearch)
  })

  const openProject = (url: string) => {
    window.open(url, '_blank')
  }

  const handleProvision = async (slug: string) => {
    try {
      await api.provision(slug)
      setAvailable(prev => prev.filter(repo => repo.slug !== slug))
    } catch {
      // The server activity feed and attention zone will surface the failure path.
    }
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-8">
        <section className="rounded-xl border border-border bg-card">
          <div className="border-b border-border px-5 py-5 sm:px-6">
            <div className="flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
              <div className="max-w-3xl">
                <div className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Hub</div>
                <h1 className="mt-2 text-3xl font-semibold tracking-tight md:text-4xl">Fleet control</h1>
                <p className="mt-3 max-w-2xl text-sm text-muted-foreground md:text-base">
                  See what is healthy, what is drifting, what is provisioning, and what actually needs a human.
                </p>
              </div>

              <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
                <div className="flex items-center gap-3 rounded-xl border border-border bg-background px-4 py-3">
                  <Workflow className="h-5 w-5 text-primary" />
                  <div>
                    <div className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Fleet</div>
                    <div className="text-sm font-medium tracking-tight">{summary.total_projects} tracked projects</div>
                  </div>
                </div>
                <button
                  onClick={() => setAddModalOpen(true)}
                  className="inline-flex items-center justify-center gap-2 rounded-lg bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground"
                >
                  <Plus className="w-4 h-4" />
                  Add Project
                </button>
              </div>
            </div>
          </div>

          <div className="space-y-5 px-5 py-5 sm:px-6">
            <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
              <div className="relative flex-1 max-w-2xl">
                <Search className="absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <input
                  value={search}
                  onChange={event => setSearch(event.target.value)}
                  placeholder="Search fleet, repos, milestones, or issues..."
                  autoFocus
                  className="w-full rounded-lg border border-border bg-background pl-11 pr-4 py-3 text-sm outline-none transition-colors focus:border-primary"
                />
              </div>

              <div className="flex flex-wrap gap-2">
                {FILTERS.map(filter => (
                  <button
                    key={filter.key}
                    onClick={() => setToggles(prev => ({ ...prev, [filter.key]: !prev[filter.key] }))}
                    className={cn(
                      'rounded-lg border px-3 py-2 text-sm transition-colors',
                      toggles[filter.key]
                        ? 'border-border bg-accent text-foreground'
                        : 'border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground',
                    )}
                  >
                    {filter.label}
                  </button>
                ))}
              </div>
            </div>

            <KpiStrip summary={summary} />
          </div>
        </section>

        {loading ? (
          <div className="mt-6 rounded-xl border border-border bg-card px-5 py-6 text-sm text-muted-foreground">
            Loading hub...
          </div>
        ) : (
          <div className="mt-6 space-y-6">
            <div className="grid gap-6 xl:grid-cols-[1.5fr_0.9fr]">
              <AttentionZone items={attentionItems} onOpenProject={openProject} />
              <ActivityFeed items={activity} />
            </div>

            <FleetTable
              instances={visibleInstances}
              selectedSlug={selectedSlug}
              onSelect={setSelectedSlug}
            />

            <section className="rounded-xl border border-border bg-card p-5">
              <div className="mb-4 flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-base font-semibold tracking-tight">Available Repos</h2>
                  <p className="mt-1 text-sm text-muted-foreground">Launch a workspace for an existing repo, or use the Add Project action to create or import one.</p>
                </div>
                <div className="inline-flex items-center gap-2 rounded-md border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground">
                  <Layers3 className="w-3.5 h-3.5" />
                  {visibleAvailable.length}
                </div>
              </div>

              {visibleAvailable.length === 0 ? (
                <div className="rounded-lg border border-border bg-background/40 px-4 py-6 text-sm text-muted-foreground">
                  {toggles.available || deferredSearch
                    ? 'No available repos match the current filters.'
                    : 'All known repos already have a live workspace or a provision in flight.'}
                </div>
              ) : (
                <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                  {visibleAvailable.map(repo => (
                    <AvailableRepoCard key={repo.slug} repo={repo} onStart={handleProvision} />
                  ))}
                </div>
              )}
            </section>

            {projects.length === 0 && instances.length === 0 && available.length === 0 && (
              <section className="rounded-xl border border-amber-400/20 bg-amber-400/5 p-5">
                <div className="flex items-start gap-3">
                  <AlertTriangle className="mt-0.5 h-5 w-5 text-amber-300" />
                  <div>
                    <div className="font-medium tracking-tight text-amber-100">Empty fleet</div>
                    <div className="mt-1 text-sm text-amber-100/80">
                      Use <span className="font-medium">Add Project</span> to create or import a repo, or connect an existing workspace so heartbeats start flowing into the hub.
                    </div>
                  </div>
                </div>
              </section>
            )}
          </div>
        )}

        <AddProjectModal
          open={addModalOpen}
          onClose={() => setAddModalOpen(false)}
          onChanged={() => {
            loadHubData()
              .then(data => {
                setProjects(data.projectData)
                setInstances(data.fleetData)
                setAvailable(data.availableData)
                setActivity(data.activityData)
              })
              .catch(() => {})
          }}
        />
      </div>
    </div>
  )
}
