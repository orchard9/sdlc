import { useEffect, useState } from 'react'
import { api } from '@/api/client'
import { StatusBadge } from '@/components/shared/StatusBadge'
import type { Artifact } from '@/lib/types'
import { CheckCircle, XCircle, FileText } from 'lucide-react'

interface ArtifactViewerProps {
  slug: string
  artifactType: string
  onStatusChange?: () => void
}

export function ArtifactViewer({ slug, artifactType, onStatusChange }: ArtifactViewerProps) {
  const [artifact, setArtifact] = useState<Artifact | null>(null)
  const [loading, setLoading] = useState(true)

  const load = async () => {
    try {
      setLoading(true)
      const data = await api.getArtifact(slug, artifactType)
      setArtifact(data)
    } catch {
      // artifact may not exist yet
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [slug, artifactType])

  const handleApprove = async () => {
    await api.approveArtifact(slug, artifactType, 'ui')
    await load()
    onStatusChange?.()
  }

  const handleReject = async () => {
    const reason = prompt('Rejection reason (optional):')
    await api.rejectArtifact(slug, artifactType, reason ?? undefined)
    await load()
    onStatusChange?.()
  }

  if (loading) return <div className="text-xs text-muted-foreground">Loading...</div>
  if (!artifact) return null

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2 bg-card/50 border-b border-border">
        <div className="flex items-center gap-2">
          <FileText className="w-3.5 h-3.5 text-muted-foreground" />
          <span className="text-sm font-medium">{artifactType.replace(/_/g, ' ')}</span>
          <StatusBadge status={artifact.status} />
        </div>
        {(artifact.status === 'draft' || artifact.status === 'needs_fix') && (
          <div className="flex items-center gap-1.5">
            <button
              onClick={handleApprove}
              className="flex items-center gap-1 px-2 py-1 text-xs rounded-md bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/30 transition-colors"
            >
              <CheckCircle className="w-3 h-3" />
              Approve
            </button>
            <button
              onClick={handleReject}
              className="flex items-center gap-1 px-2 py-1 text-xs rounded-md bg-red-600/20 text-red-400 hover:bg-red-600/30 transition-colors"
            >
              <XCircle className="w-3 h-3" />
              Reject
            </button>
          </div>
        )}
      </div>
      {artifact.content && (
        <div className="p-4 max-h-96 overflow-y-auto">
          <pre className="text-xs whitespace-pre-wrap text-muted-foreground font-mono">{artifact.content}</pre>
        </div>
      )}
      {!artifact.content && artifact.status === 'missing' && (
        <div className="p-4 text-xs text-muted-foreground">Not created yet</div>
      )}
    </div>
  )
}
