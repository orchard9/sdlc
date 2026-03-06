import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { cn } from '@/lib/utils'
import { X, Copy, Play, Loader2, Check } from 'lucide-react'
import type { OrchestratorWebhookRoute, WebhookPayloadItem } from '@/lib/types'

// ---------------------------------------------------------------------------
// Time range helpers
// ---------------------------------------------------------------------------

type TimeRange = '1h' | '6h' | '24h' | '7d'

const TIME_RANGES: { label: string; value: TimeRange; hours: number }[] = [
  { label: '1h', value: '1h', hours: 1 },
  { label: '6h', value: '6h', hours: 6 },
  { label: '24h', value: '24h', hours: 24 },
  { label: '7d', value: '7d', hours: 168 },
]

function sinceForRange(range: TimeRange): string {
  const hours = TIME_RANGES.find(r => r.value === range)?.hours ?? 24
  return new Date(Date.now() - hours * 3600_000).toISOString()
}

function formatTimestamp(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function estimateSize(body: unknown): string {
  const json = typeof body === 'string' ? body : JSON.stringify(body)
  const bytes = new Blob([json]).size
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface Props {
  route: OrchestratorWebhookRoute
  onClose: () => void
}

export function WebhookPayloadInspector({ route, onClose }: Props) {
  const [timeRange, setTimeRange] = useState<TimeRange>('24h')
  const [payloads, setPayloads] = useState<WebhookPayloadItem[]>([])
  const [selected, setSelected] = useState<WebhookPayloadItem | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [replayResult, setReplayResult] = useState<{ ok: boolean; run_id?: string; error?: string } | null>(null)
  const [isReplaying, setIsReplaying] = useState(false)
  const [copied, setCopied] = useState(false)

  const fetchPayloads = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      // Strip leading slash for the URL parameter — the backend adds it
      const routeParam = route.path.startsWith('/') ? route.path.slice(1) : route.path
      const data = await api.queryWebhookPayloads(routeParam, {
        since: sinceForRange(timeRange),
        limit: 100,
      })
      setPayloads(data)
      setSelected(null)
      setReplayResult(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load payloads')
    } finally {
      setLoading(false)
    }
  }, [route.path, timeRange])

  useEffect(() => {
    fetchPayloads()
  }, [fetchPayloads])

  async function handleReplay() {
    if (!selected) return
    setIsReplaying(true)
    setReplayResult(null)
    try {
      const routeParam = route.path.startsWith('/') ? route.path.slice(1) : route.path
      const result = await api.replayWebhookPayload(routeParam, selected.id)
      setReplayResult(result)
    } catch (e) {
      setReplayResult({ ok: false, error: e instanceof Error ? e.message : 'Replay failed' })
    } finally {
      setIsReplaying(false)
    }
  }

  function handleCopy() {
    if (!selected) return
    const text = typeof selected.body === 'string'
      ? selected.body
      : JSON.stringify(selected.body, null, 2)
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  const bodyText = selected
    ? typeof selected.body === 'string'
      ? selected.body
      : JSON.stringify(selected.body, null, 2)
    : ''

  return (
    <div className="mt-3 border border-border rounded-lg overflow-hidden">
      {/* Time range bar */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-muted/30 border-b border-border">
        <div className="flex items-center gap-1.5">
          <span className="text-xs text-muted-foreground mr-1">Time:</span>
          {TIME_RANGES.map(r => (
            <button
              key={r.value}
              onClick={() => setTimeRange(r.value)}
              className={cn(
                'px-2.5 py-1 text-xs rounded transition-colors',
                timeRange === r.value
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-accent text-muted-foreground'
              )}
            >
              {r.label}
            </button>
          ))}
          <span className="ml-3 text-xs text-muted-foreground font-mono">{route.path}</span>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          title="Close inspector"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Error state */}
      {error && (
        <div className="px-4 py-3 text-xs text-red-400 bg-red-500/5">
          {error}
        </div>
      )}

      {/* Loading state */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Empty state */}
      {!loading && !error && payloads.length === 0 && (
        <div className="px-4 py-12 text-center text-sm text-muted-foreground">
          No payloads found in this time window.
        </div>
      )}

      {/* Two-pane layout */}
      {!loading && payloads.length > 0 && (
        <div className="flex" style={{ minHeight: 300, maxHeight: 480 }}>
          {/* Left pane: payload list */}
          <div className="w-2/5 border-r border-border overflow-y-auto">
            {payloads.map(p => (
              <button
                key={p.id}
                onClick={() => { setSelected(p); setReplayResult(null) }}
                className={cn(
                  'w-full text-left px-4 py-2.5 border-b border-border transition-colors',
                  selected?.id === p.id ? 'bg-accent/30' : 'hover:bg-accent/10'
                )}
              >
                <div className="text-xs text-foreground">{formatTimestamp(p.received_at)}</div>
                <div className="flex items-center gap-2 mt-0.5">
                  <span className="text-[10px] text-muted-foreground font-mono">
                    {p.content_type ?? 'unknown'}
                  </span>
                  <span className="text-[10px] text-muted-foreground">
                    {estimateSize(p.body)}
                  </span>
                </div>
              </button>
            ))}
          </div>

          {/* Right pane: payload detail */}
          <div className="w-3/5 flex flex-col">
            {!selected ? (
              <div className="flex-1 flex items-center justify-center text-sm text-muted-foreground">
                Select a payload to inspect
              </div>
            ) : (
              <>
                {/* Metadata header */}
                <div className="px-4 py-2 border-b border-border bg-muted/20 flex items-center gap-4 text-[10px] text-muted-foreground font-mono">
                  <span>ID: {selected.id.slice(0, 8)}...</span>
                  <span>{formatTimestamp(selected.received_at)}</span>
                  <span>{selected.content_type ?? 'unknown'}</span>
                </div>

                {/* JSON body */}
                <div className="flex-1 overflow-auto px-4 py-3">
                  <pre className="text-xs text-foreground whitespace-pre-wrap break-words font-mono leading-relaxed">
                    {bodyText}
                  </pre>
                </div>

                {/* Actions bar */}
                <div className="px-4 py-2 border-t border-border flex items-center gap-2">
                  <button
                    onClick={handleCopy}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded bg-muted hover:bg-accent text-foreground transition-colors"
                    title="Copy payload"
                  >
                    {copied ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
                    {copied ? 'Copied' : 'Copy'}
                  </button>
                  <button
                    onClick={handleReplay}
                    disabled={isReplaying}
                    className={cn(
                      'flex items-center gap-1.5 px-3 py-1.5 text-xs rounded transition-colors',
                      isReplaying
                        ? 'bg-muted text-muted-foreground cursor-not-allowed'
                        : 'bg-blue-500/10 text-blue-400 hover:bg-blue-500/20'
                    )}
                    title="Replay payload through tool"
                  >
                    {isReplaying ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
                    Replay
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {/* Replay result strip */}
      {replayResult && (
        <div className={cn(
          'px-4 py-2 text-xs border-t border-border',
          replayResult.ok ? 'bg-green-500/5 text-green-400' : 'bg-red-500/5 text-red-400'
        )}>
          {replayResult.ok
            ? `Replay successful — run_id: ${replayResult.run_id}`
            : `Replay failed: ${replayResult.error}`}
        </div>
      )}
    </div>
  )
}
