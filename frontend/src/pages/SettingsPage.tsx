import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import type { ProjectConfig } from '@/lib/types'
import { useSSE } from '@/hooks/useSSE'
import { Loader2, Terminal, Info } from 'lucide-react'

export function SettingsPage() {
  const [config, setConfig] = useState<ProjectConfig | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(() => {
    api.getConfig()
      .then(setConfig)
      .catch(err => setError(err.message))
  }, [])

  useEffect(() => {
    api.getConfig()
      .then(setConfig)
      .catch(err => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

  useSSE(refresh)

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

  if (!config) return null

  const commandEntries = Object.entries(config.platform?.commands ?? {})

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="mb-6">
        <h2 className="text-xl font-semibold">Config</h2>
        <p className="text-sm text-muted-foreground mt-0.5">{config.project.name}</p>
      </div>

      {/* Platform Commands */}
      <section className="bg-card border border-border rounded-xl p-4 mb-4">
        <div className="flex items-center gap-2 mb-3">
          <Terminal className="w-4 h-4 text-muted-foreground" />
          <h3 className="text-sm font-semibold">Platform Commands</h3>
        </div>
        {commandEntries.length === 0 ? (
          <p className="text-sm text-muted-foreground italic">No platform commands</p>
        ) : (
          <dl className="space-y-3">
            {commandEntries.map(([name, cmd]) => (
              <div key={name}>
                <dt className="text-foreground font-mono text-xs font-semibold">{name}</dt>
                <dd className="text-sm text-muted-foreground mt-0.5">{cmd.description}</dd>
                {cmd.script && (
                  <dd className="text-xs text-muted-foreground font-mono mt-0.5">{cmd.script}</dd>
                )}
              </div>
            ))}
          </dl>
        )}
      </section>

      <div className="flex items-start gap-2 px-4 py-3 rounded-lg bg-accent/30 text-muted-foreground text-xs mb-8">
        <Info className="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <span>Configuration is read from .sdlc/config.yaml. Edit the file and commit to change settings.</span>
      </div>
    </div>
  )
}
