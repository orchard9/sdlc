import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '@/api/client'
import type { TunnelStatus, AppTunnelStatus } from '@/lib/types'
import { Wifi, WifiOff, Copy, Check, Loader2, AlertCircle, ExternalLink, MessageSquare } from 'lucide-react'
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
  onStart,
  onStop,
}: {
  active: boolean
  toggling: boolean
  onStart: () => void
  onStop: () => void
}) {
  return (
    <button
      onClick={active ? onStop : onStart}
      disabled={toggling}
      className={cn(
        'flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
        active
          ? 'bg-destructive/10 text-destructive hover:bg-destructive/20'
          : 'bg-primary text-primary-foreground hover:bg-primary/90',
        toggling && 'opacity-50 cursor-not-allowed'
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

function TunnelDisclosure() {
  return (
    <div className="border-t border-border/50 pt-3 space-y-1.5">
      <p className="text-xs text-muted-foreground/70">
        <span className="font-medium text-muted-foreground">Requires cloudflared.</span>{' '}
        Install: <code className="font-mono">brew install cloudflare/cloudflare/cloudflared</code>
      </p>
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

function SdlcTunnelSection() {
  const [status, setStatus] = useState<TunnelStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [toggling, setToggling] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [sessionToken, setSessionToken] = useState<string | null>(null)

  const load = useCallback(async () => {
    try {
      const s = await api.getTunnel()
      setStatus(s)
      if (!s.active) setSessionToken(null)
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
      setError(e instanceof Error ? e.message : 'Failed to start SDLC tunnel')
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
      setError(e instanceof Error ? e.message : 'Failed to stop SDLC tunnel')
    } finally {
      setToggling(false)
    }
  }

  const authUrl = status?.url && sessionToken ? `${status.url}/?auth=${sessionToken}` : null

  return (
    <section className="bg-card border border-border rounded-xl p-5 space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold">SDLC Tunnel</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Expose the SDLC UI publicly — share with collaborators via QR code or auth URL
          </p>
          {!loading && status && (
            <p className="text-xs text-muted-foreground/70 mt-1">
              Proxies <code className="font-mono">localhost:{status.port}</code>
            </p>
          )}
        </div>
        {!loading && status && (
          <TunnelToggleButton
            active={status.active}
            toggling={toggling}
            onStart={handleStart}
            onStop={handleStop}
          />
        )}
        {loading && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground mt-0.5" />}
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
            <UrlRow label="Auth URL" value={authUrl} />
          ) : (
            <p className="text-xs text-muted-foreground/70 italic">
              Auth token not shown after page reload — stop and restart to get a new auth URL.
            </p>
          )}
        </div>
      )}

      <TunnelDisclosure />
    </section>
  )
}

// ---------------------------------------------------------------------------
// App Tunnel section
// ---------------------------------------------------------------------------

function AppTunnelSection() {
  const navigate = useNavigate()
  const [status, setStatus] = useState<AppTunnelStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [toggling, setToggling] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [portInput, setPortInput] = useState('')
  const [portSaved, setPortSaved] = useState(false)
  const [feedbackCount, setFeedbackCount] = useState(0)

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
      setError('Enter a valid port number (1–65535)')
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
            Shares your app publicly and injects a feedback widget into every page — reviewers can leave notes that appear directly in your SDLC feedback inbox
          </p>
        </div>
        {!loading && status && (
          <TunnelToggleButton
            active={status.active}
            toggling={toggling}
            onStart={handleStart}
            onStop={handleStop}
          />
        )}
        {loading && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground mt-0.5" />}
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
            <span className="text-muted-foreground/40">→ sdlc-server → reviewers</span>
          </div>
          <UrlRow label="Tunnel URL" value={status.url} />
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

      <TunnelDisclosure />
    </section>
  )
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export function NetworkPage() {
  return (
    <div className="p-6 max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-xl font-semibold">Network</h1>
        <p className="text-sm text-muted-foreground mt-1">Tunnels and connectivity</p>
      </div>

      <SdlcTunnelSection />
      <AppTunnelSection />
    </div>
  )
}
