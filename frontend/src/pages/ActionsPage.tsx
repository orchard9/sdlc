import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '@/api/client'
import { useSseContext } from '@/contexts/SseContext'
import { cn } from '@/lib/utils'
import { parseRecurrence, formatRecurrence } from '@/lib/recurrence'
import { Loader2, AlertTriangle, Pencil, Trash2, Plus, Zap, Webhook, History } from 'lucide-react'
import type { OrchestratorAction, OrchestratorWebhookEvent, OrchestratorWebhookRoute, ToolMeta } from '@/lib/types'

// ---------------------------------------------------------------------------
// Relative time helper
// ---------------------------------------------------------------------------

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return `${secs}s ago`
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h ago`
  return `${Math.floor(hrs / 24)}d ago`
}

function futureRelativeTime(iso: string): string {
  const diff = new Date(iso).getTime() - Date.now()
  if (diff <= 0) return 'overdue'
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return `in ${secs}s`
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `in ${mins}m`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `in ${hrs}h`
  return `in ${Math.floor(hrs / 24)}d`
}

// ---------------------------------------------------------------------------
// Status badge
// ---------------------------------------------------------------------------

function ActionStatusBadge({ status }: { status: OrchestratorAction['status'] }) {
  const base = 'inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium border'
  if (status.type === 'pending') {
    return <span className={cn(base, 'bg-muted text-muted-foreground border-border')}>Pending</span>
  }
  if (status.type === 'running') {
    return <span className={cn(base, 'bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse')}>Running</span>
  }
  if (status.type === 'completed') {
    return <span className={cn(base, 'bg-green-500/10 text-green-400 border-green-500/20')}>Completed</span>
  }
  return <span className={cn(base, 'bg-red-500/10 text-red-400 border-red-500/20')}>Failed</span>
}

// ---------------------------------------------------------------------------
// Outcome badge
// ---------------------------------------------------------------------------

function OutcomeBadge({ outcome }: { outcome: OrchestratorWebhookEvent['outcome'] }) {
  const base = 'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border'
  if (outcome.type === 'dispatched') {
    return <span className={cn(base, 'bg-green-500/10 text-green-400 border-green-500/20')}>Dispatched</span>
  }
  if (outcome.type === 'no_route_matched') {
    return <span className={cn(base, 'bg-muted text-muted-foreground border-border')}>No match</span>
  }
  return <span className={cn(base, 'bg-red-500/10 text-red-400 border-red-500/20')}>Rejected</span>
}

// ---------------------------------------------------------------------------
// Schedule Action modal
// ---------------------------------------------------------------------------

interface ScheduleActionModalProps {
  tools: ToolMeta[]
  onClose: () => void
  onCreated: () => void
}

function ScheduleActionModal({ tools, onClose, onCreated }: ScheduleActionModalProps) {
  const [label, setLabel] = useState('')
  const [toolName, setToolName] = useState(tools[0]?.name ?? '')
  const [toolInput, setToolInput] = useState('{}')
  const [scheduledAt, setScheduledAt] = useState(() => {
    const d = new Date(Date.now() + 60_000)
    return d.toISOString().slice(0, 16)
  })
  const [recurrence, setRecurrence] = useState('')
  const [recurrenceError, setRecurrenceError] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState('')

  function validateRecurrence(val: string): boolean {
    if (val === '') return true
    return parseRecurrence(val) !== null
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (recurrence && !validateRecurrence(recurrence)) {
      setRecurrenceError('Use format: 10s, 30m, 1h, 2d')
      return
    }
    setSubmitting(true)
    setError('')
    try {
      let toolInputParsed: unknown = {}
      try { toolInputParsed = JSON.parse(toolInput) } catch { /* use empty */ }
      const recurrenceSecs = recurrence ? parseRecurrence(recurrence) : null
      await api.createAction({
        label,
        tool_name: toolName,
        tool_input: toolInputParsed,
        scheduled_at: new Date(scheduledAt).toISOString(),
        recurrence_secs: recurrenceSecs,
      })
      onCreated()
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create action')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-md shadow-xl">
        <h2 className="text-base font-semibold mb-4">Schedule Action</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Label</label>
            <input
              type="text"
              required
              value={label}
              onChange={e => setLabel(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
              placeholder="nightly-audit"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Tool</label>
            <select
              required
              value={toolName}
              onChange={e => setToolName(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {tools.map(t => (
                <option key={t.name} value={t.name}>{t.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Tool Input (JSON)</label>
            <textarea
              rows={3}
              value={toolInput}
              onChange={e => setToolInput(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg font-mono focus:outline-none focus:ring-1 focus:ring-ring"
              placeholder="{}"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Scheduled At</label>
            <input
              type="datetime-local"
              required
              value={scheduledAt}
              onChange={e => setScheduledAt(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Recurrence (optional)</label>
            <input
              type="text"
              value={recurrence}
              onChange={e => {
                setRecurrence(e.target.value)
                setRecurrenceError('')
              }}
              className={cn(
                'w-full px-3 py-2 text-sm bg-background border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring',
                recurrenceError ? 'border-red-500' : 'border-border'
              )}
              placeholder="e.g. 10s, 30m, 1h, 2d"
            />
            {recurrenceError && (
              <p className="mt-1 text-xs text-red-400">{recurrenceError}</p>
            )}
          </div>
          {error && <p className="text-xs text-red-400">{error}</p>}
          <div className="flex gap-2 justify-end pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm rounded-lg border border-border hover:bg-accent/50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="px-4 py-2 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              Create
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Edit Action modal
// ---------------------------------------------------------------------------

interface EditActionModalProps {
  action: OrchestratorAction
  onClose: () => void
  onUpdated: (updated: OrchestratorAction) => void
}

function EditActionModal({ action, onClose, onUpdated }: EditActionModalProps) {
  const [label, setLabel] = useState(action.label)
  const [recurrence, setRecurrence] = useState(
    action.recurrence_secs != null ? formatRecurrence(action.recurrence_secs) : ''
  )
  const [recurrenceError, setRecurrenceError] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (recurrence && parseRecurrence(recurrence) === null) {
      setRecurrenceError('Use format: 10s, 30m, 1h, 2d')
      return
    }
    setSubmitting(true)
    setError('')
    try {
      const recurrenceSecs = recurrence ? parseRecurrence(recurrence) : null
      const updated = await api.updateAction(action.id, { label, recurrence_secs: recurrenceSecs })
      onUpdated(updated)
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update action')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-sm shadow-xl">
        <h2 className="text-base font-semibold mb-4">Edit Action</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Label</label>
            <input
              type="text"
              required
              value={label}
              onChange={e => setLabel(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Recurrence</label>
            <input
              type="text"
              value={recurrence}
              onChange={e => {
                setRecurrence(e.target.value)
                setRecurrenceError('')
              }}
              className={cn(
                'w-full px-3 py-2 text-sm bg-background border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring',
                recurrenceError ? 'border-red-500' : 'border-border'
              )}
              placeholder="e.g. 1h — leave blank to clear"
            />
            {recurrenceError && (
              <p className="mt-1 text-xs text-red-400">{recurrenceError}</p>
            )}
          </div>
          {error && <p className="text-xs text-red-400">{error}</p>}
          <div className="flex gap-2 justify-end pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm rounded-lg border border-border hover:bg-accent/50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="px-4 py-2 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              Save
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Add Webhook Route modal
// ---------------------------------------------------------------------------

interface AddRouteModalProps {
  tools: ToolMeta[]
  onClose: () => void
  onCreated: () => void
}

function AddRouteModal({ tools, onClose, onCreated }: AddRouteModalProps) {
  const [path, setPath] = useState('/')
  const [toolName, setToolName] = useState(tools[0]?.name ?? '')
  const [inputTemplate, setInputTemplate] = useState('{"payload": "{{payload}}"}')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState('')
  const [pathError, setPathError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!path.startsWith('/')) {
      setPathError('Path must start with /')
      return
    }
    setSubmitting(true)
    setError('')
    try {
      await api.createWebhookRoute({ path, tool_name: toolName, input_template: inputTemplate })
      onCreated()
      onClose()
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to create route'
      if (msg.toLowerCase().includes('duplicate') || msg.toLowerCase().includes('conflict') || msg.toLowerCase().includes('already')) {
        setError('A route with this path already exists.')
      } else {
        setError(msg)
      }
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-md shadow-xl">
        <h2 className="text-base font-semibold mb-4">Add Webhook Route</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Path</label>
            <input
              type="text"
              required
              value={path}
              onChange={e => {
                setPath(e.target.value)
                setPathError('')
              }}
              className={cn(
                'w-full px-3 py-2 text-sm bg-background border rounded-lg font-mono focus:outline-none focus:ring-1 focus:ring-ring',
                pathError ? 'border-red-500' : 'border-border'
              )}
              placeholder="/hooks/github"
            />
            {pathError && <p className="mt-1 text-xs text-red-400">{pathError}</p>}
            {error && error.includes('already exists') && (
              <p className="mt-1 text-xs text-red-400">{error}</p>
            )}
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Tool</label>
            <select
              required
              value={toolName}
              onChange={e => setToolName(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {tools.map(t => (
                <option key={t.name} value={t.name}>{t.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-muted-foreground mb-1">Input Template</label>
            <textarea
              rows={3}
              required
              value={inputTemplate}
              onChange={e => setInputTemplate(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg font-mono focus:outline-none focus:ring-1 focus:ring-ring"
              placeholder='{"payload": "{{payload}}"}'
            />
          </div>
          {error && !error.includes('already exists') && (
            <p className="text-xs text-red-400">{error}</p>
          )}
          <div className="flex gap-2 justify-end pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm rounded-lg border border-border hover:bg-accent/50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="px-4 py-2 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              {submitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              Add Route
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Scheduled Actions section
// ---------------------------------------------------------------------------

interface ScheduledActionsSectionProps {
  actions: OrchestratorAction[]
  tools: ToolMeta[]
  onRefresh: () => void
  onActionUpdated: (updated: OrchestratorAction) => void
  onActionDeleted: (id: string) => void
}

function ScheduledActionsSection({
  actions,
  tools,
  onRefresh,
  onActionUpdated,
  onActionDeleted,
}: ScheduledActionsSectionProps) {
  const [showCreate, setShowCreate] = useState(false)
  const [editingAction, setEditingAction] = useState<OrchestratorAction | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)

  async function handleDelete(id: string) {
    setDeletingId(id)
    onActionDeleted(id) // optimistic
    try {
      await api.deleteAction(id)
    } catch {
      onRefresh() // revert on failure
    } finally {
      setDeletingId(null)
    }
  }

  return (
    <section>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-muted-foreground" />
          <h2 className="text-sm font-semibold">Scheduled Actions</h2>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
          Schedule Action
        </button>
      </div>

      {actions.length === 0 ? (
        <div className="border border-dashed border-border rounded-lg py-8 text-center">
          <p className="text-sm text-muted-foreground">No actions scheduled.</p>
          <p className="text-xs text-muted-foreground/60 mt-1 font-mono">sdlc orchestrate add</p>
        </div>
      ) : (
        <div className="border border-border rounded-lg overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-muted/30">
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Label</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Tool</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Status</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Next Run</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Recurrence</th>
                <th className="px-4 py-2.5 text-right text-xs font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {actions.map(action => (
                <tr
                  key={action.id}
                  className={cn('hover:bg-accent/20 transition-colors', deletingId === action.id && 'opacity-40')}
                >
                  <td className="px-4 py-3 font-medium text-foreground">{action.label}</td>
                  <td className="px-4 py-3 text-muted-foreground font-mono text-xs">{action.tool_name}</td>
                  <td className="px-4 py-3"><ActionStatusBadge status={action.status} /></td>
                  <td className="px-4 py-3 text-xs text-muted-foreground">
                    {action.trigger.type === 'scheduled'
                      ? futureRelativeTime(action.trigger.next_tick_at)
                      : 'webhook-triggered'}
                  </td>
                  <td className="px-4 py-3 text-xs text-muted-foreground">
                    {action.recurrence_secs != null ? `every ${formatRecurrence(action.recurrence_secs)}` : '—'}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1.5 justify-end">
                      <button
                        onClick={() => setEditingAction(action)}
                        className="p-1 rounded hover:bg-accent/50 text-muted-foreground hover:text-foreground transition-colors"
                        title="Edit action"
                      >
                        <Pencil className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => handleDelete(action.id)}
                        className="p-1 rounded hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors"
                        title="Delete action"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showCreate && tools.length > 0 && (
        <ScheduleActionModal
          tools={tools}
          onClose={() => setShowCreate(false)}
          onCreated={onRefresh}
        />
      )}
      {editingAction && (
        <EditActionModal
          action={editingAction}
          onClose={() => setEditingAction(null)}
          onUpdated={updated => {
            onActionUpdated(updated)
            setEditingAction(null)
          }}
        />
      )}
    </section>
  )
}

// ---------------------------------------------------------------------------
// Webhook Routes section
// ---------------------------------------------------------------------------

interface WebhookRoutesSectionProps {
  routes: OrchestratorWebhookRoute[]
  tools: ToolMeta[]
  onRefresh: () => void
  onRouteDeleted: (id: string) => void
}

function WebhookRoutesSection({ routes, tools, onRefresh, onRouteDeleted }: WebhookRoutesSectionProps) {
  const [showCreate, setShowCreate] = useState(false)
  const [deletingId, setDeletingId] = useState<string | null>(null)

  async function handleDelete(id: string) {
    setDeletingId(id)
    onRouteDeleted(id) // optimistic
    try {
      await api.deleteWebhookRoute(id)
    } catch {
      onRefresh() // revert on failure
    } finally {
      setDeletingId(null)
    }
  }

  return (
    <section>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Webhook className="w-4 h-4 text-muted-foreground" />
          <h2 className="text-sm font-semibold">Webhook Routes</h2>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
          Add Route
        </button>
      </div>

      {routes.length === 0 ? (
        <div className="border border-dashed border-border rounded-lg py-8 text-center">
          <p className="text-sm text-muted-foreground">No webhook routes configured.</p>
        </div>
      ) : (
        <div className="border border-border rounded-lg overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-muted/30">
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Path</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Tool</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Input Template</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Created</th>
                <th className="px-4 py-2.5" />
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {routes.map(route => (
                <tr
                  key={route.id}
                  className={cn('hover:bg-accent/20 transition-colors', deletingId === route.id && 'opacity-40')}
                >
                  <td className="px-4 py-3 font-mono text-xs text-foreground">{route.path}</td>
                  <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{route.tool_name}</td>
                  <td className="px-4 py-3 text-xs text-muted-foreground font-mono truncate max-w-xs">
                    {route.input_template.length > 60
                      ? route.input_template.slice(0, 60) + '…'
                      : route.input_template}
                  </td>
                  <td className="px-4 py-3 text-xs text-muted-foreground">
                    {relativeTime(route.created_at)}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <button
                      onClick={() => handleDelete(route.id)}
                      className="p-1 rounded hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors"
                      title="Delete route"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showCreate && (
        <AddRouteModal
          tools={tools}
          onClose={() => setShowCreate(false)}
          onCreated={onRefresh}
        />
      )}
    </section>
  )
}

// ---------------------------------------------------------------------------
// Webhook Events section
// ---------------------------------------------------------------------------

interface WebhookEventsSectionProps {
  events: OrchestratorWebhookEvent[]
  actions: OrchestratorAction[]
  limit: number
  onLoadMore: () => void
}

function WebhookEventsSection({ events, actions, limit, onLoadMore }: WebhookEventsSectionProps) {
  function findActionLabel(actionId: string | null): string | null {
    if (!actionId) return null
    return actions.find(a => a.id === actionId)?.label ?? null
  }

  return (
    <section>
      <div className="flex items-center gap-2 mb-3">
        <History className="w-4 h-4 text-muted-foreground" />
        <h2 className="text-sm font-semibold">Recent Webhook Events</h2>
      </div>

      {events.length === 0 ? (
        <div className="border border-dashed border-border rounded-lg py-8 text-center">
          <p className="text-sm text-muted-foreground">No webhook events recorded.</p>
          <p className="text-xs text-muted-foreground/60 mt-1">Events appear here after the first webhook request arrives.</p>
        </div>
      ) : (
        <>
          <div className="border border-border rounded-lg overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Time</th>
                  <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Path</th>
                  <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Outcome</th>
                  <th className="px-4 py-2.5 text-left text-xs font-medium text-muted-foreground">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {events.map(event => {
                  const actionLabel = findActionLabel(event.action_id)
                  return (
                    <tr key={event.id} className="hover:bg-accent/20 transition-colors">
                      <td
                        className="px-4 py-3 text-xs text-muted-foreground"
                        title={new Date(event.received_at).toLocaleString()}
                      >
                        {relativeTime(event.received_at)}
                      </td>
                      <td className="px-4 py-3 text-xs font-mono text-foreground">{event.path}</td>
                      <td className="px-4 py-3"><OutcomeBadge outcome={event.outcome} /></td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">
                        {actionLabel ?? '—'}
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
          {events.length >= limit && (
            <button
              onClick={onLoadMore}
              className="mt-2 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              Load more
            </button>
          )}
        </>
      )}
    </section>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function ActionsPage() {
  const [actions, setActions] = useState<OrchestratorAction[]>([])
  const [routes, setRoutes] = useState<OrchestratorWebhookRoute[]>([])
  const [events, setEvents] = useState<OrchestratorWebhookEvent[]>([])
  const [tools, setTools] = useState<ToolMeta[]>([])
  const [loading, setLoading] = useState(true)
  const [dbUnavailable, setDbUnavailable] = useState(false)
  const eventsLimit = useRef(20)
  const [eventsLimitState, setEventsLimitState] = useState(20)

  const fetchAll = useCallback(async () => {
    const [actionsResult, routesResult, eventsResult, toolsResult] = await Promise.allSettled([
      api.listActions(),
      api.listWebhookRoutes(),
      api.listWebhookEvents(eventsLimit.current),
      api.listTools(),
    ])

    let anyUnavailable = false
    if (actionsResult.status === 'fulfilled') setActions(actionsResult.value)
    else if (actionsResult.reason?.message?.includes('503') || String(actionsResult.reason).includes('unavailable')) anyUnavailable = true

    if (routesResult.status === 'fulfilled') setRoutes(routesResult.value)
    if (eventsResult.status === 'fulfilled') setEvents(eventsResult.value)
    if (toolsResult.status === 'fulfilled') setTools(toolsResult.value)

    setDbUnavailable(anyUnavailable)
    setLoading(false)
  }, [])

  const refetchActions = useCallback(async () => {
    try {
      const result = await api.listActions()
      setActions(result)
    } catch { /* silently ignore */ }
  }, [])

  const refetchRoutes = useCallback(async () => {
    try {
      const [r, e] = await Promise.all([api.listWebhookRoutes(), api.listWebhookEvents(eventsLimit.current)])
      setRoutes(r)
      setEvents(e)
    } catch { /* silently ignore */ }
  }, [])

  // Initial load
  useEffect(() => { fetchAll() }, [fetchAll])

  // SSE subscription — refetch actions on ActionStateChanged
  const { subscribe } = useSseContext()
  useEffect(() => {
    return subscribe({
      onActionEvent: () => { refetchActions() },
    })
  }, [subscribe, refetchActions])

  // 5-second polling fallback
  useEffect(() => {
    const interval = setInterval(refetchActions, 5000)
    return () => clearInterval(interval)
  }, [refetchActions])

  // Optimistic handlers
  function handleActionUpdated(updated: OrchestratorAction) {
    setActions(prev => prev.map(a => a.id === updated.id ? updated : a))
  }

  function handleActionDeleted(id: string) {
    setActions(prev => prev.filter(a => a.id !== id))
  }

  function handleRouteDeleted(id: string) {
    setRoutes(prev => prev.filter(r => r.id !== id))
  }

  function handleLoadMoreEvents() {
    const newLimit = eventsLimitState + 20
    eventsLimit.current = newLimit
    setEventsLimitState(newLimit)
    api.listWebhookEvents(newLimit).then(setEvents).catch(() => {})
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="max-w-5xl mx-auto px-6 py-8 space-y-8">
      <div>
        <h1 className="text-xl font-semibold">Actions</h1>
        <p className="text-sm text-muted-foreground mt-1">Scheduled actions, webhook routes, and event history</p>
      </div>

      {dbUnavailable && (
        <div className="flex items-center gap-2.5 px-4 py-3 rounded-lg border border-yellow-500/30 bg-yellow-500/5 text-yellow-400 text-sm">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          Orchestrator DB unavailable — start the daemon to enable full functionality.
        </div>
      )}

      <ScheduledActionsSection
        actions={actions}
        tools={tools}
        onRefresh={fetchAll}
        onActionUpdated={handleActionUpdated}
        onActionDeleted={handleActionDeleted}
      />

      <WebhookRoutesSection
        routes={routes}
        tools={tools}
        onRefresh={refetchRoutes}
        onRouteDeleted={handleRouteDeleted}
      />

      <WebhookEventsSection
        events={events}
        actions={actions}
        limit={eventsLimitState}
        onLoadMore={handleLoadMoreEvents}
      />
    </div>
  )
}
