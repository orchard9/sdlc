import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { QRCodeSVG } from 'qrcode.react'
import { api } from '@/api/client'
import type { TunnelStatus, AppTunnelStatus, TunnelPreflightResult } from '@/lib/types'
import { Wifi, WifiOff, Copy, Check, Loader2, AlertCircle, ExternalLink, MessageSquare, AlertTriangle } from 'lucide-react'
import { cn } from '@/lib/utils'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  const copy = () => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }
  return (
    <button
      onClick={copy}
      className="p-1 rounded hover:bg-accent transition-colors text-muted-foreground hover:text-foreground"
      aria-label="Copy to clipboard"
    >
      {copied ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
    </button>
  )
}

function UrlRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center gap-2">
      <span className="w-20 text-xs text-muted-foreground shrink-0">{label}</span>
      <code className="flex-1 text-xs font-mono bg-muted/50 px-2 py-1 rounded truncate">{value}</code>
      <CopyButton text={value} />
    </div>
  )
}

function TunnelToggleButton({
  active,
  toggling,
  disabled,
  onStart,
  onStop,
}: {
  active: boolean
  toggling: boolean
  disabled?: boolean
  onStart: () => void
  onStop: () => void
}) {
  return (
    <button
      onClick={active ? onStop : onStart}
      disabled={toggling || disabled}
      className={cn(
        'flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
        active
          ? 'bg-destructive/10 text-destructive hover:bg-destructive/20'
          : 'bg-primary text-primary-foreground hover:bg-primary/90',
        (toggling || disabled) && 'opacity-50 cursor-not-allowed'
      )}
    >
      {toggling ? (
        <Loader2 className="w-3.5 h-3.5 animate-spin" />
      ) : active ? (
        <WifiOff className="w-3.5 h-3.5" />
      ) : (
        <Wifi className="w-3.5 h-3.5" />
      )}
      {active ? 'Stop tunnel' : 'Start tunnel'}
    </button>
  )
}

