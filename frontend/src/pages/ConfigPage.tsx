import { useEffect, useState, useCallback } from 'react'
import { api } from '@/api/client'
import { Loader2, Pencil, Save, X } from 'lucide-react'

export function ConfigPage() {
  const [config, setConfig] = useState<Record<string, unknown> | null>(null)
  const [loading, setLoading] = useState(true)
  const [editing, setEditing] = useState(false)
  const [editText, setEditText] = useState('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchConfig = useCallback(() => {
    setLoading(true)
    api.getAgentsConfig()
      .then(data => setConfig(data as Record<string, unknown>))
      .catch(err => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    fetchConfig()
  }, [fetchConfig])

  const handleEdit = () => {
    setEditText(JSON.stringify(config, null, 2))
    setError(null)
    setEditing(true)
  }

  const handleCancel = () => {
    setEditing(false)
    setEditText('')
    setError(null)
  }

  const handleSave = async () => {
    setError(null)
    let parsed: Record<string, unknown>
    try {
      parsed = JSON.parse(editText)
    } catch {
      setError('Invalid JSON')
      return
    }

    setSaving(true)
    try {
      await api.putAgentsConfig(parsed)
      setEditing(false)
      setEditText('')
      fetchConfig()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Save failed')
    } finally {
      setSaving(false)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">Agent Configuration</h2>
        {!editing && (
          <button
            onClick={handleEdit}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-card border border-border hover:bg-muted transition-colors"
          >
            <Pencil className="w-3.5 h-3.5" />
            Edit
          </button>
        )}
      </div>

      {error && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
        </div>
      )}

      <div className="bg-card border border-border rounded-xl p-4">
        {editing ? (
          <>
            <textarea
              value={editText}
              onChange={e => setEditText(e.target.value)}
              spellCheck={false}
              className="w-full min-h-[400px] bg-background border border-border rounded-lg p-3 text-xs font-mono text-foreground resize-y focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <div className="flex items-center gap-2 mt-3">
              <button
                onClick={handleSave}
                disabled={saving}
                className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {saving ? (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                ) : (
                  <Save className="w-3.5 h-3.5" />
                )}
                Save
              </button>
              <button
                onClick={handleCancel}
                disabled={saving}
                className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-card border border-border hover:bg-muted transition-colors disabled:opacity-50"
              >
                <X className="w-3.5 h-3.5" />
                Cancel
              </button>
            </div>
          </>
        ) : (
          <pre className="text-xs font-mono text-muted-foreground whitespace-pre-wrap">
            {JSON.stringify(config, null, 2)}
          </pre>
        )}
      </div>
    </div>
  )
}
