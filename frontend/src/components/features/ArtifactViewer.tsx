import { useState } from 'react'
import { api } from '@/api/client'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import type { Artifact } from '@/lib/types'
import { CheckCircle, XCircle, FileText, Maximize2, MinusCircle } from 'lucide-react'

interface ArtifactViewerProps {
  slug: string
  artifact: Artifact
  onStatusChange?: () => void
}

export function ArtifactViewer({ slug, artifact, onStatusChange }: ArtifactViewerProps) {
  const [fullscreen, setFullscreen] = useState(false)
  const [rejectMode, setRejectMode] = useState(false)
  const [rejectReason, setRejectReason] = useState('')
  const [waiveMode, setWaiveMode] = useState(false)
  const [waiveReason, setWaiveReason] = useState('')

  const handleApprove = async () => {
    await api.approveArtifact(slug, artifact.artifact_type, 'ui')
    onStatusChange?.()
  }

  const handleRejectConfirm = async () => {
    await api.rejectArtifact(slug, artifact.artifact_type, rejectReason || undefined)
    setRejectMode(false)
    setRejectReason('')
    onStatusChange?.()
  }

  const handleWaiveConfirm = async () => {
    await api.waiveArtifact(slug, artifact.artifact_type, waiveReason || undefined)
    setWaiveMode(false)
    setWaiveReason('')
    onStatusChange?.()
  }

  const canApproveReject = artifact.status === 'draft' || artifact.status === 'needs_fix'
  const canWaive = artifact.status === 'missing' || artifact.status === 'draft'

  return (
    <>
      <div className="border border-border rounded-lg overflow-hidden">
        <div className="flex items-center justify-between px-3 py-2 bg-card/50 border-b border-border">
          <div className="flex items-center gap-2">
            <FileText className="w-3.5 h-3.5 text-muted-foreground" />
            <span className="text-sm font-medium">{artifact.artifact_type.replace(/_/g, ' ')}</span>
            <StatusBadge status={artifact.status} />
          </div>
          <div className="flex items-center gap-1.5">
            {artifact.content && (
              <button
                onClick={() => setFullscreen(true)}
                className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                title="Fullscreen"
              >
                <Maximize2 className="w-3.5 h-3.5" />
              </button>
            )}
            {canApproveReject && !rejectMode && !waiveMode && (
              <>
                <button
                  onClick={handleApprove}
                  className="flex items-center gap-1 px-2 py-1 text-xs rounded-md bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/30 transition-colors"
                >
                  <CheckCircle className="w-3 h-3" />
                  Approve
                </button>
                <button
                  onClick={() => setRejectMode(true)}
                  className="flex items-center gap-1 px-2 py-1 text-xs rounded-md bg-red-600/20 text-red-400 hover:bg-red-600/30 transition-colors"
                >
                  <XCircle className="w-3 h-3" />
                  Reject
                </button>
              </>
            )}
            {canWaive && !rejectMode && !waiveMode && (
              <button
                onClick={() => setWaiveMode(true)}
                className="flex items-center gap-1 px-2 py-1 text-xs rounded-md bg-neutral-600/20 text-neutral-400 hover:bg-neutral-600/30 transition-colors"
              >
                <MinusCircle className="w-3 h-3" />
                Waive
              </button>
            )}
          </div>
        </div>

        {rejectMode && (
          <div className="flex items-center gap-2 px-3 py-2 bg-red-950/20 border-b border-border">
            <input
              autoFocus
              value={rejectReason}
              onChange={e => setRejectReason(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') handleRejectConfirm(); if (e.key === 'Escape') { setRejectMode(false); setRejectReason('') } }}
              placeholder="Reason (optional)"
              className="flex-1 text-xs bg-muted/60 border border-border/50 px-2 py-1 rounded-md text-foreground"
            />
            <button
              onClick={handleRejectConfirm}
              className="text-xs px-2 py-1 rounded-md bg-red-600/20 text-red-400 hover:bg-red-600/30 transition-colors"
            >
              Confirm
            </button>
            <button
              onClick={() => { setRejectMode(false); setRejectReason('') }}
              className="text-xs px-2 py-1 rounded-md bg-muted/60 text-muted-foreground hover:bg-muted transition-colors"
            >
              Cancel
            </button>
          </div>
        )}

        {waiveMode && (
          <div className="flex items-center gap-2 px-3 py-2 bg-neutral-900/40 border-b border-border">
            <input
              autoFocus
              value={waiveReason}
              onChange={e => setWaiveReason(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') handleWaiveConfirm(); if (e.key === 'Escape') { setWaiveMode(false); setWaiveReason('') } }}
              placeholder="Reason (optional)"
              className="flex-1 text-xs bg-muted/60 border border-border/50 px-2 py-1 rounded-md text-foreground"
            />
            <button
              onClick={handleWaiveConfirm}
              className="text-xs px-2 py-1 rounded-md bg-neutral-600/20 text-neutral-400 hover:bg-neutral-600/30 transition-colors"
            >
              Confirm
            </button>
            <button
              onClick={() => { setWaiveMode(false); setWaiveReason('') }}
              className="text-xs px-2 py-1 rounded-md bg-muted/60 text-muted-foreground hover:bg-muted transition-colors"
            >
              Cancel
            </button>
          </div>
        )}

        {artifact.content && (
          <div className="p-4 max-h-96 overflow-y-auto">
            <MarkdownContent content={artifact.content} />
          </div>
        )}
        {!artifact.content && artifact.status === 'missing' && (
          <div className="p-4 text-xs text-muted-foreground">Not created yet</div>
        )}
      </div>

      {artifact.content && (
        <FullscreenModal
          open={fullscreen}
          onClose={() => setFullscreen(false)}
          title={artifact.artifact_type}
        >
          <MarkdownContent content={artifact.content} />
        </FullscreenModal>
      )}
    </>
  )
}
