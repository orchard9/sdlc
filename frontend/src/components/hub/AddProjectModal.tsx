import { useEffect, useState } from 'react'
import { CheckCircle2, Download, Loader2, Plus, Upload, X } from 'lucide-react'
import { api } from '@/api/client'
import type { CreateRepoResponse } from '@/lib/types'
import { cn } from '@/lib/utils'

const NAME_RE = /^[a-z0-9][a-z0-9-]*$/

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  return (
    <button
      onClick={() => {
        navigator.clipboard.writeText(text).catch(() => {})
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      }}
      className={cn(
        'inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors',
        copied ? 'border-emerald-400/30 bg-emerald-500/10 text-emerald-100' : 'border-border bg-background/70 text-muted-foreground hover:bg-accent hover:text-foreground',
      )}
    >
      {copied ? 'Copied' : 'Copy'}
    </button>
  )
}

export function AddProjectModal({
  open,
  onClose,
  onChanged,
}: {
  open: boolean
  onClose: () => void
  onChanged: () => void
}) {
  const [mode, setMode] = useState<'create' | 'import'>('create')
  const [name, setName] = useState('')
  const [createState, setCreateState] = useState<'idle' | 'creating' | 'done' | 'error'>('idle')
  const [createError, setCreateError] = useState<string | null>(null)
  const [createResult, setCreateResult] = useState<CreateRepoResponse | null>(null)

  const [url, setUrl] = useState('')
  const [pat, setPat] = useState('')
  const [importState, setImportState] = useState<'idle' | 'importing' | 'done' | 'error'>('idle')
  const [importError, setImportError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [open, onClose])

  useEffect(() => {
    if (open) return
    setMode('create')
    setName('')
    setCreateState('idle')
    setCreateError(null)
    setCreateResult(null)
    setUrl('')
    setPat('')
    setImportState('idle')
    setImportError(null)
  }, [open])

  if (!open) return null

  const nameError =
    name.length > 0 && !NAME_RE.test(name)
      ? 'Lowercase letters, numbers, and hyphens only'
      : name.length > 100
        ? 'Name must be 100 characters or fewer'
        : null

  const handleCreate = async () => {
    if (!name.trim() || nameError) return
    setCreateState('creating')
    setCreateError(null)
    try {
      const result = await api.createRepo(name.trim())
      setCreateResult(result)
      setCreateState('done')
      onChanged()
    } catch (error) {
      setCreateState('error')
      setCreateError(error instanceof Error ? error.message : 'Failed to create repo')
    }
  }

  const handleImport = async () => {
    if (!url.trim()) return
    setImportState('importing')
    setImportError(null)
    try {
      await api.importRepo(url.trim(), pat.trim() || undefined)
      setImportState('done')
      setUrl('')
      setPat('')
      onChanged()
    } catch (error) {
      setImportState('error')
      setImportError(error instanceof Error ? error.message : 'Import failed')
    }
  }

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto bg-black/70 px-4 py-6">
      <div className="mx-auto max-w-3xl overflow-hidden rounded-xl border border-border bg-card shadow-2xl">
        <div className="flex items-start justify-between gap-4 px-6 py-5 border-b border-border">
          <div>
            <div className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Hub</div>
            <h2 className="mt-1 text-xl font-semibold tracking-tight">Add Project</h2>
            <p className="text-sm text-muted-foreground mt-2">Create a new deployment repo or import an existing codebase into the fleet.</p>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg border border-border bg-background p-2 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="px-6 pt-5">
          <div className="inline-flex rounded-lg border border-border bg-muted/30 p-1">
            <button
              onClick={() => setMode('create')}
              className={cn('rounded-md px-4 py-2 text-sm transition-colors', mode === 'create' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground')}
            >
              Create
            </button>
            <button
              onClick={() => setMode('import')}
              className={cn('rounded-md px-4 py-2 text-sm transition-colors', mode === 'import' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground')}
            >
              Import
            </button>
          </div>
        </div>

        <div className="px-6 py-6">
          {mode === 'create' ? (
            <div className="space-y-5">
              {createState === 'done' && createResult ? (
                <div className="space-y-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                  <div className="flex items-center gap-2 text-emerald-100">
                    <CheckCircle2 className="w-5 h-5" />
                    <span className="font-medium">{createResult.repo_slug} created</span>
                  </div>
                  <div>
                    <div className="mb-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Add remote</div>
                    <div className="flex items-center gap-3">
                      <code className="flex-1 break-all rounded-lg border border-border bg-background/70 px-4 py-3 text-xs">
                        git remote add gitea {createResult.push_url}
                      </code>
                      <CopyButton text={`git remote add gitea ${createResult.push_url}`} />
                    </div>
                  </div>
                  <div>
                    <div className="mb-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Push</div>
                    <div className="flex items-center gap-3">
                      <code className="flex-1 rounded-lg border border-border bg-background/70 px-4 py-3 text-xs">
                        git push gitea main
                      </code>
                      <CopyButton text="git push gitea main" />
                    </div>
                  </div>
                  <button
                    onClick={() => {
                      setName('')
                      setCreateResult(null)
                      setCreateState('idle')
                    }}
                    className="inline-flex items-center gap-2 rounded-lg border border-border px-4 py-2 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                  >
                    <Plus className="w-4 h-4" />
                    Create another
                  </button>
                </div>
              ) : (
                <>
                  <div>
                    <label className="text-sm font-medium">Project name</label>
                    <div className="mt-2 flex flex-col md:flex-row gap-3">
                      <input
                        value={name}
                        onChange={event => {
                          setName(event.target.value)
                          if (createState === 'error') setCreateState('idle')
                        }}
                        onKeyDown={event => event.key === 'Enter' && handleCreate()}
                        placeholder="project-name"
                        className={cn(
                          'flex-1 rounded-lg border bg-background/70 px-4 py-3 text-sm outline-none transition-colors',
                          nameError ? 'border-rose-500' : 'border-border focus:border-primary',
                        )}
                      />
                      <button
                        onClick={handleCreate}
                        disabled={!name.trim() || !!nameError || createState === 'creating'}
                        className="inline-flex items-center justify-center gap-2 rounded-lg bg-primary px-5 py-3 text-sm font-medium text-primary-foreground disabled:opacity-50"
                      >
                        {createState === 'creating' ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
                        {createState === 'creating' ? 'Creating' : 'Create repo'}
                      </button>
                    </div>
                    <div className="mt-2 text-xs text-muted-foreground">Lowercase letters, numbers, and hyphens.</div>
                    {nameError && <div className="mt-2 text-sm text-rose-300">{nameError}</div>}
                    {createState === 'error' && createError && <div className="mt-2 text-sm text-rose-300">{createError}</div>}
                  </div>

                  <div className="rounded-xl border border-border bg-background/40 p-5">
                    <div className="text-sm font-medium tracking-tight">What happens next</div>
                    <div className="mt-2 space-y-2 text-sm text-muted-foreground">
                      <div>1. Hub creates a repo in the fleet’s Gitea org.</div>
                      <div>2. You push your project to the provided deployment remote.</div>
                      <div>3. If provisioning is enabled, the cluster starts a workspace automatically.</div>
                    </div>
                  </div>
                </>
              )}
            </div>
          ) : (
            <div className="space-y-5">
              <div>
                <label className="text-sm font-medium">Clone URL</label>
                <input
                  value={url}
                  onChange={event => setUrl(event.target.value)}
                  placeholder="https://github.com/org/repo"
                  className="mt-2 w-full rounded-lg border border-border bg-background/70 px-4 py-3 text-sm outline-none transition-colors focus:border-primary"
                />
              </div>
              <div>
                <label className="text-sm font-medium">Personal access token</label>
                <input
                  value={pat}
                  onChange={event => setPat(event.target.value)}
                  type="password"
                  placeholder="Optional, for private repos"
                  className="mt-2 w-full rounded-lg border border-border bg-background/70 px-4 py-3 text-sm outline-none transition-colors focus:border-primary"
                />
              </div>
              <div className="flex items-center gap-3">
                <button
                  onClick={handleImport}
                  disabled={!url.trim() || importState === 'importing'}
                  className="inline-flex items-center gap-2 rounded-lg bg-primary px-5 py-3 text-sm font-medium text-primary-foreground disabled:opacity-50"
                >
                  {importState === 'importing' ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                  {importState === 'importing' ? 'Importing' : 'Import repo'}
                </button>
                {importState === 'done' && (
                  <div className="inline-flex items-center gap-2 text-sm text-emerald-200">
                    <Upload className="w-4 h-4" />
                    Imported and handed to provisioning.
                  </div>
                )}
              </div>
              {importState === 'error' && importError && <div className="text-sm text-rose-300">{importError}</div>}
              <div className="rounded-xl border border-border bg-background/40 p-5">
                <div className="text-sm font-medium tracking-tight">Import flow</div>
                <div className="mt-2 space-y-2 text-sm text-muted-foreground">
                  <div>1. Hub mirrors the external repository into the fleet’s Gitea org.</div>
                  <div>2. Provisioning starts a workspace for the imported slug.</div>
                  <div>3. The project appears in the fleet as soon as provisioning or heartbeats land.</div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
