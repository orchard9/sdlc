# Design: Knowledge Research Modal and Research Button on List View

## Summary

Two small UI additions to `KnowledgePage.tsx`:

1. A hover-revealed "Research" icon button on each row in the `EntryListPane`.
2. A `NewResearchModal` component that lets the user optionally specify a topic hint before calling `api.researchKnowledge(slug, topic?)`.

---

## Component Architecture

```
KnowledgePage
├── CatalogPane          (no changes)
├── EntryListPane        (add: onResearch callback + Research button per row)
│   └── [entry row]
│       └── [Research icon button] → opens NewResearchModal
├── EntryDetailPane      (no changes)
└── NewResearchModal     (new component — rendered at page level)
```

`NewResearchModal` is rendered by `KnowledgePage` (not inside `EntryListPane`) to avoid z-index and overflow-clip issues.

---

## NewResearchModal — Wireframe

```
┌────────────────────────────────────────────┐
│  Research: Rust async patterns         [X] │
├────────────────────────────────────────────┤
│                                            │
│  Topic hint (optional)                     │
│  ┌──────────────────────────────────────┐  │
│  │ e.g. error handling, cancellation…  │  │
│  └──────────────────────────────────────┘  │
│                                            │
│  [error message if any]                    │
│                                            │
├────────────────────────────────────────────┤
│              [Cancel]  [Start Research]    │
└────────────────────────────────────────────┘
```

- Modal card: `max-w-sm`, centered, `z-50`
- Backdrop: `bg-black/60`, click to close
- Header: title = "Research: {entryTitle}", close button (X)
- Body: single `<textarea>` (or `<input>`) labelled "Topic hint (optional)", placeholder text
- Footer: Cancel (ghost style) + "Start Research" (primary, disabled while submitting)
- Escape key closes without submitting

---

## EntryListPane — Row Wireframe

```
┌─────────────────────────────────────────────────────┐
│  Entry Title                             published   │  ← always visible
│  Brief summary text here…                     [🔬]  │  ← Research button on hover
│  tag1  tag2  tag3                                    │
└─────────────────────────────────────────────────────┘
```

- The Research button (`FlaskConical` icon, 14px) appears absolutely positioned at the right side of the row.
- It is hidden by default and visible on `group-hover` using Tailwind's `group` / `group-hover:opacity-100` pattern.
- On touch devices (no hover support) it is always visible (`opacity-100`).
- Click: `e.stopPropagation()` + calls `onResearch(entry.slug, entry.title)`.
- Tooltip: "Research More"

---

## State Changes in KnowledgePage

```ts
// New state
const [researchTarget, setResearchTarget] = useState<{
  slug: string
  title: string
} | null>(null)

// Handler passed to EntryListPane
const handleResearch = (slug: string, title: string) => {
  setResearchTarget({ slug, title })
}

// In JSX — alongside existing children
{researchTarget && (
  <NewResearchModal
    open
    entrySlug={researchTarget.slug}
    entryTitle={researchTarget.title}
    onClose={() => setResearchTarget(null)}
    onStarted={() => setResearchTarget(null)}
  />
)}
```

---

## File Changes

| File | Change |
|------|--------|
| `frontend/src/components/knowledge/NewResearchModal.tsx` | New file — modal component |
| `frontend/src/pages/KnowledgePage.tsx` | Add Research button to `EntryListPane` rows; wire `NewResearchModal` |

A new directory `frontend/src/components/knowledge/` is created for the modal component. This mirrors the existing `components/ponder/` and `components/investigation/` directories.

---

## API Integration

No changes to the API layer. `api.researchKnowledge` already accepts `(slug, topic?)`:

```ts
researchKnowledge: (slug: string, topic?: string) =>
  request<{ status: string; run_id: string }>(
    `/api/knowledge/${encodeURIComponent(slug)}/research`,
    { method: 'POST', body: JSON.stringify({ topic: topic ?? null }) }
  )
```

The modal calls this and closes on success. Errors are shown inline in the modal.

---

## Styling Conventions

Follows existing patterns from `NewIdeaModal.tsx`:
- `fixed inset-0 z-50 flex items-center justify-center` — overlay wrapper
- `absolute inset-0 bg-black/60` — backdrop
- `relative bg-card border border-border rounded-xl shadow-xl` — modal card
- Input: `px-3 py-2 text-sm bg-muted/60 border border-border rounded-lg` with focus ring

Research button on the row: `opacity-0 group-hover:opacity-100 transition-opacity` with `shrink-0` so it doesn't affect row layout.
