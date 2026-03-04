# Code Review: Human UAT Frontend — Submission Modal and Secondary Buttons

## Summary

Four files were created or modified to ship this feature:

1. `frontend/src/components/shared/HumanUatModal.tsx` — new shared modal component
2. `frontend/src/components/milestones/MilestonePreparePanel.tsx` — updated `VerifyingMini`
3. `frontend/src/pages/FeatureDetail.tsx` — updated `run_qa` action card
4. `frontend/src/api/client.ts` — three new API helpers

`npx tsc --noEmit` passes. `SDLC_NO_NPM=1 cargo build --all` passes.

## Findings

### HumanUatModal.tsx

**Pass.** The component is clean and well-structured.

- State is properly reset on every `open=true` transition via the `useEffect` dependency on `open`.
- Checklist fetch is async and sets `checklistLoading` to false in `finally` — no loading state leak.
- The `fetchChecklist` function is defined inside the effect and called once — no closure issue.
- `notesRequired` is a pure helper — easy to test in isolation.
- Submit guard (`canSubmit && !submitting`) is correct and redundant-safe: even if button is rendered as disabled, the handler re-checks.
- Error handling catches both `Error` instances and unknown types gracefully.
- Keyboard (Escape) listener is added/removed correctly with deps `[open, onClose, submitting]`. No stale closure risk.
- `aria-modal` and `aria-label` on the overlay make the dialog accessible.
- No `unwrap()`-style force-casts; the non-null assertion (`verdict!`) on submit is safe because `canSubmit` guarantees `verdict !== null` before reaching that branch.

**Minor observation** (not a blocker): The checklist `<pre>` element uses `whitespace-pre-wrap` which is correct for Markdown-like content. If the content is very long the `max-h-36 overflow-y-auto` keeps it contained.

### MilestonePreparePanel.tsx

**Pass.**

- `HumanUatModal` import is added correctly.
- `useState<boolean>` for `modalOpen` is scoped inside `VerifyingMini` — appropriate locality.
- The "Submit manually" button is rendered inside `{!running && (...)}` — correctly hidden during active runs.
- The `<>` fragment wrapper around `<div>` + `<HumanUatModal>` is correct.
- No changes to `ProgressBarMini` or `MilestonePreparePanel` proper — surgical edit.

### FeatureDetail.tsx

**Pass.**

- `HumanUatModal` import added.
- `humanQaModalOpen` state added adjacent to other interaction state.
- "Submit manually" button rendered inside `classification.action === 'run_qa'` guard — correct.
- The button is inside the `!running` branch (else clause) — not shown while running.
- `HumanUatModal` rendered at the bottom of the component return, outside all `section` elements — correct placement.
- No existing functionality altered.

### client.ts

**Pass.**

- `submitHumanMilestoneUat` — correct path, method, body shape. Optional test count fields included.
- `submitHumanFeatureQa` — minimal body as per spec.
- `getMilestoneAcceptanceTest` — GET request, returns `{ content: string | null }`. Graceful because `HumanUatModal` treats any falsy content as "no checklist".

## No Issues Found

All acceptance criteria from the spec are met:

- "Submit manually" appears in both surfaces. ✓
- Modal opens and closes correctly. ✓
- Checklist renders read-only; absence shows placeholder. ✓
- Notes required for non-Pass. ✓
- Submission disabled while in-flight. ✓
- API errors surface inline. ✓
- TypeScript compiles cleanly. ✓

No `unwrap()`, no hardcoded URLs, no CORS issues (all paths are relative `/api/...`). No stale state or memory leak hazards.
