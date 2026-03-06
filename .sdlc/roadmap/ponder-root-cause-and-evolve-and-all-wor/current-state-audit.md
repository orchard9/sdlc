# Current State: Workspace UI Duplication Audit

## Frontend Duplication Map

### Tier 1: Near-Identical Page Shells (highest impact)
`InvestigationPage.tsx`, `EvolvePage.tsx`, `GuidelinePage.tsx` are **copy-pasted files** differing only in:
- `kind` argument to `api.getInvestigations()`
- Route prefix (`/investigations`, `/evolve`, `/guidelines`)
- One scope field name (`entry.scope` vs `entry.guideline_scope`)

Duplicated verbatim across all three:
- `STATUS_TABS` constant
- `PhaseBadge` local component
- `titleToSlug` utility (also in PonderPage — 4 copies)
- `EntryRow` component
- `EntryDetailPane` component
- Mobile bottom-sheet pattern (drag handle, translate-y, z-50)
- Skeleton loading pattern

### Tier 2: Dialogue Panel Duplication
`DialoguePanel.tsx` (Ponder) and `InvestigationDialoguePanel.tsx` (all investigation types) are structural duplicates:
- `InputBar` — identical in both
- `McpCallCard` — identical except label string
- `WorkingPlaceholder` — identical
- `handleSend`/`handleStop`/`loadSessions`/auto-scroll — identical logic

Only substantive differences:
- Ponder shows `TeamRow` + `OrientationStrip`; Investigation shows `PhaseStrip`
- Different SSE event channels (`onPonderEvent` vs `onInvestigationEvent`)
- Different API calls (`api.startPonderChat` vs `api.startInvestigationChat`)

### Tier 3: Output Gate Duplication
`OutputGate.tsx` and `EvolveOutputGate.tsx` — identical create-feature-from-investigation pattern, differing in slug prefix and artifact read.

### Tier 4: WorkspacePanel Mega-Switch
`WorkspacePanel.tsx` has 8 conditional `kind+phase` branches. It unified artifact viewing but at the cost of a god-component.

## Backend Duplication Map

### Route handlers (roadmap.rs vs investigations.rs)
- `SessionPath` struct — copy-pasted
- Session list/get handlers — near-identical bodies
- `capture_artifact` handlers — identical bodies
- CRUD handlers — same shape, different types

### Core layer (ponder.rs vs investigation.rs)
- `log_session` — same workspace delegation + manifest update pattern
- Artifact CRUD — same delegation to workspace.rs
- Both use `workspace::Orientation` and `SessionMeta`

## What's Genuinely Different (cannot unify naively)

| Concern | Ponder | Investigation |
|---------|--------|---------------|
| Lifecycle | exploring → converging → committed → parked | in_progress → complete → parked |
| Phase concept | None | Kind-specific progression |
| Team model | Full team.yaml with partners | None |
| Tags | Yes | No |
| Commitment | committed_to milestones | output_ref to features |
| Kind | Always 'ponder' | root_cause / evolve / guideline |
| Type-specific fields | None | confidence, lens_scores, evidence_counts, etc. |
| state.yaml tracking | active_ponders list | None |
| Agent tools | Playwright MCP + WebSearch | Base only (guideline adds WebSearch) |
