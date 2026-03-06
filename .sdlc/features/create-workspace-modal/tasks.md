# Tasks: CreateWorkspaceModal

## T1: Create CreateWorkspaceModal component

Create `frontend/src/components/shared/CreateWorkspaceModal.tsx`.

The component must:
- Accept `open`, `onClose`, `onCreated`, `title`, `submitLabel`, `initialTitle`, `initialSlug`, `fields`, and `onSubmit` props
- Manage form state: `title`, `slug`, `brief`, `scope`, `context`, `refs[]`, `submitting`, `error`
- Auto-derive slug from title using `titleToSlug()` until user manually edits the slug field
- Auto-focus the title input on open (`useEffect` + `useRef`)
- Close on Escape key
- Reset all state on open
- Conditionally render: `brief` textarea if `fields.showBrief`, `scope` input if `fields.showScope`, `context` textarea if `fields.showContext`, references section if `fields.showReferences`
- Submit guard: require `slug`, `title`, and `context` (if `fields.requireContext`)
- Call `props.onSubmit(values)` on submit; on error, display the error message and re-enable the form
- Call `props.onCreated(slug)` on success
- Use same modal card/backdrop pattern as existing modals (same Tailwind classes as `NewIdeaModal`)

## T2: Update NewIdeaModal to use CreateWorkspaceModal

Refactor `frontend/src/components/ponder/NewIdeaModal.tsx` to delegate rendering to `CreateWorkspaceModal`.

`NewIdeaModal` still exists and keeps the same public API (preserving any existing imports in `PonderPage`), but its internals call `CreateWorkspaceModal` with `fields={{ showBrief: true, showReferences: true }}`.

The `onSubmit` callback in `NewIdeaModal` handles:
- `api.createPonderEntry({ slug, title, brief })`
- If references provided: `api.capturePonderArtifact(slug, { filename: 'references.md', content: refMd })`
- `api.startPonderChat(slug, seed).catch(() => {})`

After refactor, `NewIdeaModal` should be a thin wrapper with no duplicated form state of its own.

## T3: Replace InvestigationPage inline form with CreateWorkspaceModal

In `frontend/src/pages/InvestigationPage.tsx`:
- Remove the `NewInvestigationForm` inline component and its local state/handlers
- Add `showCreateModal` state (boolean)
- Change the "+" button to `setShowCreateModal(true)`
- Render `CreateWorkspaceModal` with:
  - `title="New Root Cause"`
  - `fields={{ showContext: true, contextPlaceholder: "Describe what broke...", requireContext: true }}`
  - `onSubmit` calls `api.createInvestigation({ slug, title, kind: 'root_cause', context })`

## T4: Replace EvolvePage inline form with CreateWorkspaceModal

In `frontend/src/pages/EvolvePage.tsx`:
- Remove the `NewEvolveForm` inline component
- Add `showCreateModal` state
- Change the "+" button to `setShowCreateModal(true)`
- Render `CreateWorkspaceModal` with:
  - `title="New Evolve Session"`
  - `fields={{ showScope: true, scopePlaceholder: "scope — e.g. crates/sdlc-server/src/", showContext: true, contextPlaceholder: "What are you improving?", requireContext: true }}`
  - `onSubmit` calls `api.createInvestigation({ slug, title, kind: 'evolve', context })` then `api.updateInvestigation(slug, { scope }).catch(() => {})`

## T5: Replace GuidelinePage inline form with CreateWorkspaceModal

In `frontend/src/pages/GuidelinePage.tsx`:
- Remove the `NewGuidelineForm` inline component
- Add `showCreateModal` state
- Change the "+" button to `setShowCreateModal(true)`
- Render `CreateWorkspaceModal` with:
  - `title="New Guideline"`
  - `fields={{ showScope: true, scopePlaceholder: "scope — files or modules this applies to (optional)", showContext: true, contextPlaceholder: "Why does this keep going wrong?", requireContext: true }}`
  - `onSubmit` calls `api.createInvestigation({ slug, title, kind: 'guideline', context })` then `api.updateInvestigation(slug, { scope }).catch(() => {})`

## T6: TypeScript and lint validation

Run:
```bash
cd frontend && npx tsc --noEmit
```

Fix any TypeScript errors introduced by the refactor. Confirm no lint errors with `cargo clippy --all -- -D warnings` (backend unchanged, but confirm nothing breaks).
