import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { Skeleton } from '@/components/shared/Skeleton'
import { GitBranch, Sparkles } from 'lucide-react'
import type { DocsSseEvent } from '@/lib/types'

export function ArchitecturePage() {
  const [architecture, setArchitecture] = useState<{ content: string; exists: boolean } | null>(null)
  const [loading, setLoading] = useState(true)
  const [aligning, setAligning] = useState(false)

  const fetchArchitecture = useCallback(() => {
    api.getArchitecture()
      .then(a => setArchitecture(a))
      .catch(() => setArchitecture({ content: '', exists: false }))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    fetchArchitecture()
  }, [fetchArchitecture])

  const onDocsEvent = useCallback((event: DocsSseEvent) => {
    if (event.type === 'architecture_align_completed') {
      setAligning(false)
      fetchArchitecture()
    }
  }, [fetchArchitecture])

  useSSE(() => {}, undefined, undefined, undefined, onDocsEvent)

  const handleAlign = () => {
    setAligning(true)
    api.runArchitectureAlign().catch(() => setAligning(false))
  }

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2.5">
          <GitBranch className="w-5 h-5 text-muted-foreground" />
          <h2 className="text-xl font-semibold">Architecture</h2>
        </div>
        <button
          onClick={handleAlign}
          disabled={aligning}
          className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-md border border-border text-muted-foreground hover:text-foreground hover:border-foreground/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Sparkles className="w-3.5 h-3.5" />
          {aligning ? 'Aligning…' : 'Align'}
        </button>
      </div>

      {loading && (
        <div className="space-y-3">
          <Skeleton width="w-full" className="h-4" />
          <Skeleton width="w-4/5" className="h-4" />
          <Skeleton width="w-3/5" className="h-4" />
        </div>
      )}

      {!loading && (!architecture?.exists || !architecture?.content) && (
        <div className="border border-dashed border-border rounded-xl p-10 text-center">
          <GitBranch className="w-8 h-8 text-muted-foreground/30 mx-auto mb-3" />
          <p className="text-sm text-muted-foreground">No architecture document yet.</p>
          <p className="text-xs text-muted-foreground/60 mt-1">
            Create <code className="text-primary">ARCHITECTURE.md</code> to document your system architecture.
          </p>
        </div>
      )}

      {!loading && architecture?.exists && architecture?.content && (
        <div className="prose prose-invert prose-sm max-w-none">
          <MarkdownContent content={architecture.content} />
        </div>
      )}
    </div>
  )
}