function QrDisplay({ url, label }: { url: string; label?: string }) {
  return (
    <div className="flex flex-col items-center gap-2 pt-1">
      <div className="bg-white p-3 rounded-xl inline-block">
        <QRCodeSVG value={url} size={140} />
      </div>
      {label && <p className="text-xs text-muted-foreground/70 text-center">{label}</p>}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Preflight-aware disclosure / warning
// ---------------------------------------------------------------------------

function PreflightWarning({ preflight }: { preflight: TunnelPreflightResult }) {
  return (
    <div className="bg-amber-500/8 border border-amber-500/25 rounded-lg px-4 py-3 space-y-2">
      <div className="flex items-center gap-2">
        <AlertTriangle className="w-4 h-4 text-amber-500 shrink-0" />
        <span className="text-sm font-medium text-amber-500">orch-tunnel not found</span>
      </div>
      <p className="text-xs text-muted-foreground">
        Install to enable tunnels:
      </p>
      <div className="flex items-center gap-2">
        <code className="text-xs font-mono bg-muted/50 px-2 py-1 rounded">brew install orch-tunnel</code>
        <CopyButton text="brew install orch-tunnel" />
      </div>
      <p className="text-xs text-muted-foreground/70">
        or <code className="font-mono text-xs">gh release download --repo orchard9/tunnel</code>
      </p>
      {preflight.checked.length > 0 && (
        <details className="text-xs text-muted-foreground/60">
          <summary className="cursor-pointer hover:text-muted-foreground transition-colors">
            Checked {preflight.checked.length} locations
          </summary>
          <ul className="mt-1.5 space-y-0.5 pl-1">
            {preflight.checked.map((loc, i) => (
              <li key={i} className="flex items-center gap-1.5">
                <span className={loc.found ? 'text-green-500' : 'text-red-400'}>
                  {loc.found ? 'Y' : 'N'}
                </span>
                <span className="font-mono">{loc.location}</span>
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  )
}

function TunnelDisclosure({ preflight }: { preflight: TunnelPreflightResult | null }) {
  // If preflight says not installed, show warning instead of generic disclosure
  if (preflight && !preflight.installed) {
    return <PreflightWarning preflight={preflight} />
  }

  return (
    <div className="border-t border-border/50 pt-3 space-y-1.5">
      {preflight?.installed && (
        <p className="text-xs text-muted-foreground/70">
          <code className="font-mono text-xs bg-muted/50 px-1.5 py-0.5 rounded">
            {preflight.version ?? 'orch-tunnel'}
          </code>
          {preflight.path && (
            <span className="ml-1.5">
              at <code className="font-mono text-xs">{preflight.path}</code>
            </span>
          )}
          {preflight.process_path_stale && (
            <span className="ml-1.5 text-amber-500">(found via fallback — not on process PATH)</span>
          )}
        </p>
      )}
      <p className="text-xs text-muted-foreground/70">
        <span className="font-medium text-muted-foreground">Proxy only.</span>{' '}
        Tunnels proxy HTTP requests — no static hosting beyond what the server exposes.
      </p>
    </div>
  )
}

// ---------------------------------------------------------------------------
// SDLC Tunnel section
// ---------------------------------------------------------------------------

function SdlcTunnelSection({ preflight, preflightLoading }: { preflight: TunnelPreflightResult | null; preflightLoading: boolean }) {
  const [status, setStatus] = useState<TunnelStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [toggling, setToggling] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [sessionToken, setSessionToken] = useState<string | null>(null)

  const tunnelUnavailable = preflightLoading || (preflight != null && !preflight.installed)

  const load = useCallback(async () => {
    try {
      const s = await api.getTunnel()
      setStatus(s)
      if (!s.active) {
        setSessionToken(null)
      } else if (s.token) {
        setSessionToken(s.token)
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load tunnel status')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  const handleStart = async () => {
    setToggling(true)
    setError(null)
    try {
      const s = await api.startTunnel()
      setStatus(s)
      if (s.token) setSessionToken(s.token)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start Ponder tunnel')
    } finally {
      setToggling(false)
    }
  }

  const handleStop = async () => {
    setToggling(true)
    setError(null)
    try {
      const s = await api.stopTunnel()
      setStatus(s)
      setSessionToken(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to stop Ponder tunnel')
    } finally {
      setToggling(false)
    }
  }

  const authUrl = status?.url && sessionToken ? `${status.url}/?auth=${sessionToken}` : null

  return (
    <section className="bg-card border border-border rounded-xl p-5 space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold">Ponder Tunnel</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Expose the Ponder UI publicly — share with collaborators via QR code or auth URL
          </p>
          {!loading && status && (
            <p className="text-xs text-muted-foreground/70 mt-1">
              Proxies <code className="font-mono">localhost:{status.port}</code>
            </p>
          )}
        </div>
        {preflightLoading && (
          <button disabled className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium opacity-50 cursor-not-allowed bg-primary text-primary-foreground whitespace-nowrap">
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
            Checking...
          </button>
        )}
        {!preflightLoading && !loading && status && (
          <TunnelToggleButton
            active={status.active}
            toggling={toggling}
            disabled={tunnelUnavailable && !status.active}
            onStart={handleStart}
            onStop={handleStop}
          />
        )}
        {!preflightLoading && loading && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground mt-0.5" />}
      </div>

      {error && (
        <div className="flex items-start gap-2 text-sm text-destructive bg-destructive/10 rounded-lg px-3 py-2">
          <AlertCircle className="w-4 h-4 mt-0.5 shrink-0" />
          {error}
        </div>
      )}

      {status?.active && status.url && (
        <div className="space-y-2 pt-1">
          <UrlRow label="Tunnel URL" value={status.url} />
          {authUrl ? (
            <>
              <UrlRow label="Auth URL" value={authUrl} />
              <QrDisplay url={authUrl} label="Scan to open — token embedded" />
            </>
          ) : (
            <p className="text-xs text-muted-foreground/70 italic">
              Auth token not shown after page reload — stop and restart to get a new auth URL.
            </p>
          )}
        </div>
      )}

      <TunnelDisclosure preflight={preflight} />
    </section>
  )
}

// ---------------------------------------------------------------------------
// App Tunnel section
// ---------------------------------------------------------------------------

function AppTunnelSection({ preflight, preflightLoading }: { preflight: TunnelPreflightResult | null; preflightLoading: boolean }) {
  const navigate = useNavigate()
  const [status, setStatus] = useState<AppTunnelStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [toggling, setToggling] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [portInput, setPortInput] = useState('')
  const [portSaved, setPortSaved] = useState(false)
  const [feedbackCount, setFeedbackCount] = useState(0)

  const tunnelUnavailable = preflightLoading || (preflight != null && !preflight.installed)

  const load = useCallback(async () => {
    try {
      const [s, notes] = await Promise.all([api.getAppTunnel(), api.getFeedback()])
      setStatus(s)
      setFeedbackCount(notes.length)
      if (s.configured_port) setPortInput(String(s.configured_port))
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load app tunnel status')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  const savePort = useCallback(async (value: string) => {
    const port = parseInt(value, 10)
    if (!value || isNaN(port) || port < 1 || port > 65535) return
    try {
      await api.setAppPort(port)
      setPortSaved(true)
      setTimeout(() => setPortSaved(false), 1500)
    } catch {
      // non-fatal — port still usable for starting
    }
  }, [])

  const handleStart = async () => {
    const port = parseInt(portInput, 10)
    if (!portInput || isNaN(port) || port < 1 || port > 65535) {
      setError('Enter a valid port number (1-65535)')
      return
    }
    setToggling(true)
    setError(null)
    try {
      const s = await api.startAppTunnel(port)
      setStatus(s)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start app tunnel')
    } finally {
      setToggling(false)
    }
  }

  const handleStop = async () => {
    setToggling(true)
    setError(null)
    try {
      const s = await api.stopAppTunnel()
      setStatus(s)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to stop app tunnel')
    } finally {
      setToggling(false)
    }
  }

  return (
    <section className="bg-card border border-border rounded-xl p-5 space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold">App Tunnel</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Shares your app publicly and injects a feedback widget into every page — reviewers can leave notes that appear directly in your Ponder feedback inbox
          </p>
        </div>
        {preflightLoading && (
          <button disabled className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium opacity-50 cursor-not-allowed bg-primary text-primary-foreground whitespace-nowrap">
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
            Checking...
          </button>
        )}
        {!preflightLoading && !loading && status && (
          <TunnelToggleButton
            active={status.active}
            toggling={toggling}
            disabled={tunnelUnavailable && !status.active}
            onStart={handleStart}
            onStop={handleStop}
          />
        )}
        {!preflightLoading && loading && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground mt-0.5" />}
      </div>

      {/* Port input — shown when tunnel is not active */}
      {!loading && status && !status.active && (
        <div className="flex items-center gap-3">
          <label className="text-xs text-muted-foreground shrink-0">App port</label>
          <input
            type="number"
            min={1}
            max={65535}
            placeholder="e.g. 3000"
            value={portInput}
            onChange={e => setPortInput(e.target.value)}
            onBlur={e => savePort(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter') {
                savePort(portInput)
                handleStart()
              }
            }}
            className="w-32 px-2.5 py-1.5 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring font-mono"
          />
          {portSaved && (
            <span className="flex items-center gap-1 text-xs text-green-500">
              <Check className="w-3 h-3" />
              saved
            </span>
          )}
          {!portSaved && (
            <a
              href={portInput ? `http://localhost:${portInput}` : undefined}
              target="_blank"
              rel="noreferrer"
              className={cn(
                'flex items-center gap-1 text-xs transition-colors',
                portInput
                  ? 'text-muted-foreground hover:text-foreground'
                  : 'text-muted-foreground/30 pointer-events-none'
              )}
            >
              open local
              <ExternalLink className="w-3 h-3" />
            </a>
          )}
        </div>
      )}

      {error && (
        <div className="flex items-start gap-2 text-sm text-destructive bg-destructive/10 rounded-lg px-3 py-2">
          <AlertCircle className="w-4 h-4 mt-0.5 shrink-0" />
          {error}
        </div>
      )}

      {status?.active && status.url && (
        <div className="space-y-2 pt-1">
          <div className="flex items-center gap-2 text-xs text-muted-foreground/70">
            <span>Proxying</span>
            <code className="font-mono">localhost:{status.configured_port}</code>
            <span className="text-muted-foreground/40">-&gt; sdlc-server -&gt; reviewers</span>
          </div>
          <UrlRow label="Tunnel URL" value={status.url} />
          <QrDisplay url={status.url} label="Share with reviewers" />
          {feedbackCount > 0 && (
            <button
              onClick={() => navigate('/feedback')}
              className="flex items-center gap-1.5 text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
            >
              <MessageSquare className="w-3.5 h-3.5" />
              {feedbackCount} feedback note{feedbackCount !== 1 ? 's' : ''} pending
            </button>
          )}
        </div>
      )}

      <TunnelDisclosure preflight={preflight} />
    </section>
  )
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export function NetworkPage() {
  const [preflight, setPreflight] = useState<TunnelPreflightResult | null>(null)
  const [preflightLoading, setPreflightLoading] = useState(true)

  useEffect(() => {
    api.getTunnelPreflight()
      .then(setPreflight)
      .catch(() => {
        // Preflight failed — degrade gracefully: allow buttons (old behavior)
        setPreflight({ installed: true, path: null, version: null, source: null, process_path_stale: false, checked: [], install_hint: null })
      })
      .finally(() => setPreflightLoading(false))
  }, [])

  return (
    <div className="max-w-3xl mx-auto p-4 sm:p-6 space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Network</h2>
        <p className="text-sm text-muted-foreground mt-1">Tunnels and connectivity</p>
      </div>

      <SdlcTunnelSection preflight={preflight} preflightLoading={preflightLoading} />
      <AppTunnelSection preflight={preflight} preflightLoading={preflightLoading} />
    </div>
  )
}
