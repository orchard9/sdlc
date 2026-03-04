import { useCallback, useEffect, useState } from 'react'
import { ChevronRight, Layers } from 'lucide-react'
import { api } from '@/api/client'
import type { HubProjectEntry, HubProjectStatus } from '@/lib/types'
import { useHubSSE } from '@/hooks/useHubSSE'
import { cn } from '@/lib/utils'

// ---------------------------------------------------------------------------
// Status dot
// ---------------------------------------------------------------------------

function StatusDot({ status }: { status: HubProjectStatus }) {
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
// Project card
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
// Empty state
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-4 text-center">
      <Layers className="w-12 h-12 text-muted-foreground opacity-30" />
      <div className="text-lg font-semibold text-muted-foreground">No projects registered</div>
      <div className="text-sm text-muted-foreground bg-muted/40 border border-border rounded-lg px-5 py-3 font-mono">
        Configure projects to send heartbeats. See ~/.sdlc/hub.yaml
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// HubPage
// ---------------------------------------------------------------------------

export function HubPage() {
  const [projects, setProjects] = useState<HubProjectEntry[]>([])
  const [filter, setFilter] = useState('')
  const [loading, setLoading] = useState(true)

  // Initial load
  useEffect(() => {
    api.getHubProjects()
      .then(setProjects)
      .catch(() => setProjects([]))
      .finally(() => setLoading(false))
  }, [])

  // SSE live updates
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

  const onRecompute = useCallback(
    (updater: (projects: HubProjectEntry[]) => HubProjectEntry[]) => {
      setProjects(updater)
    },
    [],
  )

  useHubSSE({ onProjectUpdated, onProjectRemoved }, onRecompute)

  // Filter
  const lowerFilter = filter.toLowerCase()
  const visible = lowerFilter
    ? projects.filter(
        p =>
          p.name.toLowerCase().includes(lowerFilter) ||
          p.url.toLowerCase().includes(lowerFilter),
      )
    : projects

  const countLabel =
    lowerFilter
      ? `${visible.length} of ${projects.length} project${projects.length !== 1 ? 's' : ''}`
      : `${projects.length} project${projects.length !== 1 ? 's' : ''}`

  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="max-w-6xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center gap-3 mb-6">
          <h1 className="text-2xl font-bold">Projects</h1>
          <span className="bg-muted border border-border rounded-full px-3 py-0.5 text-xs text-muted-foreground">
            {projects.length}
          </span>
        </div>

        {/* Filter */}
        <div className="mb-2">
          <input
            type="text"
            value={filter}
            onChange={e => setFilter(e.target.value)}
            placeholder="Filter projects..."
            className="w-full max-w-sm bg-muted/50 border border-border rounded-lg px-3 py-2 text-sm outline-none focus:border-primary transition-colors"
          />
        </div>

        {/* Count line */}
        {!loading && (
          <p className="text-xs text-muted-foreground mb-5">{countLabel}</p>
        )}

        {/* Content */}
        {loading ? (
          <div className="flex items-center justify-center py-20">
            <div className="w-6 h-6 border-2 border-border border-t-primary rounded-full animate-spin" />
          </div>
        ) : projects.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {visible.map(project => (
              <ProjectCard key={project.url} project={project} />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
