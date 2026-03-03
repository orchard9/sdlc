# Tasks: ponder-creation-modal

## T1: Create `NewIdeaModal` component

Create `frontend/src/components/ponder/NewIdeaModal.tsx`.

- Props: `open`, `onClose`, `onCreated`, `initialTitle?`, `initialSlug?`, `initialBrief?`
- State: `title`, `slug`, `brief`, `refs: string[]` (starts as `['']`), `submitting`, `error`
- Fields: Title input (required, auto-focus on open), Slug input (derived, editable), Description textarea (6 rows), References URL list
- References list: dynamic rows; "Add reference" button appends empty string; remove button per row
- Slug auto-derivation: uses the same `titleToSlug` logic from `PonderPage.tsx`; breaks link on manual edit
- Submit logic: validate title+slug → createPonderEntry → capturePonderArtifact (if refs) → startPonderChat → onCreated(slug)
- Error display below the form
- Keyboard: Escape calls onClose; no Enter-to-submit
- Styling: `fixed inset-0 z-50`, backdrop `bg-black/60`, card `bg-card border border-border rounded-xl shadow-xl max-w-xl mx-4 max-h-[85vh] flex flex-col`

## T2: Update `PonderPage.tsx` to use `NewIdeaModal`

- Remove `NewIdeaForm` function entirely
- Import `NewIdeaModal`
- Replace `{showForm && <NewIdeaForm ... />}` with `<NewIdeaModal open={showForm} ... />`
- Also remove the border-b block that `NewIdeaForm` occupied in the sidebar (the form no longer lives inline)
- Pass `initialTitle`, `initialSlug`, `initialBrief` props from existing prefill state
- `onClose` and `onCreated` handlers clear prefill state and navigate as before

## T3: Verify references artifact saved correctly

Manual verification task (can be done as part of QA):
- Create a ponder entry with one or two reference URLs
- Confirm `references.md` appears in the ponder's artifacts panel
- Confirm file content is a markdown list

_This task is tracked here for completeness; it is validated by the QA plan._
