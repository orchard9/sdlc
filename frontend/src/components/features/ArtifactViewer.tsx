import { useState } from 'react'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { FullscreenModal } from '@/components/shared/FullscreenModal'
import type { Artifact } from '@/lib/types'
import { FileText, Maximize2 } from 'lucide-react'

interface ArtifactViewerProps {
  artifact: Artifact
}

function extractTeaser(content: string, maxLen = 120): string {
  const lines = content.split('\n')
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('---') && !trimmed.startsWith('```')) {
      return trimmed.length > maxLen ? trimmed.slice(0, maxLen) + '…' : trimmed
    }
  }
  return ''
}

function formatRelativeTime(dateStr: string): string {
  const delta = Date.now() - new Date(dateStr).getTime()
  const s = Math.floor(delta / 1000)
  if (s < 60) return `${s}s ago`
  const m = Math.floor(s / 60)
  if (m < 60) return `${m}m ago`
  const h = Math.floor(m / 60)
  if (h < 24) return `${h}h ago`
  return `${Math.floor(h / 24)}d ago`
}

export function ArtifactViewer({ artifact }: ArtifactViewerProps) {
  const [fullscreen, setFullscreen] = useState(false)

  const teaser = artifact.content ? extractTeaser(artifact.content) : ''
  const timestamp = artifact.approved_at ?? artifact.waived_at ?? null

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

        {teaser && (
          <div className="flex items-center justify-between px-3 py-1.5 border-b border-border/50 bg-muted/20">
            <span
              data-testid="artifact-teaser"
              className="text-xs text-muted-foreground truncate max-w-[75%]"
            >
              {teaser}
            </span>
            {timestamp && (
              <span className="text-xs text-muted-foreground/70 shrink-0 ml-2">
                {formatRelativeTime(timestamp)}
              </span>
            )}
          </div>
        )}

        {artifact.content && (
          <div className="p-4">
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
          hasToc
        >
          <MarkdownContent content={artifact.content} showToc />
        </FullscreenModal>
      )}
    </>
  )
}
