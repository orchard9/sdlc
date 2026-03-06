# Tasks: UnifiedDialoguePanel

## T1 — Create UnifiedDialoguePanel shared component

Create `frontend/src/components/shared/UnifiedDialoguePanel.tsx` containing:
- `DialoguePanelAdapter` interface
- `DialogueRunState` type
- `McpCallCard` sub-component (accepts `mcpLabel: string` prop)
- `WorkingPlaceholder` sub-component
- `InputBar` sub-component (accepts `placeholder: string` prop)
- `UnifiedDialoguePanel` main component with:
  - `slug`, `adapter`, `header?`, `emptyState?`, `onRefresh` props
  - Full scroll area with auto-scroll + scroll-lock logic
  - Session loading via `adapter.loadSessions`
  - SSE subscription via `useSSE`, dispatching based on `adapter.sseEventType`
  - Pending-message optimistic overlay
  - Send/stop via `adapter.startChat` / `adapter.stopChat`

## T2 — Refactor DialoguePanel.tsx to thin wrapper

Rewrite `frontend/src/components/ponder/DialoguePanel.tsx` to:
- Define `PonderDialogueAdapter` (object constant conforming to `DialoguePanelAdapter`)
- Render `<UnifiedDialoguePanel>` with:
  - `adapter={PonderDialogueAdapter}`
  - `header` = `<TeamRow>` + `<OrientationStrip>` nodes (conditionally rendered as before)
  - `emptyState` = the "Start from title & brief" button + ZeroStateCommitButton
- Preserve the `DialoguePanel` export signature (`entry`, `onRefresh`, `onCommit?`, `commitRunning?` props)

## T3 — Refactor InvestigationDialoguePanel.tsx to thin wrapper

Rewrite `frontend/src/components/investigation/InvestigationDialoguePanel.tsx` to:
- Define `InvestigationDialogueAdapter` (object constant conforming to `DialoguePanelAdapter`)
- Render `<UnifiedDialoguePanel>` with:
  - `adapter={InvestigationDialogueAdapter}`
  - `header` = `<PhaseStrip>` node
  - `emptyState` = the simple text message used today
- Preserve the `InvestigationDialoguePanel` export signature (`entry`, `onRefresh` props)

## T4 — Verify build and types

Run:
```bash
cd /Users/jordanwashburn/Workspace/orchard9/sdlc/frontend && npm run build
```
Fix any TypeScript errors. No runtime behavior changes.

## T5 — Run backend tests

Run:
```bash
cd /Users/jordanwashburn/Workspace/orchard9/sdlc && SDLC_NO_NPM=1 cargo test --all
```
Ensure all tests pass.
