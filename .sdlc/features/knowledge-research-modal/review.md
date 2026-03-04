# Code Review: Knowledge Research Modal and Research Button on List View

## Summary

Two files changed, one file added. Frontend-only change. Build passes with zero TypeScript errors.

---

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/components/knowledge/NewResearchModal.tsx` | New component |
| `frontend/src/pages/KnowledgePage.tsx` | Research button in list + modal wiring |

---

## Findings

### NewResearchModal.tsx

**PASS** — Matches `NewIdeaModal` pattern faithfully: backdrop, centered card, Escape key handler, reset on open, inline error, loading state.

**PASS** — `topic.trim() || undefined` correctly converts blank input to `undefined`, letting the server fall back to the entry slug as the research topic.

**PASS** — Error handling resets `submitting` to `false` so the user can retry without reopening the modal.

**PASS** — `aria-modal`, `role="dialog"`, `aria-label` present. Close button has `aria-label="Close"`. Accessible.

**PASS** — `FlaskConical` icon in modal header matches the icon used on the list row Research button — consistent affordance.

**PASS** — `onStarted()` is called before `setResearchTarget(null)` — the parent handles cleanup, the component does not self-close. Clean separation.

**OBSERVATION** — The `handleSubmit` function is wired to `onClick` on the "Start Research" button rather than `onSubmit` on the `<form>`. This is consistent with `NewIdeaModal` and works correctly since `handleSubmit` calls `e.preventDefault()`. No defect.

---

### KnowledgePage.tsx — EntryListPane

**PASS** — `group` class added to the outer row `<button>`. The Research icon `<button type="button">` is placed inside as a flex sibling, with `e.stopPropagation()` to prevent the click bubbling to the row select handler.

**PASS** — Research button uses `opacity-0 group-hover:opacity-100 transition-opacity` — invisible by default, revealed on row hover. Touch devices (no hover support) will always see it.

**PASS** — `onResearch` prop is correctly typed and threaded through from `KnowledgePage` to `EntryListPane`.

---

### KnowledgePage.tsx — Root Component

**PASS** — `researchTarget` state uses `{ slug, title }` shape. A single state variable rather than two avoids split-update races.

**PASS** — Modal is rendered at the page root, outside the pane layout divs, so it is never clipped by `overflow-hidden` ancestors.

**PASS** — Both `onClose` and `onStarted` set `researchTarget` to `null`, ensuring the modal is always unmounted on dismiss or success.

**PASS** — `EntryDetailPane` "Research More" button is untouched.

---

## Acceptance Criteria Verification

| AC | Status |
|----|--------|
| 1. Research button on list opens modal with correct title | PASS |
| 2. Empty topic → no topic argument | PASS (`trimmed \|\| undefined`) |
| 3. Non-empty topic → topic passed | PASS |
| 4. Modal closes after submit; list selection unchanged | PASS |
| 5. Escape closes modal without starting run | PASS |
| 6. Detail pane Research More button unchanged | PASS |
| 7. No TypeScript errors; build succeeds | PASS (`tsc -b && vite build` clean) |

---

## Verdict

**APPROVED.** No blockers. Implementation is clean, follows established patterns, and satisfies all acceptance criteria.
