# Design: UnifiedDialoguePanel

## Overview

Extract the shared dialogue panel structure from `DialoguePanel.tsx` (ponder) and
`InvestigationDialoguePanel.tsx` (investigation) into a single
`UnifiedDialoguePanel` component. Both existing panels will be replaced by
thin wrappers that pass domain-specific adapters.

## Component Architecture

```
UnifiedDialoguePanel<TRunState extends BaseRunState>
  props:
    slug: string
    adapter: DialoguePanelAdapter
    header?: ReactNode          // OrientationStrip, PhaseStrip, or null
    emptyState?: ReactNode      // custom zero-state content
    onRefresh: () => void
```

### DialoguePanelAdapter interface

```typescript
interface DialoguePanelAdapter {
  // Data loading
  loadSessions: (slug: string) => Promise<SessionContent[]>

  // Chat control
  startChat: (slug: string, message?: string) => Promise<ChatResponse>
  stopChat: (slug: string) => Promise<void>

  // Live indicator
  mcpLabel: string   // e.g. "sdlc_ponder_chat" or "sdlc_investigation_chat"

  // SSE wiring — adapter provides one of:
  //   sseEventType: 'ponder' | 'investigation'
  // Used by UnifiedDialoguePanel to subscribe to the right useSSE callback param
  sseEventType: 'ponder' | 'investigation'

  // Slug filter — adapters check event.slug === slug themselves via the component
  // The unified panel calls the right handler based on sseEventType
}

interface ChatResponse {
  status: 'started' | 'conflict'
  session: number
  owner_name: string
}
```

### Sub-components (extracted shared, housed in UnifiedDialoguePanel.tsx)

| Sub-component | Notes |
|---|---|
| `McpCallCard` | Receives `mcpLabel` prop instead of hardcoded string |
| `WorkingPlaceholder` | Identical, extracted as local function |
| `InputBar` | Receives `placeholder` prop; existing placeholder strings become adapter config or per-usage prop |

### Adapters

**`PonderDialogueAdapter`** (defined in or near `DialoguePanel.tsx`):
```typescript
const PonderDialogueAdapter: DialoguePanelAdapter = {
  loadSessions: async (slug) => {
    const metas = await api.getPonderSessions(slug)
    return Promise.all(metas.map(m => api.getPonderSession(slug, m.session)))
  },
  startChat: (slug, msg) => api.startPonderChat(slug, msg),
  stopChat: (slug) => api.stopPonderChat(slug),
  mcpLabel: 'sdlc_ponder_chat',
  sseEventType: 'ponder',
}
```

**`InvestigationDialogueAdapter`** (defined in or near `InvestigationDialoguePanel.tsx`):
```typescript
const InvestigationDialogueAdapter: DialoguePanelAdapter = {
  loadSessions: async (slug) => {
    const metas = await api.getInvestigationSessions(slug)
    return Promise.all(metas.map(m => api.getInvestigationSession(slug, m.session)))
  },
  startChat: (slug, msg) => api.startInvestigationChat(slug, msg),
  stopChat: (slug) => api.stopInvestigationChat(slug),
  mcpLabel: 'sdlc_investigation_chat',
  sseEventType: 'investigation',
}
```

### SSE dispatch inside UnifiedDialoguePanel

The unified panel subscribes via `useSSE`. Based on `adapter.sseEventType` it registers:

```typescript
useSSE(
  handleUpdate,
  adapter.sseEventType === 'ponder' ? handlePonderEvent : undefined,
  undefined,
  adapter.sseEventType === 'investigation' ? handleInvestigationEvent : undefined,
)
```

Both handlers follow the same structure: filter by `event.slug === slug`, then:
- `*_run_completed` → set idle, clear pending, reload sessions, call `onRefresh`
- `*_run_stopped` → set idle, clear pending

### RunState type

A shared `DialogueRunState` type replaces the two near-identical types:

```typescript
type DialogueRunState =
  | { status: 'idle' }
  | { status: 'running'; session: number; ownerName: string; ownerMessage: string | null }
  | { status: 'stopped'; session: number }
```

The existing `PonderRunState` and `InvestigationRunState` remain in `types.ts` for
backward compatibility (they are used in the old panel props today). After replacement
they can be removed — tracked as a follow-up task.

## File Layout

```
frontend/src/components/shared/
  UnifiedDialoguePanel.tsx      ← new (the shared core)

frontend/src/components/ponder/
  DialoguePanel.tsx             ← becomes thin wrapper: passes PonderDialogueAdapter
                                   + ponder-specific header/emptyState

frontend/src/components/investigation/
  InvestigationDialoguePanel.tsx ← becomes thin wrapper: passes InvestigationDialogueAdapter
                                    + investigation-specific header/emptyState
```

No page files change. PonderPage and InvestigationPage continue to import from the
same paths as today.

## Data Flow

```
PonderPage
  └── DialoguePanel (thin wrapper)
        └── UnifiedDialoguePanel
              ├── adapter = PonderDialogueAdapter
              ├── header = <OrientationStrip> + <TeamRow>
              ├── emptyState = <ZeroStateCommitButton> etc.
              └── (all shared scroll/session/SSE logic)

InvestigationPage
  └── InvestigationDialoguePanel (thin wrapper)
        └── UnifiedDialoguePanel
              ├── adapter = InvestigationDialogueAdapter
              ├── header = <PhaseStrip>
              ├── emptyState = simple text message
              └── (all shared scroll/session/SSE logic)
```

## Preserved Behaviors

| Behavior | Preserved by |
|---|---|
| Auto-scroll to bottom on new content | Unchanged scroll logic in unified panel |
| Scroll-lock when user scrolls up | Unchanged `userScrolledUp` ref logic |
| Optimistic pending-message overlay | Unchanged pending state structure |
| Stop button while running | Unchanged InputBar logic |
| "Start from title & brief" empty button | Passed as `emptyState` ReactNode from DialoguePanel wrapper |
| Commit shortcut (ZeroStateCommitButton) | Passed as `emptyState` ReactNode from DialoguePanel wrapper |
| Phase strip in investigation | Passed as `header` ReactNode from InvestigationDialoguePanel wrapper |
| Orientation + team strips in ponder | Passed as `header` ReactNode from DialoguePanel wrapper |

## Non-Goal

The `InputBar` placeholder text differs (`"Add a thought, constraint, or question..."` vs
`"Add context or answer questions..."`). This is trivially added as a `placeholder` prop to
`InputBar`. Each adapter or wrapper provides the correct string.

## Implementation Order

1. Create `frontend/src/components/shared/UnifiedDialoguePanel.tsx` with `DialoguePanelAdapter` interface and unified logic
2. Update `DialoguePanel.tsx` to be a thin wrapper (preserve all ponder-specific header/emptyState rendering)
3. Update `InvestigationDialoguePanel.tsx` to be a thin wrapper
4. Verify no TypeScript errors: `cd frontend && npm run build`
5. Manual smoke test both PonderPage and InvestigationPage
