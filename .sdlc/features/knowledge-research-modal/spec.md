# Spec: Knowledge Research Modal and Research Button on List View

## Overview

The Knowledge page currently has a "Research More" button only in the `EntryDetailPane` (right-hand panel). This feature adds two UI improvements to the Knowledge page:

1. A **Research button on the entry list** (`EntryListPane`) — so users can trigger a research run without having to open an entry.
2. A **NewResearchModal** component — a focused modal dialog (parallel to `NewIdeaModal` in Ponder) that lets the user specify an optional custom topic before triggering `api.researchKnowledge(slug, topic)`.

## Background

`api.researchKnowledge(slug, topic?)` exists and is already wired to `POST /api/knowledge/:slug/research`. The server-side handler accepts an optional `topic` in the request body and uses it as the research focus. Today the UI always calls the API with no topic (falling back to the entry slug), so users have no way to steer the research agent toward a specific angle.

## User Problem

Users who want to expand knowledge on a sub-topic (e.g., "error handling patterns" inside a broader "Rust async" entry) must use the CLI. The UI provides no way to provide a topic hint or to launch research directly from the list without drilling into the entry.

## Goals

1. Surface research intent earlier — directly from the entry list.
2. Allow users to provide a topic hint to the research agent before the run starts.
3. Keep the UX lightweight — the modal is small, one-field, confirmation-style.

## Non-Goals

- No changes to the server-side research endpoint.
- No changes to how research results are displayed.
- No bulk research across multiple entries.

## Proposed Changes

### 1. `NewResearchModal` component

**File:** `frontend/src/components/knowledge/NewResearchModal.tsx` (new file)

Props:
```ts
interface NewResearchModalProps {
  open: boolean
  entrySlug: string
  entryTitle: string
  onClose: () => void
  onStarted: () => void  // called after the API call succeeds
}
```

Behaviour:
- Renders a modal overlay (same pattern as `NewIdeaModal`: backdrop + centered card, `z-50`, Escape to close).
- Header: "Research: {entryTitle}"
- Single optional text field labelled "Topic hint" — placeholder "Leave blank to research the full entry topic".
- Footer: Cancel and "Start Research" button (disabled while submitting).
- On submit, calls `api.researchKnowledge(entrySlug, topic || undefined)`, then calls `onStarted()` and closes.
- Error state shown inline if the API call throws.

### 2. Research button in `EntryListPane`

**File:** `frontend/src/pages/KnowledgePage.tsx`

Each entry row in `EntryListPane` gains a small "Research" icon button (a `FlaskConical` or `RefreshCw` icon from lucide-react) that appears on hover (or always visible on touch). Clicking it opens the `NewResearchModal` for that entry.

- The button must stop propagation so it does not select the entry.
- While the modal is open for an entry, the button shows a subtle loading state if research is already running (optional, can use a `researching` boolean in the parent state keyed by slug).

### 3. Wire modal into `KnowledgePage`

`KnowledgePage` manages:
```ts
const [researchModalSlug, setResearchModalSlug] = useState<string | null>(null)
```

When non-null, render `<NewResearchModal open entrySlug={researchModalSlug} ... />`.

## Acceptance Criteria

1. From the entry list, clicking the Research button on any entry opens `NewResearchModal` with that entry's title in the header.
2. Submitting the modal with an empty topic calls `api.researchKnowledge(slug)` (no topic argument).
3. Submitting with a non-empty topic calls `api.researchKnowledge(slug, topic)`.
4. After the API call returns, the modal closes and the list entry is not deselected.
5. Pressing Escape while the modal is open closes it without starting a research run.
6. The Research button in `EntryDetailPane` (existing button) is unchanged.
7. No TypeScript errors; passes `cargo clippy` (frontend-only change, but build must succeed).

## Out of Scope

- Showing live run status in the list row (handled by existing SSE reload on `KnowledgeResearchCompleted`).
- Any changes to the detail pane's existing "Research More" button.
