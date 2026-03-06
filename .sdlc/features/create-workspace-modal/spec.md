# Spec: CreateWorkspaceModal — shared creation form with configurable fields

## Problem

There are four workspace types in the application: Ponder (ideas), Root Cause (investigations), Evolve (improvements), and Guideline (recurring patterns). Each workspace type has its own creation UI:

- `PonderPage` uses `NewIdeaModal` — a full-featured modal with title, slug, description, and references
- `InvestigationPage` has an inline `NewInvestigationForm` with title, slug, context (for root_cause kind)
- `EvolvePage` has an inline `NewEvolveForm` with title, slug, scope, context
- `GuidelinePage` has an inline `NewGuidelineForm` with title, slug, scope, context

This duplication means each page independently manages:
- `titleToSlug()` derivation logic
- Slug sanitization (lowercase, alphanumeric + hyphens, max 40 chars)
- Form state management (title, slug, submitting, error)
- Auto-focus and Escape-to-close behavior
- Submit guard (`!slug || !title || submitting`)

Every time a pattern changes, it must be updated in four places.

## Goal

Extract a single shared `CreateWorkspaceModal` component that can handle all workspace creation flows via configuration. The modal must:

1. Be configurable with per-workspace field sets (each workspace type shows different fields)
2. Reduce code duplication across `PonderPage`, `InvestigationPage`, `EvolvePage`, `GuidelinePage`
3. Preserve all existing UX (auto-focus, Escape close, auto-slug derivation, validation)
4. Be placed at `frontend/src/components/shared/CreateWorkspaceModal.tsx`

## Out of Scope

- Changing the actual API calls or backend behavior
- Changing the visual design of any existing page
- Adding new workspace types

## Field Configuration Per Workspace Type

| Workspace | Fields Required |
|---|---|
| Ponder (idea) | title, slug, brief (optional), references (optional) |
| Root Cause | title, slug, context (required) |
| Evolve | title, slug, scope (optional), context (required) |
| Guideline | title, slug, scope (optional), context (required) |

## Component Interface

```tsx
interface WorkspaceFieldConfig {
  /** Label/placeholder for the context/description textarea */
  contextLabel?: string
  contextPlaceholder?: string
  /** Whether to show the scope input */
  showScope?: boolean
  scopePlaceholder?: string
  /** Whether to show the brief/description textarea (Ponder style) */
  showBrief?: boolean
  briefPlaceholder?: string
  /** Whether to show the references section (Ponder only) */
  showReferences?: boolean
  /** Whether context is required for submit */
  requireContext?: boolean
}

interface CreateWorkspaceModalProps {
  open: boolean
  onClose: () => void
  onCreated: (slug: string) => void
  title: string              // Modal header text (e.g. "New Idea", "New Root Cause")
  submitLabel?: string       // Button label (e.g. "Create Idea")
  initialTitle?: string
  initialSlug?: string
  fields?: WorkspaceFieldConfig
  onSubmit: (values: {
    slug: string
    title: string
    brief?: string
    scope?: string
    context?: string
    references?: string[]
  }) => Promise<void>
}
```

The `onSubmit` callback receives the form values, and each page handles its own API calls. This keeps the modal as a pure UI component — it does not know about `api.createPonderEntry` or `api.createInvestigation`.

## Acceptance Criteria

1. `CreateWorkspaceModal` exists at `frontend/src/components/shared/CreateWorkspaceModal.tsx`
2. `NewIdeaModal` is replaced or updated to use `CreateWorkspaceModal` internally
3. `InvestigationPage` inline form is replaced by `CreateWorkspaceModal`
4. `EvolvePage` inline form is replaced by `CreateWorkspaceModal`
5. `GuidelinePage` inline form is replaced by `CreateWorkspaceModal`
6. All existing behavior preserved: auto-focus, Escape close, auto-slug, validation, submit guard
7. No TypeScript errors, no lint errors
8. `SDLC_NO_NPM=1 cargo test --all` still passes
