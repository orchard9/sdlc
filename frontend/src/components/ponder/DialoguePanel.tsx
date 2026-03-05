import { GitMerge, Loader2, Zap } from 'lucide-react'
import { api } from '@/api/client'
import { OrientationStrip } from './OrientationStrip'
import { TeamRow } from './TeamRow'
import { UnifiedDialoguePanel, type DialoguePanelAdapter } from '@/components/shared/UnifiedDialoguePanel'
import type { PonderDetail } from '@/lib/types'

// ---------------------------------------------------------------------------
// Ponder dialogue adapter
// ---------------------------------------------------------------------------

const PonderDialogueAdapter: DialoguePanelAdapter = {
  loadSessions: async (slug) => {
    const metas = await api.getPonderSessions(slug)
    return Promise.all(metas.map(m => api.getPonderSession(slug, m.session)))
  },
  startChat: (slug, message) => api.startPonderChat(slug, message),
  stopChat: (slug) => api.stopPonderChat(slug),
  mcpLabel: 'sdlc_ponder_chat',
  sseEventType: 'ponder',
  inputPlaceholder: 'Add a thought, constraint, or question...',
}

// ---------------------------------------------------------------------------
// Zero-state commit shortcut
// ---------------------------------------------------------------------------

function ZeroStateCommitButton({
  onCommit,
  running,
}: {
  onCommit: () => void
  running: boolean
}) {
  return (
    <button
      onClick={onCommit}
      disabled={running}
      className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg border border-border/40 text-muted-foreground/50 hover:text-foreground hover:border-border hover:bg-accent/40 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      title="Commit this ponder — synthesize milestones and mark committed"
    >
      {running
        ? <Loader2 className="w-3 h-3 animate-spin" />
        : <GitMerge className="w-3 h-3" />}
      <span>{running ? 'Committing…' : 'Commit anyway'}</span>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Main DialoguePanel — thin wrapper around UnifiedDialoguePanel
// ---------------------------------------------------------------------------

interface Props {
  entry: PonderDetail
  onRefresh: () => void
  onCommit?: () => void
  commitRunning?: boolean
}

export function DialoguePanel({ entry, onRefresh, onCommit, commitRunning = false }: Props) {
  const { slug } = entry

  const header = (
    <>
      {entry.team.length > 0 && <TeamRow team={entry.team} />}
      <OrientationStrip orientation={entry.orientation ?? null} />
    </>
  )

  const emptyState = (
    <div className="flex flex-col items-center justify-center h-full text-center gap-3">
      <p className="text-sm text-muted-foreground/60">No sessions yet.</p>
      <p className="text-xs text-muted-foreground/40 max-w-xs">
        The agent will interview this idea, recruit thought partners, and write the
        dialogue here.
      </p>
      {entry.status !== 'committed' && entry.status !== 'parked' && (
        <button
          onClick={() => {
            const briefArtifact = entry.artifacts.find(a => a.filename === 'brief.md')
            const seed = briefArtifact?.content
              ? `${entry.title}\n\n${briefArtifact.content.trim()}`
              : entry.title
            api.startPonderChat(slug, seed).catch(() => {})
          }}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
        >
          <Zap className="w-3 h-3" />
          Start from title &amp; brief
        </button>
      )}
      <p className="text-xs text-muted-foreground/30 -mt-1">
        or add a seed thought below
      </p>
      {entry.status !== 'committed' && entry.status !== 'parked' && onCommit && (
        <ZeroStateCommitButton onCommit={onCommit} running={commitRunning} />
      )}
    </div>
  )

  return (
    <UnifiedDialoguePanel
      slug={slug}
      adapter={PonderDialogueAdapter}
      header={header}
      emptyState={emptyState}
      artifacts={entry.artifacts}
      onRefresh={onRefresh}
    />
  )
}
