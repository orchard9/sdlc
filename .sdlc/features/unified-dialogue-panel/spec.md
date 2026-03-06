# Spec: UnifiedDialoguePanel

## Problem

Two nearly-identical dialogue panel components exist side-by-side:

- `frontend/src/components/ponder/DialoguePanel.tsx` — used by PonderPage
- `frontend/src/components/investigation/InvestigationDialoguePanel.tsx` — used by InvestigationPage

Both components contain:
- The same `McpCallCard` sub-component (different `span` label text only)
- The same `WorkingPlaceholder` sub-component (identical)
- The same `InputBar` sub-component (identical logic, different placeholder text)
- The same scroll management logic (identical)
- The same session loading pattern (identical, just different API calls)
- The same SSE subscription pattern (identical shape, different event types)
- The same pending-message optimistic UI (identical)

The only meaningful differences are:
1. The **header strip** — ponder shows `OrientationStrip + TeamRow`; investigation shows `PhaseStrip`
2. The **empty state** — ponder has a "Start from title & brief" button and commit shortcut; investigation has a simpler message
3. The **API calls** — `api.startPonderChat` vs `api.startInvestigationChat`, etc.
4. The **SSE event types** — `ponder_run_*` vs `investigation_run_*`
5. The **MCP call label** — `sdlc_ponder_chat` vs `sdlc_investigation_chat`

Maintaining two near-identical components creates drift risk, increases future maintenance cost, and violates the DRY principle.

## Solution

Extract a single `UnifiedDialoguePanel` component that:

1. Owns the shared structure: scroll area, session list, pending-message overlay, input bar
2. Accepts an **adapter prop** (`DialoguePanelAdapter`) that provides all domain-specific behavior via a well-typed interface
3. Renders a **header slot** passed in as a React node (render prop) — letting each workspace render its own strip (OrientationStrip, PhaseStrip, or nothing)
4. Accepts an **empty-state slot** — letting each workspace define its own zero-state content
5. The adapter provides:
   - `loadSessions(slug)` — returns `Promise<SessionContent[]>`
   - `startChat(slug, message?)` — returns chat response
   - `stopChat(slug)` — stops the running session
   - `mcpLabel` — string shown in the McpCallCard header
   - `sseEventType` — discriminates which SSE event family to subscribe to (`'ponder' | 'investigation'`)
   - `isRunStarted(event)` / `isRunCompleted(event)` / `isRunStopped(event)` — type-narrowing helpers OR a single `onSseEvent` callback

## Scope

- Extract `UnifiedDialoguePanel` to `frontend/src/components/shared/UnifiedDialoguePanel.tsx`
- Create `PonderDialogueAdapter` and `InvestigationDialogueAdapter` that conform to the adapter interface
- Update `DialoguePanel.tsx` to be a thin wrapper (or replace entirely with adapter-based usage in PonderPage)
- Update `InvestigationDialoguePanel.tsx` to be a thin wrapper (or replace entirely in InvestigationPage)
- Preserve all existing behavior exactly — no UX changes

## Out of Scope

- Changes to the server or API
- Changes to SSE event types
- Changes to the WorkspacePanel (artifact sidebar)
- Changes to session rendering (SessionBlock stays as-is)
- New features or visible UX changes

## Acceptance Criteria

1. A single `UnifiedDialoguePanel` component exists in `frontend/src/components/shared/`
2. `DialoguePanel.tsx` (ponder) and `InvestigationDialoguePanel.tsx` (investigation) either delegate to it or are replaced
3. PonderPage renders identically to before (same header, same empty state with "Start from title & brief", same commit shortcut)
4. InvestigationPage renders identically to before (same PhaseStrip, same simpler empty state)
5. Auto-scroll, scroll-locking, and pending-message overlay behave identically
6. SSE event handling behaves identically — no events are missed
7. `SDLC_NO_NPM=1 cargo test --all` passes
8. `cargo clippy --all -- -D warnings` passes
9. Frontend builds with no TypeScript errors (`cd frontend && npm run build`)
