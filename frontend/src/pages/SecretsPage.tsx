import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import type { SecretsKey, SecretsEnvMeta, AuthToken, CreatedAuthToken } from '@/lib/types'
import { useSSE } from '@/hooks/useSSE'
import {
  KeyRound,
  Lock,
  Plus,
  Trash2,
  Copy,
  Check,
  Loader2,
  AlertCircle,
  X,
  Shield,
} from 'lucide-react'

// ---------------------------------------------------------------------------
// Copy button
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

// ---------------------------------------------------------------------------
// Pair type for Add Environment modal
// ---------------------------------------------------------------------------

interface Pair {
  key: string
  value: string
}

// ---------------------------------------------------------------------------
// Add Environment modal
// ---------------------------------------------------------------------------

interface AddEnvModalProps {
  onAdd: (env: string, pairs: Pair[]) => Promise<void>
  onClose: () => void
}

function AddEnvModal({ onAdd, onClose }: AddEnvModalProps) {
  const [env, setEnv] = useState('')
  const [pairs, setPairs] = useState<Pair[]>([{ key: '', value: '' }])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const addRow = () => setPairs(prev => [...prev, { key: '', value: '' }])

  const removeRow = (i: number) =>
    setPairs(prev => prev.length > 1 ? prev.filter((_, idx) => idx !== i) : prev)

  const updatePair = (i: number, field: keyof Pair, val: string) =>
    setPairs(prev => prev.map((p, idx) => idx === i ? { ...p, [field]: val } : p))

  const submit = async () => {
    if (!env.trim()) {
      setError('Environment name is required')
      return
    }
    const validPairs = pairs.filter(p => p.key.trim())
    if (validPairs.length === 0) {
      setError('At least one secret key is required')
      return
    }
    setLoading(true)
    setError(null)
    try {
      await onAdd(env.trim(), validPairs)
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create environment')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-lg mx-4 shadow-xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold">Add Environment</h3>
          <button onClick={onClose} className="p-1 rounded hover:bg-accent transition-colors">
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Environment name
            </label>
            <input
              value={env}
              onChange={e => setEnv(e.target.value)}
              placeholder="e.g. production, staging"
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Secrets
            </label>
            <div className="space-y-2">
              {pairs.map((pair, i) => (
                <div key={i} className="flex items-center gap-2">
                  <input
                    value={pair.key}
                    onChange={e => updatePair(i, 'key', e.target.value)}
                    placeholder="MY_API_KEY"
                    className="flex-1 px-3 py-2 text-xs font-mono bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
                  />
                  <input
                    value={pair.value}
                    onChange={e => updatePair(i, 'value', e.target.value)}
                    placeholder="value"
                    className="flex-1 px-3 py-2 text-xs font-mono bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
                  />
                  <button
                    onClick={() => removeRow(i)}
                    disabled={pairs.length === 1}
                    className="p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors text-muted-foreground disabled:opacity-30 disabled:cursor-not-allowed"
                    aria-label="Remove row"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              ))}
            </div>
            <button
              onClick={addRow}
              className="mt-2 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              + Add row
            </button>
          </div>

          {error && (
            <div className="flex items-center gap-2 text-destructive text-xs">
              <AlertCircle className="w-3.5 h-3.5 shrink-0" />
              {error}
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 mt-5">
          <button
            onClick={onClose}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={submit}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50"
          >
            {loading && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
            Create Environment
          </button>
        </div>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Add Key modal
// ---------------------------------------------------------------------------

interface AddKeyModalProps {
  onAdd: (name: string, publicKey: string) => Promise<void>
  onClose: () => void
}

function AddKeyModal({ onAdd, onClose }: AddKeyModalProps) {
  const [name, setName] = useState('')
  const [publicKey, setPublicKey] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const submit = async () => {
    if (!name.trim() || !publicKey.trim()) {
      setError('Name and public key are required')
      return
    }
    setLoading(true)
    setError(null)
    try {
      await onAdd(name.trim(), publicKey.trim())
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to add key')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-lg mx-4 shadow-xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold">Add Authorized Key</h3>
          <button onClick={onClose} className="p-1 rounded hover:bg-accent transition-colors">
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        <div className="space-y-3">
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Display name
            </label>
            <input
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="e.g. jordan, ci-bot"
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Public key
            </label>
            <textarea
              value={publicKey}
              onChange={e => setPublicKey(e.target.value)}
              placeholder={'ssh-ed25519 AAAA... user@host\nor: age1...'}
              rows={3}
              className="w-full px-3 py-2 text-xs font-mono bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring resize-none"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Accepts SSH public keys (<code>ssh-ed25519</code>, <code>ssh-rsa</code>) or native age keys (<code>age1...</code>).
            </p>
          </div>

          {error && (
            <div className="flex items-center gap-2 text-destructive text-xs">
              <AlertCircle className="w-3.5 h-3.5 shrink-0" />
              {error}
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 mt-5">
          <button
            onClick={onClose}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={submit}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50"
          >
            {loading && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
            Add Key
          </button>
        </div>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Add Token modal
// ---------------------------------------------------------------------------

interface AddTokenModalProps {
  onAdd: (name: string) => Promise<CreatedAuthToken>
  onClose: () => void
}

function AddTokenModal({ onAdd, onClose }: AddTokenModalProps) {
  const [name, setName] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [created, setCreated] = useState<CreatedAuthToken | null>(null)
  const [copied, setCopied] = useState(false)

  const submit = async () => {
    if (!name.trim()) {
      setError('Token name is required')
      return
    }
    setLoading(true)
    setError(null)
    try {
      const result = await onAdd(name.trim())
      setCreated(result)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create token')
    } finally {
      setLoading(false)
    }
  }

  const copyToken = () => {
    if (!created) return
    navigator.clipboard.writeText(created.token).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-card border border-border rounded-xl p-6 w-full max-w-md mx-4 shadow-xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold">Add Access Token</h3>
          <button onClick={onClose} className="p-1 rounded hover:bg-accent transition-colors">
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        {created ? (
          <div className="space-y-4">
            <div className="flex items-start gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-xs text-green-700 dark:text-green-400">
              <Check className="w-3.5 h-3.5 mt-0.5 shrink-0" />
              <span>Token <strong>{created.name}</strong> created. Copy it now — it won't be shown again.</span>
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1">Token value</label>
              <div className="flex items-center gap-2 bg-muted rounded-lg px-3 py-2">
                <code className="font-mono text-sm flex-1 text-foreground">{created.token}</code>
                <button
                  onClick={copyToken}
                  className="p-1 rounded hover:bg-accent transition-colors text-muted-foreground hover:text-foreground"
                  aria-label="Copy token"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>
            </div>
            <p className="text-xs text-muted-foreground">
              Use as <code className="font-mono bg-muted px-1 py-0.5 rounded">?auth={created.token}</code> or{' '}
              <code className="font-mono bg-muted px-1 py-0.5 rounded">Authorization: Bearer {created.token}</code>.
            </p>
            <div className="flex justify-end">
              <button
                onClick={onClose}
                className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
              >
                Done
              </button>
            </div>
          </div>
        ) : (
          <>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  Token name
                </label>
                <input
                  value={name}
                  onChange={e => setName(e.target.value)}
                  onKeyDown={e => e.key === 'Enter' && submit()}
                  placeholder="e.g. jordan, ci-bot"
                  className="w-full px-3 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring"
                  autoFocus
                />
              </div>
              {error && (
                <div className="flex items-center gap-2 text-destructive text-xs">
                  <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                  {error}
                </div>
              )}
            </div>
            <div className="flex justify-end gap-2 mt-5">
              <button
                onClick={onClose}
                className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={submit}
                disabled={loading}
                className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {loading && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                Generate Token
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function SecretsPage() {
  const [keys, setKeys] = useState<SecretsKey[]>([])
  const [envs, setEnvs] = useState<SecretsEnvMeta[]>([])
  const [authTokens, setAuthTokens] = useState<AuthToken[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [showAddKey, setShowAddKey] = useState(false)
  const [showAddEnv, setShowAddEnv] = useState(false)
  const [showAddToken, setShowAddToken] = useState(false)

  const refresh = useCallback(() => {
    return Promise.all([api.getSecretsKeys(), api.getSecretsEnvs(), api.getAuthTokens()])
      .then(([k, e, t]) => {
        setKeys(k)
        setEnvs(e)
        setAuthTokens(t)
      })
      .catch(err => setError(err.message))
  }, [])

  useEffect(() => {
    refresh().finally(() => setLoading(false))
  }, [refresh])

  useSSE(refresh)

  const handleAddKey = async (name: string, publicKey: string) => {
    await api.addSecretsKey({ name, public_key: publicKey })
    refresh()
  }

  const handleRemoveKey = async (name: string) => {
    setActionError(null)
    try {
      await api.removeSecretsKey(name)
      refresh()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to remove key')
    }
  }

  const handleDeleteEnv = async (env: string) => {
    setActionError(null)
    try {
      await api.deleteSecretsEnv(env)
      refresh()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to delete env')
    }
  }

  const handleCreateEnv = async (envName: string, pairs: Pair[]) => {
    await api.createSecretsEnv({ env: envName, pairs })
    refresh()
  }

  const handleAddToken = async (name: string): Promise<CreatedAuthToken> => {
    const result = await api.createAuthToken(name)
    refresh()
    return result
  }

  const handleDeleteToken = async (name: string) => {
    setActionError(null)
    try {
      await api.deleteAuthToken(name)
      refresh()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to delete token')
    }
  }

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
    <>
      {showAddKey && (
        <AddKeyModal
          onAdd={handleAddKey}
          onClose={() => setShowAddKey(false)}
        />
      )}
      {showAddEnv && (
        <AddEnvModal
          onAdd={handleCreateEnv}
          onClose={() => setShowAddEnv(false)}
        />
      )}
      {showAddToken && (
        <AddTokenModal
          onAdd={handleAddToken}
          onClose={() => setShowAddToken(false)}
        />
      )}

      <div className="max-w-3xl mx-auto p-6">
        {/* Action error */}
        {actionError && (
          <div className="flex items-center gap-2 text-destructive text-xs bg-destructive/10 border border-destructive/20 rounded-lg px-3 py-2 mb-4">
            <AlertCircle className="w-3.5 h-3.5 shrink-0" />
            {actionError}
          </div>
        )}

        {/* Header */}
        <div className="mb-6">
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Lock className="w-5 h-5 text-muted-foreground" />
            Secrets
          </h2>
          <p className="text-sm text-muted-foreground mt-0.5">
            AGE-encrypted env files. Authorized keys and key names are safe to commit — values never leave your machine.
          </p>
        </div>

        {/* Recipients */}
        <section className="bg-card border border-border rounded-xl p-4 mb-4">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <KeyRound className="w-4 h-4 text-muted-foreground" />
              <h3 className="text-sm font-semibold">Authorized Keys</h3>
              <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded-md">
                {keys.length}
              </span>
            </div>
            <button
              onClick={() => setShowAddKey(true)}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
              Add Key
            </button>
          </div>

          {keys.length === 0 ? (
            <div className="text-center py-6">
              <p className="text-sm text-muted-foreground mb-3">No keys configured yet.</p>
              <code className="text-xs bg-muted px-2 py-1 rounded font-mono text-muted-foreground">
                sdlc secrets keys add --name me --key "$(cat ~/.ssh/id_ed25519.pub)"
              </code>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {keys.map(key => (
                <div key={key.name} className="flex items-center justify-between py-2.5 first:pt-0 last:pb-0">
                  <div className="flex items-center gap-3">
                    <div className="w-6 h-6 rounded-full bg-muted flex items-center justify-center shrink-0">
                      <KeyRound className="w-3 h-3 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm font-medium">{key.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">
                        {key.type} · {key.short_id}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <span>{new Date(key.added_at).toLocaleDateString()}</span>
                    <button
                      onClick={() => handleRemoveKey(key.name)}
                      className="p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors"
                      aria-label={`Remove key ${key.name}`}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {keys.length > 0 && (
            <div className="mt-3 pt-3 border-t border-border flex items-center gap-2 text-xs text-muted-foreground">
              <span>After key changes, rekey:</span>
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">sdlc secrets keys rekey</code>
              <CopyButton text="sdlc secrets keys rekey" />
            </div>
          )}
        </section>

        {/* Environments */}
        <section className="bg-card border border-border rounded-xl p-4 mb-4">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Lock className="w-4 h-4 text-muted-foreground" />
              <h3 className="text-sm font-semibold">Environments</h3>
              <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded-md">
                {envs.length}
              </span>
            </div>
            <button
              onClick={() => setShowAddEnv(true)}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
              Add Environment
            </button>
          </div>

          {envs.length === 0 ? (
            <div className="text-center py-6">
              <p className="text-sm text-muted-foreground mb-3">No encrypted env files yet.</p>
              <code className="text-xs bg-muted px-2 py-1 rounded font-mono text-muted-foreground">
                sdlc secrets env set production API_KEY=value
              </code>
            </div>
          ) : (
            <div className="space-y-3">
              {envs.map(env => (
                <div key={env.env} className="border border-border rounded-lg p-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <p className="text-sm font-semibold font-mono">{env.env}</p>
                      <p className="text-xs text-muted-foreground mt-0.5">
                        {env.key_names.length} key{env.key_names.length !== 1 ? 's' : ''} ·{' '}
                        updated {new Date(env.updated_at).toLocaleDateString()}
                      </p>
                    </div>
                    <button
                      onClick={() => handleDeleteEnv(env.env)}
                      className="p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors text-muted-foreground"
                      aria-label={`Delete ${env.env} env`}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {env.key_names.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {env.key_names.map(name => (
                        <span
                          key={name}
                          className="text-xs font-mono bg-muted text-muted-foreground px-1.5 py-0.5 rounded"
                        >
                          {name}
                        </span>
                      ))}
                    </div>
                  )}

                  {/* Copy export command */}
                  <div className="mt-3 flex items-center gap-1.5 text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1.5">
                    <code className="font-mono flex-1">
                      eval $(sdlc secrets env export {env.env})
                    </code>
                    <CopyButton text={`eval $(sdlc secrets env export ${env.env})`} />
                  </div>

                  {/* Set secret CLI hint */}
                  <div className="mt-1.5 flex items-center gap-1.5 text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1.5">
                    <code className="font-mono flex-1">
                      sdlc secrets env set {env.env} KEY=value
                    </code>
                    <CopyButton text={`sdlc secrets env set ${env.env} KEY=value`} />
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Tunnel Access */}
        <section className="bg-card border border-border rounded-xl p-4 mb-4">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-muted-foreground" />
              <h3 className="text-sm font-semibold">Tunnel Access</h3>
              <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded-md">
                {authTokens.length}
              </span>
            </div>
            <button
              onClick={() => setShowAddToken(true)}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
              Add Token
            </button>
          </div>

          {authTokens.length === 0 ? (
            <div className="text-center py-6">
              <p className="text-sm text-muted-foreground mb-1">No tokens — tunnel is open to anyone with the URL.</p>
              <p className="text-xs text-muted-foreground mb-3">
                Add a token to restrict access. All requests must include the token once any token exists.
              </p>
              <code className="text-xs bg-muted px-2 py-1 rounded font-mono text-muted-foreground">
                sdlc auth token add &lt;name&gt;
              </code>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {authTokens.map(tok => (
                <div key={tok.name} className="flex items-center justify-between py-2.5 first:pt-0 last:pb-0">
                  <div className="flex items-center gap-3">
                    <div className="w-6 h-6 rounded-full bg-muted flex items-center justify-center shrink-0">
                      <Shield className="w-3 h-3 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm font-medium">{tok.name}</p>
                      <p className="text-xs text-muted-foreground">
                        added {new Date(tok.created_at).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDeleteToken(tok.name)}
                    className="p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors text-muted-foreground"
                    aria-label={`Revoke token ${tok.name}`}
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )}

          {authTokens.length > 0 && (
            <div className="mt-3 pt-3 border-t border-border flex items-center gap-2 text-xs text-muted-foreground">
              <span>CLI:</span>
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">sdlc auth token list</code>
              <CopyButton text="sdlc auth token list" />
            </div>
          )}
        </section>

        {/* Info */}
        <div className="flex items-start gap-2 px-4 py-3 rounded-lg bg-accent/30 text-muted-foreground text-xs">
          <Lock className="w-3.5 h-3.5 mt-0.5 shrink-0" />
          <span>
            Encrypted files are stored in <code className="font-mono">.sdlc/secrets/</code> and are safe to commit.
            Decryption requires your private SSH key — the server never handles plaintext secrets.
          </span>
        </div>
      </div>
    </>
  )
}
