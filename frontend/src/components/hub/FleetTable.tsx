import { Activity, ChevronDown, ChevronUp, ExternalLink, Trash2, TriangleAlert } from 'lucide-react'
import { Fragment, useState } from 'react'
import type { FleetInstance } from '@/lib/types'
import { cn } from '@/lib/utils'
import { api } from '@/api/client'

type FleetStatusTone = 'online' | 'degraded' | 'provisioning' | 'failed'

function classify(instance: FleetInstance): FleetStatusTone {
  if (instance.deployment_status === 'failed' || instance.provision_status === 'failed') return 'failed'
  if (instance.deployment_status === 'pending' || instance.provision_status === 'requested' || instance.provision_status === 'provisioning') return 'provisioning'
  if (instance.attention_reasons.length > 0) return 'degraded'
  return 'online'
}

function toneClasses(tone: FleetStatusTone) {
  switch (tone) {
    case 'online':
      return 'bg-emerald-400'
    case 'degraded':
      return 'bg-amber-400'
    case 'provisioning':
      return 'bg-sky-400'
    case 'failed':
      return 'bg-rose-400'
  }
}

function statusLabel(instance: FleetInstance) {
  const tone = classify(instance)
  if (tone === 'provisioning') return 'Provisioning'
  if (tone === 'failed') return 'Failed'
  if (tone === 'degraded') return 'Degraded'
  return 'Healthy'
}

