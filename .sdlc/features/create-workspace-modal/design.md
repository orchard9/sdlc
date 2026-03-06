# Design: CreateWorkspaceModal

## Overview

Extract duplicated workspace creation forms from four pages into a single shared `CreateWorkspaceModal` component. The modal is UI-only — it renders the form, handles state, and calls an `onSubmit` callback. Callers own the API logic.

## Component Location

```
frontend/src/components/shared/CreateWorkspaceModal.tsx
```

## Architecture

```
CreateWorkspaceModal
  Props:
    - open: boolean
    - onClose: () => void
    - onCreated: (slug: string) => void
    - title: string                  // Modal header
    - submitLabel?: string           // Default: "Create"
    - initialTitle?: string
    - initialSlug?: string
    - fields?: WorkspaceFieldConfig
    - onSubmit: (values) => Promise<void>

  WorkspaceFieldConfig:
    - showBrief?: boolean             // Ponder's "Description" textarea
    - briefPlaceholder?: string
    - showReferences?: boolean        // Ponder's reference URL list
    - showScope?: boolean             // Evolve + Guideline scope input
    - scopePlaceholder?: string
    - showContext?: boolean           // RootCause/Evolve/Guideline context textarea
    - contextPlaceholder?: string
    - requireContext?: boolean        // Default false

  Internal State:
    - title, slug, brief, scope, context, refs[], submitting, error
    - slugManuallyEdited ref (breaks auto-derive when user edits slug)

  Behavior:
    - Auto-focus title input on open
    - Auto-derive slug from title (until user manually edits slug)
    - Escape key closes modal
    - Submit guard: requires slug + title + (context if requireContext)
    - Resets all state on open
```

## Callers After Refactor

### PonderPage / NewIdeaModal

```tsx
<CreateWorkspaceModal
  open={showModal}
  onClose={() => setShowModal(false)}
  onCreated={handleCreated}
  title="New Idea"
  submitLabel="Create Idea"
  fields={{
    showBrief: true,
    briefPlaceholder: "Expand on the idea...",
    showReferences: true,
  }}
  onSubmit={async ({ slug, title, brief, references }) => {
    await api.createPonderEntry({ slug, title, brief })
    if (references?.length) {
      const refMd = `# References\n\n${references.map(r => `- ${r}`).join('\n')}\n`
      await api.capturePonderArtifact(slug, { filename: 'references.md', content: refMd })
    }
    api.startPonderChat(slug, brief ? `${title}\n\n${brief}` : title).catch(() => {})
  }}
/>
```

### InvestigationPage (root_cause)

```tsx
<CreateWorkspaceModal
  open={showModal}
  onClose={() => setShowModal(false)}
  onCreated={handleCreated}
  title="New Root Cause"
  submitLabel="Create"
  fields={{
    showContext: true,
    contextPlaceholder: "What broke?",
    requireContext: true,
  }}
  onSubmit={async ({ slug, title, context }) => {
    await api.createInvestigation({ slug, title, kind: 'root_cause', context })
  }}
/>
```

### EvolvePage

```tsx
<CreateWorkspaceModal
  open={showModal}
  onClose={() => setShowModal(false)}
  onCreated={handleCreated}
  title="New Evolve Session"
  submitLabel="Create"
  fields={{
    showScope: true,
    scopePlaceholder: "scope — e.g. crates/sdlc-server/src/",
    showContext: true,
    contextPlaceholder: "What are you improving?",
    requireContext: true,
  }}
  onSubmit={async ({ slug, title, scope, context }) => {
    await api.createInvestigation({ slug, title, kind: 'evolve', context })
    if (scope) await api.updateInvestigation(slug, { scope }).catch(() => {})
  }}
/>
```

### GuidelinePage

```tsx
<CreateWorkspaceModal
  open={showModal}
  onClose={() => setShowModal(false)}
  onCreated={handleCreated}
  title="New Guideline"
  submitLabel="Create"
  fields={{
    showScope: true,
    scopePlaceholder: "scope — files or modules this applies to (optional)",
    showContext: true,
    contextPlaceholder: "Why does this keep going wrong?",
    requireContext: true,
  }}
  onSubmit={async ({ slug, title, scope, context }) => {
    await api.createInvestigation({ slug, title, kind: 'guideline', context })
    if (scope) await api.updateInvestigation(slug, { scope }).catch(() => {})
  }}
/>
```

## Migration Strategy

1. Create `CreateWorkspaceModal.tsx`
2. Update `NewIdeaModal.tsx` to use `CreateWorkspaceModal` internally (preserves any existing imports)
3. Replace inline forms in `InvestigationPage`, `EvolvePage`, `GuidelinePage` with `CreateWorkspaceModal`
4. Remove dead inline form components from each page

## Visual Layout

See [Mockup](mockup.html) for the rendered UI states.

The modal uses the same card/backdrop pattern as existing modals (`FullscreenModal`, `AlignModal`, `SearchModal`):
- Backdrop: `fixed inset-0 bg-black/60`
- Card: `bg-card border border-border rounded-xl shadow-xl w-full max-w-xl mx-4`
- Header: title + close button, `border-b border-border`
- Body: scrollable form fields
- Footer: Cancel + Submit, `border-t border-border`

Field order: Title → Slug → [Scope?] → [Brief/Context?] → [References?]
