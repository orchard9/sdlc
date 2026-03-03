# Spec: ponder-creation-modal

## Overview

The current ponder idea creation form is an inline sidebar form (`NewIdeaForm` in `PonderPage.tsx`) that is too small for the way users actually think when starting an exploration. Users want to dump a larger description, reference URLs they've been reading, and see a spacious canvas that matches the expansive nature of a "ponder" entry.

This feature replaces the inline `NewIdeaForm` with a proper modal dialog and adds a URL references field so that source material can be captured at creation time.

**Note:** Image/binary paste is explicitly out of scope for this feature. Binary attachment storage strategy has not been decided (see ponder entry `we-need-a-way-to-add-images-to-ponder-id`). That is deferred to a follow-on feature. This feature focuses on: bigger modal UX + URL reference capture at creation time.

---

## Problem

1. The inline sidebar form (`NewIdeaForm`) is cramped — only 2 rows for `brief`, no space to paste a URL, and visually undersells the importance of starting a new exploration.
2. Users frequently have a URL or two in mind when starting a ponder (a blog post, a GitHub issue, a tool). There is currently nowhere to capture that.
3. The form lives inside a narrow sidebar, so large descriptions overflow or get truncated before the user finishes typing.

---

## Goals

1. **Bigger creation surface.** Replace the inline form with a centered modal dialog that has ample space for title, brief (larger textarea), and URL references.
2. **URL references.** Add a multi-URL input so users can capture reference URLs at creation time. Each URL is saved as a line in a `references.md` artifact in the ponder's scrapbook.
3. **Preserve existing behavior.** Title → slug auto-derivation, slug override, brief-to-`brief.md` flow, and auto-start of ponder chat are all unchanged.
4. **No binary / image support.** Out of scope for this feature.

---

## Non-Goals

- Binary image paste / file upload
- Editing existing ponder entries through this modal
- Changing the data schema of `PonderEntry` in Rust (no new YAML fields required — references live in a scrapbook artifact)
- Any server-side changes (existing `POST /api/roadmap` and `POST /api/roadmap/:slug/capture` endpoints are sufficient)

---

## User Flow

1. User clicks the `+` (New Idea) button in the Ponder sidebar header or the "New idea" button in the empty-state.
2. A centered modal opens (replacing the inline form appearance) with:
   - **Title** — large text input, auto-focused
   - **Slug** — derived from title; editable
   - **Description** — textarea, at least 6 rows, generous padding
   - **References** — dynamic list of URL inputs; user can add / remove rows
3. User fills in the form and clicks "Create".
4. On success:
   - If `brief` was provided → `api.createPonderEntry({ slug, title, brief })` (same as today)
   - If references were provided → `api.capturePonderArtifact(slug, { filename: 'references.md', content })` where content is a markdown list of the URLs
   - `api.startPonderChat(slug, seed)` is fired (same as today)
   - Modal closes, user is navigated to `/ponder/<slug>`
5. Escape or backdrop click closes the modal.

---

## UI / UX Details

### Modal dimensions

- `max-w-xl` (568px) centered on desktop
- Full-width on mobile (mx-4)
- `max-h-[85vh]` with internal scroll if content overflows

### Fields

| Field | Type | Validation | Notes |
|---|---|---|---|
| Title | `<input type="text">` | Required, non-empty | Auto-derived slug on change |
| Slug | `<input type="text">` | Required, matches `[a-z0-9-]` | Editable, max 40 chars |
| Description | `<textarea>` | Optional | Min 6 rows |
| References | Dynamic list of `<input type="url">` | Optional, each URL validated | Plus button adds row, X removes |

### References input

- Initially shows one empty URL row with a `+` button to add more
- Each row: URL text input + remove (`X`) button
- A row with an empty URL value is ignored on submit (not captured)
- On submit, non-empty URLs are assembled into a markdown file:

```markdown
# References

- <url1>
- <url2>
```

Saved via `api.capturePonderArtifact(slug, { filename: 'references.md', content })`.

---

## Component Architecture

### New component: `NewIdeaModal`

Location: `frontend/src/components/ponder/NewIdeaModal.tsx`

Props:
```ts
interface NewIdeaModalProps {
  open: boolean
  onClose: () => void
  onCreated: (slug: string) => void
  initialTitle?: string
  initialSlug?: string
  initialBrief?: string
}
```

The existing `NewIdeaForm` in `PonderPage.tsx` is **removed** (deleted entirely) and the `showForm` state that controls it is replaced by an `open` boolean that drives `NewIdeaModal`.

### Changes to `PonderPage.tsx`

- Remove `NewIdeaForm` component
- Replace inline `{showForm && <NewIdeaForm ... />}` with `<NewIdeaModal open={showForm} ... />`
- Keep all existing `prefillTitle`, `prefillSlug`, `prefillBrief` state (they are passed to the modal)

---

## API

No new API endpoints required.

Existing endpoints used:
- `POST /api/roadmap` — create entry (slug, title, brief)
- `POST /api/roadmap/:slug/capture` — save references artifact
- `POST /api/ponder/:slug/chat` — start ponder agent session

---

## Acceptance Criteria

1. Clicking "New Idea" in the Ponder sidebar opens a modal (not an inline form).
2. The modal has: Title (required), Slug (derived, editable), Description (multi-line), References (URL list).
3. Creating with a description saves `brief.md` via the API (same as today).
4. Creating with one or more URLs saves `references.md` with a markdown list.
5. Creating without any URLs does not call `capturePonderArtifact` for references.
6. Escape closes the modal without creating anything.
7. The ponder chat is auto-started after creation (same as today).
8. On mobile, the modal is appropriately sized.
9. All slug validation (lowercase, hyphens only, max 40 chars) is preserved.
10. Error messages are shown if creation fails.
