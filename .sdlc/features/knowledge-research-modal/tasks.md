# Tasks: Knowledge Research Modal and Research Button on List View

## T1 — Create `NewResearchModal` component

**File:** `frontend/src/components/knowledge/NewResearchModal.tsx`

Create the directory `frontend/src/components/knowledge/` if it does not exist.

Implement the `NewResearchModal` component with:
- Props: `open`, `entrySlug`, `entryTitle`, `onClose`, `onStarted`
- Modal overlay pattern matching `NewIdeaModal` (backdrop + centered card, `z-50`)
- Header: "Research: {entryTitle}" + close (X) button
- Single optional text input labelled "Topic hint (optional)" with placeholder
- Footer: Cancel + "Start Research" button (disabled while submitting)
- On submit: call `api.researchKnowledge(entrySlug, topic.trim() || undefined)`, then `onStarted()` + close
- Inline error display on API failure
- Escape key closes without submitting

## T2 — Add Research button to `EntryListPane` rows

**File:** `frontend/src/pages/KnowledgePage.tsx`

- Add `onResearch: (slug: string, title: string) => void` prop to `EntryListPane`.
- Add `FlaskConical` (or `RefreshCw`) icon button to each entry row with:
  - `group` class on the row button element
  - `opacity-0 group-hover:opacity-100 transition-opacity` on the research button
  - `e.stopPropagation()` in the click handler to prevent entry selection
  - `aria-label="Research More"` and `title="Research More"`

## T3 — Wire `NewResearchModal` into `KnowledgePage`

**File:** `frontend/src/pages/KnowledgePage.tsx`

- Add `researchTarget` state: `{ slug: string; title: string } | null`
- Pass `onResearch` handler to `EntryListPane`
- Render `<NewResearchModal>` when `researchTarget` is non-null
- Close modal on `onClose` and `onStarted` callbacks

## T4 — Build and type-check

Run `cd frontend && npm ci && npm run build` to verify:
- No TypeScript errors
- No missing imports
- Build succeeds
