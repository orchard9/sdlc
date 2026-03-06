# QA Plan: CreateWorkspaceModal

## Scope

Verify that the shared `CreateWorkspaceModal` component works correctly across all four workspace creation flows, and that no existing functionality is broken.

## Test Cases

### TC1: CreateWorkspaceModal component exists

- File `frontend/src/components/shared/CreateWorkspaceModal.tsx` exists
- Component exports `CreateWorkspaceModal` as a named export
- TypeScript types for `WorkspaceFieldConfig` and `CreateWorkspaceModalProps` are exported

### TC2: Auto-slug derivation

- Open any creation modal (Ponder, Evolve, Guideline, Root Cause)
- Type a title — slug field auto-updates to the slugified form
- Manually edit the slug field — auto-derivation stops
- Close and reopen modal — auto-derivation resumes for new input

### TC3: Escape key closes modal

- Open modal
- Press Escape — modal closes without submitting

### TC4: Submit guard

- Leave title empty — submit button is disabled
- Leave slug empty — submit button is disabled
- For root_cause / evolve / guideline: leave context empty — submit button is disabled
- For ponder: context is not required — submit is enabled with just title + slug

### TC5: Ponder creation (NewIdeaModal)

- Open PonderPage, click "+" to create new idea
- Modal shows: Title, Slug, Description (optional), References (optional)
- Fill title + slug — submit button enabled
- Submit — ponder entry created, modal closes, list updates

### TC6: Root Cause creation (InvestigationPage)

- Open InvestigationPage (root cause tab or `/investigations`)
- Click "+" button — modal opens titled "New Root Cause"
- Modal shows: Title, Slug, Context
- Context is required — submit disabled until filled
- Submit — investigation created with `kind: 'root_cause'`

### TC7: Evolve creation (EvolvePage)

- Open EvolvePage (`/evolve`)
- Click "+" — modal opens titled "New Evolve Session"
- Modal shows: Title, Slug, Scope (optional), Context (required)
- Fill title + slug + context — submit enabled even without scope
- Submit — investigation created with `kind: 'evolve'`; scope saved if provided

### TC8: Guideline creation (GuidelinePage)

- Open GuidelinePage (`/guidelines`)
- Click "+" — modal opens titled "New Guideline"
- Modal shows: Title, Slug, Scope (optional), Context (required)
- Submit — investigation created with `kind: 'guideline'`; scope saved if provided

### TC9: Error handling

- Simulate API failure (duplicate slug)
- Modal shows error message, stays open, re-enables submit button

### TC10: TypeScript compilation

```bash
cd frontend && npx tsc --noEmit
```

Must exit 0 with no errors.

### TC11: No deleted inline form regressions

- `NewInvestigationForm` component no longer exists in `InvestigationPage.tsx`
- `NewEvolveForm` component no longer exists in `EvolvePage.tsx`
- `NewGuidelineForm` component no longer exists in `GuidelinePage.tsx`
- All three pages render `CreateWorkspaceModal` in their place

### TC12: Backend tests still pass

```bash
SDLC_NO_NPM=1 cargo test --all
```

Must pass (no backend changes expected, but verify nothing was inadvertently broken).

## Pass Criteria

All 12 test cases pass. TypeScript compiles clean. `cargo test --all` passes.
