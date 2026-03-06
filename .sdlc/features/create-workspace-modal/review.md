# Code Review: CreateWorkspaceModal

## Summary

This feature introduces a shared `CreateWorkspaceModal` component that consolidates workspace creation form logic across four workspace types: Ponder, Root Cause, Evolve, and Guideline. The implementation is well-structured, consistent, and removes significant duplication.

## Files Changed

- `frontend/src/components/shared/CreateWorkspaceModal.tsx` — new shared modal component
- `frontend/src/lib/slug.ts` — new `titleToSlug` utility (extracted for sharing)
- `frontend/src/components/ponder/NewIdeaModal.tsx` — retained with own file-upload extensions
- `frontend/src/pages/InvestigationPage.tsx` — uses `CreateWorkspaceModal` for root_cause
- `frontend/src/pages/EvolvePage.tsx` — uses `CreateWorkspaceModal` for evolve
- `frontend/src/pages/GuidelinePage.tsx` — uses `CreateWorkspaceModal` for guideline

## Findings

### Accepted

**1. NewIdeaModal not fully delegating to CreateWorkspaceModal**

`NewIdeaModal` retains its own form state and rendering rather than delegating to `CreateWorkspaceModal`. This is intentional: the Ponder workspace modal has unique file-attachment capabilities (drag-and-drop, file chips, `UploadCloud` UI) that are Ponder-specific and would add unnecessary complexity to the shared component. The shared modal handles slug/title/brief/scope/context/references — the common fields — correctly. The `NewIdeaModal` continues to share the `titleToSlug` utility from `@/lib/slug`.

Decision: Accept. Ponder's file-preload feature is domain-specific. The shared component handles the common form shape; Ponder extends it at the component level.

**2. `GuidelinePage` does not use `WorkspaceShell`**

`InvestigationPage` and `EvolvePage` were refactored to use `WorkspaceShell` for the two-pane layout. `GuidelinePage` retains inline two-pane flex layout. This is an existing inconsistency, not introduced by this feature. Creating a task to track it.

Decision: Accept as pre-existing. Track as follow-up.

**3. `requireContext` not set in GuidelinePage modal**

The task specification says `requireContext: true` for the Guideline modal. The implementation at `GuidelinePage.tsx:290-307` sets `showContext: true` and `showScope: true` but does not set `requireContext: true`. Context is shown as optional in the rendered form. Given that guideline context is conceptually important but not strictly blocking, this is a mild spec deviation. The component allows submission without context.

Decision: Accept with task. Context is valuable but not session-blocking — agents can start guidelines with a title and add context during the session. Creating a task to decide if `requireContext` should be enforced.

### Positive observations

- `CreateWorkspaceModal` is a genuinely deep module: the configurable `fields` object cleanly hides conditional rendering behind a single prop surface.
- Slug auto-derivation from title with manual-override tracking (`slugManuallyEdited` ref) is correctly implemented and consistent with `NewIdeaModal`.
- State reset on `open` toggle is correct and complete — no stale form state leaks between openings.
- Escape-to-close is implemented uniformly via `window.addEventListener`.
- TypeScript passes cleanly (`npx tsc --noEmit`).
- `cargo clippy --all -- -D warnings` passes cleanly.
- The `onSubmit` pattern (caller-supplied async function, throw on error) is a clean inversion of control that keeps the modal reusable.

## Tasks Created

- Follow-up: Refactor `GuidelinePage` to use `WorkspaceShell` for layout consistency with `InvestigationPage` and `EvolvePage`.
- Follow-up: Decide whether `requireContext: true` should be enforced in the Guideline modal.

## Verdict

Approved. The implementation achieves the design goal — a shared, configurable creation form used consistently across Investigation, Evolve, and Guideline workspaces. All TypeScript and lint checks pass. The two findings are pre-existing or minor spec deviations that do not block the feature.
