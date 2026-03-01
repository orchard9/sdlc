import { useState } from 'react'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import type { Artifact } from '@/lib/types'
import { FileText, Maximize2 } from 'lucide-react'

interface ArtifactViewerProps {
  artifact: Artifact
}

export function ArtifactViewer({ artifact }: ArtifactViewerProps) {
  const [fullscreen, setFullscreen] = useState(false)

  return (
    <>
      <div className="border border-border rounded-lg overflow-hidden">
        <div className="flex items-center justify-between px-3 py-2 bg-card/50 border-b border-border">
          <div className="flex items-center gap-2">
            <FileText className="w-3.5 h-3.5 text-muted-foreground" />
            <span className="text-sm font-medium">{artifact.artifact_type.replace(/_/g, ' ')}</span>
            <StatusBadge status={artifact.status} testId="artifact-status" />
          </div>
          {artifact.content && (
            <button
              onClick={() => setFullscreen(true)}
              className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
              title="Fullscreen"
            >
              <Maximize2 className="w-3.5 h-3.5" />
            </button>
          )}
        </div>

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
