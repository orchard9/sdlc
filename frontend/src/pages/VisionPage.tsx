import { useCallback, useEffect, useState } from 'react'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { Skeleton } from '@/components/shared/Skeleton'
import { Target, Sparkles } from 'lucide-react'
import type { DocsSseEvent } from '@/lib/types'

export function VisionPage() {
  const [vision, setVision] = useState<{ content: string; exists: boolean } | null>(null)
  const [loading, setLoading] = useState(true)
  const [aligning, setAligning] = useState(false)

  const fetchVision = useCallback(() => {
    api.getVision()
      .then(v => setVision(v))
      .catch(() => setVision({ content: '', exists: false }))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    fetchVision()
  }, [fetchVision])

  const onDocsEvent = useCallback((event: DocsSseEvent) => {
    if (event.type === 'vision_align_completed') {
      setAligning(false)
      fetchVision()
    }
  }, [fetchVision])

  useSSE(() => {}, undefined, undefined, undefined, onDocsEvent)

  const handleAlign = () => {
    setAligning(true)
    api.runVisionAlign().catch(() => setAligning(false))
  }

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2.5">
          <Target className="w-5 h-5 text-muted-foreground" />
          <h2 className="text-xl font-semibold">Vision</h2>
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

      {!loading && (!vision?.exists || !vision?.content) && (
        <div className="border border-dashed border-border rounded-xl p-10 text-center">
          <Target className="w-8 h-8 text-muted-foreground/30 mx-auto mb-3" />
          <p className="text-sm text-muted-foreground">No vision document yet.</p>
          <p className="text-xs text-muted-foreground/60 mt-1">
            Create <code className="text-primary">VISION.md</code> to define your project's direction.
          </p>
        </div>
      )}

      {!loading && vision?.exists && vision?.content && (
        <div className="prose prose-invert prose-sm max-w-none">
          <MarkdownContent content={vision.content} />
        </div>
      )}
    </div>
  )
}
