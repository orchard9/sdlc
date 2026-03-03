# Design: ponder-creation-modal

## Summary

A new `NewIdeaModal` component replaces the inline `NewIdeaForm` in `PonderPage.tsx`. It is a proper centered dialog that gives users ample space to write a description and optionally add URL references at creation time.

---

## Component Tree

```
PonderPage
  └── NewIdeaModal          ← new component (replaces NewIdeaForm)
        ├── Modal backdrop
        └── Modal card
              ├── Header ("New Idea")
              ├── Title input
              ├── Slug preview/input
              ├── Description textarea (6+ rows)
              ├── ReferencesInput (dynamic list)
              │     ├── URL row × N
              │     └── "+ Add reference" button
              ├── Error message (conditional)
              └── Footer (Cancel / Create buttons)
```

---

## ASCII Wireframe

```
┌────────────────────────────────────────────────────┐
│  New Idea                                    [✕]   │
├────────────────────────────────────────────────────┤
│  Title                                             │
│  ┌──────────────────────────────────────────────┐  │
│  │ What are you thinking about?                 │  │
│  └──────────────────────────────────────────────┘  │
│                                                    │
│  Slug                                              │
│  ┌──────────────────────────────────────────────┐  │
│  │ my-idea-slug               (monospace, dim)  │  │
│  └──────────────────────────────────────────────┘  │
│                                                    │
│  Description (optional)                            │
│  ┌──────────────────────────────────────────────┐  │
│  │                                              │  │
│  │                                              │  │
│  │                                              │  │
│  │                                              │  │
│  │                                              │  │
│  │                                              │  │
│  └──────────────────────────────────────────────┘  │
│                                                    │
│  References (optional)                             │
│  ┌────────────────────────────────────────┐ [✕]   │
│  │ https://...                            │       │
│  └────────────────────────────────────────┘       │
│  + Add reference                                   │
│                                                    │
│  [error message if any]                            │
├────────────────────────────────────────────────────┤
│               [Cancel]  [Create Idea →]            │
└────────────────────────────────────────────────────┘
```

---

## File Location

```
frontend/src/components/ponder/NewIdeaModal.tsx    ← new file
frontend/src/pages/PonderPage.tsx                 ← modified (remove NewIdeaForm, add NewIdeaModal)
```

---

## `NewIdeaModal` Props

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

---

## State Inside `NewIdeaModal`

```ts
const [title, setTitle] = useState(initialTitle ?? '')
const [slug, setSlug] = useState(initialSlug ?? '')
const [brief, setBrief] = useState(initialBrief ?? '')
const [refs, setRefs] = useState<string[]>([''])  // always at least one empty row
const [submitting, setSubmitting] = useState(false)
const [error, setError] = useState<string | null>(null)
```

---

## References Input Design

- `refs` is an array of strings. Each element corresponds to one URL input row.
- Initial state: `['']` — one empty row.
- "Add reference" appends `''` to the array.
- Remove button at index `i` removes `refs[i]`. If removing would result in an empty array, it resets to `['']`.
- On submit, filter out empty strings: `refs.filter(r => r.trim())`.
- If the resulting list is non-empty, build a markdown string:

```ts
const refMd = `# References\n\n${validRefs.map(r => `- ${r}`).join('\n')}\n`
await api.capturePonderArtifact(slug, { filename: 'references.md', content: refMd })
```

---

## Submit Logic

```
1. Validate: title and slug must be non-empty
2. setSubmitting(true)
3. await api.createPonderEntry({ slug, title, brief: brief.trim() || undefined })
4. if refs → await api.capturePonderArtifact(slug, { filename: 'references.md', content: refMd })
5. api.startPonderChat(slug, seed).catch(() => {})
   seed = brief.trim() ? `${title}\n\n${brief}` : title
6. onCreated(slug)
```

Error handling: catch block sets `error` message, resets `submitting`.

---

## PonderPage Changes

```diff
- function NewIdeaForm({ ... }) { ... }   // deleted entirely

+ import { NewIdeaModal } from '@/components/ponder/NewIdeaModal'

  // in PonderPage render:
- {showForm && (
-   <NewIdeaForm ... />
- )}

+ <NewIdeaModal
+   open={showForm}
+   onClose={() => { setShowForm(false); clearPrefill() }}
+   onCreated={(newSlug) => {
+     setShowForm(false)
+     clearPrefill()
+     load()
+     navigate(`/ponder/${newSlug}`)
+   }}
+   initialTitle={prefillTitle ?? undefined}
+   initialSlug={prefillSlug ?? undefined}
+   initialBrief={prefillBrief ?? undefined}
+ />
```

Where `clearPrefill` resets `prefillTitle`, `prefillSlug`, `prefillBrief` to null.

---

## Interaction Details

### Keyboard
- **Escape**: call `onClose()`
- **Tab**: standard browser tab order through form fields
- No Enter-to-submit (textarea would break)

### Auto-focus
- Title input is auto-focused on open (`useEffect` on `open` flag)

### Slug derivation
- Same `titleToSlug` function as today (already defined in `PonderPage.tsx`)
- Slug auto-updates when title changes, as long as the slug hasn't been manually edited
- Manual slug edit breaks the auto-derive link (same as current behavior)

### Backdrop click
- Clicking the backdrop (`fixed inset-0 bg-black/60`) calls `onClose()`

---

## Styling

Follows existing modal patterns in the codebase (`ThreadToPonderModal`, `AdvisoryPanel`):

- `fixed inset-0 z-50 flex items-center justify-center`
- Backdrop: `absolute inset-0 bg-black/60`
- Card: `relative bg-card border border-border rounded-xl shadow-xl w-full max-w-xl mx-4`
- `max-h-[85vh] flex flex-col` with `overflow-y-auto` on the body section
- Footer: `shrink-0` row with Cancel + Create buttons

---

## No Backend Changes Required

The three API calls used are all existing endpoints:

| Call | Endpoint | Status |
|---|---|---|
| Create entry | `POST /api/roadmap` | Existing |
| Save references | `POST /api/roadmap/:slug/capture` | Existing |
| Start chat | `POST /api/ponder/:slug/chat` | Existing |
