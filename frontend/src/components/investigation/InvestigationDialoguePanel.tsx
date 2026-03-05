import { api } from '@/api/client'
import { PhaseStrip } from './PhaseStrip'
import { UnifiedDialoguePanel, type DialoguePanelAdapter } from '@/components/shared/UnifiedDialoguePanel'
import type { InvestigationDetail } from '@/lib/types'

// ---------------------------------------------------------------------------
// Phase sequences by kind
// ---------------------------------------------------------------------------

const PHASE_SEQUENCES: Record<string, string[]> = {
  root_cause: ['triage', 'investigate', 'synthesize', 'output'],
  evolve: ['survey', 'analyze', 'paths', 'roadmap', 'output'],
  guideline: ['problem', 'evidence', 'principles', 'draft', 'publish'],
}

// ---------------------------------------------------------------------------
// Investigation dialogue adapter
// ---------------------------------------------------------------------------

const InvestigationDialogueAdapter: DialoguePanelAdapter = {
  loadSessions: async (slug) => {
    const metas = await api.getInvestigationSessions(slug)
    return Promise.all(metas.map(m => api.getInvestigationSession(slug, m.session)))
  },
  startChat: (slug, message) => api.startInvestigationChat(slug, message),
  stopChat: (slug) => api.stopInvestigationChat(slug),
  mcpLabel: 'sdlc_investigation_chat',
  sseEventType: 'investigation',
  inputPlaceholder: 'Add context or answer questions...',
}

// ---------------------------------------------------------------------------
// Main InvestigationDialoguePanel — thin wrapper around UnifiedDialoguePanel
// ---------------------------------------------------------------------------

interface Props {
  entry: InvestigationDetail
  onRefresh: () => void
}

export function InvestigationDialoguePanel({ entry, onRefresh }: Props) {
  const { slug } = entry

  const phases = PHASE_SEQUENCES[entry.kind] ?? []

  const header = <PhaseStrip phases={phases} current={entry.phase} />

  const emptyState = (
    <div className="flex flex-col items-center justify-center h-full text-center gap-3">
      <p className="text-sm text-muted-foreground/60">No sessions yet. Start a session to begin investigating.</p>
      <p className="text-xs text-muted-foreground/40 max-w-xs">
        The agent will work through this investigation phase by phase,
        writing findings as artifacts here.
      </p>
      <p className="text-xs text-muted-foreground/30 mt-1">
        Add context below or just hit send.
      </p>
    </div>
  )

  return (
    <UnifiedDialoguePanel
      slug={slug}
      adapter={InvestigationDialogueAdapter}
      header={header}
      emptyState={emptyState}
      artifacts={entry.artifacts}
      onRefresh={onRefresh}
    />
  )
}