function formatRelative(date: string | null) {
  if (!date) return 'No heartbeat'
  const delta = Math.max(0, Date.now() - new Date(date).getTime())
  const secs = Math.round(delta / 1000)
  if (secs < 60) return `${secs}s ago`
  const mins = Math.round(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.round(mins / 60)
  return `${hours}h ago`
}

export function FleetTable({
  instances,
  selectedSlug,
  onSelect,
}: {
  instances: FleetInstance[]
  selectedSlug: string | null
  onSelect: (slug: string | null) => void
}) {
  return (
    <section className="overflow-hidden rounded-xl border border-border bg-card">
      <div className="px-5 py-4 border-b border-border/80">
        <h2 className="text-base font-semibold tracking-tight">Fleet</h2>
        <p className="text-sm text-muted-foreground mt-1">Healthy, degraded, and provisioning projects in one scan-friendly table.</p>
      </div>

      {instances.length === 0 ? (
        <div className="px-5 py-10 text-sm text-muted-foreground">No fleet rows match the current filters.</div>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead className="bg-muted/20">
              <tr className="text-left text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
                <th className="px-5 py-3 font-medium">Status</th>
                <th className="px-5 py-3 font-medium">Project</th>
                <th className="px-5 py-3 font-medium">Milestone</th>
                <th className="px-5 py-3 font-medium">Fleet</th>
                <th className="px-5 py-3 font-medium">Agents</th>
                <th className="px-5 py-3 font-medium">Heartbeat</th>
                <th className="px-5 py-3 font-medium">Open</th>
              </tr>
            </thead>
            <tbody>
              {instances.map(instance => {
                const expanded = selectedSlug === instance.slug
                const tone = classify(instance)
                return (
                  <Fragment key={instance.slug}>
                    <tr
                      className="cursor-pointer border-t border-border/70 transition-colors hover:bg-muted/15"
                      onClick={() => onSelect(expanded ? null : instance.slug)}
                    >
                      <td className="px-5 py-4">
                        <div className="inline-flex items-center gap-2 rounded-md border border-border bg-background/70 px-2.5 py-1 text-xs font-medium">
                          <span className={cn('inline-block w-2.5 h-2.5 rounded-full', toneClasses(tone))} />
                          {statusLabel(instance)}
                        </div>
                      </td>
                      <td className="px-5 py-4">
                        <div className="flex items-center gap-2">
                          <span className="font-medium tracking-tight">{instance.slug}</span>
                          {instance.attention_reasons.length > 0 && <TriangleAlert className="w-4 h-4 text-amber-400" />}
                          {expanded ? <ChevronUp className="w-4 h-4 text-muted-foreground" /> : <ChevronDown className="w-4 h-4 text-muted-foreground" />}
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">{instance.namespace}</div>
                      </td>
                      <td className="px-5 py-4 text-sm text-muted-foreground">
                        {instance.active_milestone || 'No active milestone'}
                      </td>
                      <td className="px-5 py-4 text-sm text-muted-foreground">
                        <div>{instance.feature_count ?? '—'} features</div>
                        <div className="mt-1 capitalize">{instance.deployment_status}</div>
                      </td>
                      <td className="px-5 py-4 text-sm text-muted-foreground">
                        {instance.agent_running ? (
                          <span className="inline-flex items-center gap-1.5 text-emerald-300">
                            <Activity className="w-3.5 h-3.5" />
                            Running
                          </span>
                        ) : 'Idle'}
                      </td>
                      <td className="px-5 py-4 text-sm text-muted-foreground">
                        <div>{formatRelative(instance.last_heartbeat_at)}</div>
                        <div className="mt-1 capitalize">{instance.heartbeat_status || 'unknown'}</div>
                      </td>
                      <td className="px-5 py-4">
                        <button
                          onClick={event => {
                            event.stopPropagation()
                            window.open(instance.url, '_blank')
                          }}
                          className="inline-flex items-center gap-2 rounded-lg border border-border bg-background/70 px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
                        >
                          Open
                          <ExternalLink className="w-3.5 h-3.5" />
                        </button>
                      </td>
                    </tr>
                    {expanded && (
                      <tr className="border-t border-border/40 bg-muted/10">
                        <td colSpan={7} className="px-5 py-4">
                          <div className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
                            <div className="space-y-3">
                              <div className="text-sm font-medium tracking-tight">Project detail</div>
                              <div className="space-y-3 rounded-lg border border-border bg-card/70 p-4">
                                <div className="text-sm text-muted-foreground break-all">{instance.url}</div>
                                <div className="flex flex-wrap gap-2">
                                  {instance.attention_reasons.length > 0 ? (
                                    instance.attention_reasons.map(reason => (
                                      <span key={reason} className="rounded-md border border-amber-400/20 bg-amber-400/10 px-2.5 py-1 text-xs text-amber-100">
                                        {reason}
                                      </span>
                                    ))
                                  ) : (
                                    <span className="rounded-md border border-emerald-500/20 bg-emerald-500/10 px-2.5 py-1 text-xs text-emerald-100">
                                      No active issues
                                    </span>
                                  )}
                                </div>
                              </div>
                            </div>
                            <div className="space-y-4">
                              <div className="rounded-lg border border-border bg-card/70 p-4">
                                <div className="mb-3 text-sm font-medium tracking-tight">At a glance</div>
                                <div className="space-y-2 text-sm text-muted-foreground">
                                  <div className="flex items-center justify-between gap-4">
                                    <span>Provision</span>
                                    <span className="capitalize">{instance.provision_status || 'none'}</span>
                                  </div>
                                  <div className="flex items-center justify-between gap-4">
                                    <span>Heartbeat</span>
                                    <span className="capitalize">{instance.heartbeat_status || 'unknown'}</span>
                                  </div>
                                  <div className="flex items-center justify-between gap-4">
                                    <span>Created</span>
                                    <span>{instance.created_at ? new Date(instance.created_at).toLocaleString() : '—'}</span>
                                  </div>
                                </div>
                              </div>
                              <DeleteProjectButton slug={instance.slug} onDeleted={() => onSelect(null)} />
                            </div>
                          </div>
                        </td>
                      </tr>
                    )}
                  </Fragment>
                )
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}

function DeleteProjectButton({ slug, onDeleted }: { slug: string; onDeleted: () => void }) {
  const [deleting, setDeleting] = useState(false)

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation()
    if (!window.confirm(`Delete ${slug}? This removes the k8s namespace and all its resources.`)) return
    setDeleting(true)
    try {
      await api.deleteProject(slug)
      onDeleted()
    } catch {
      setDeleting(false)
    }
  }

  return (
    <button
      onClick={handleDelete}
      disabled={deleting}
      className="inline-flex items-center gap-2 rounded-lg border border-rose-500/30 bg-rose-500/10 px-3 py-1.5 text-xs font-medium text-rose-300 hover:bg-rose-500/20 hover:text-rose-200 transition-colors disabled:opacity-50"
    >
      <Trash2 className="w-3.5 h-3.5" />
      {deleting ? 'Deleting…' : 'Delete project'}
    </button>
  )
}
