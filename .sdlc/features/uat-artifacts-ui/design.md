# Design: uat-artifacts-ui

## Overview

This design covers the frontend changes only. The backend artifact-serving endpoint and `UatRun.screenshots` model extension are designed in `uat-artifacts-storage`. This feature reads from those contracts.

Two independent UI enhancements:

1. **Screenshot filmstrip** in `UatHistoryPanel` — each run card gains a horizontal scrollable thumbnail strip with a keyboard-accessible lightbox.
2. **Hero thumbnail** in `MilestoneDigestRow` — the dashboard milestone card shows a small preview of the most recent UAT screenshot.

---

## Component Architecture

```
frontend/src/
├── api/client.ts                                  [+] uatArtifactUrl helper
├── components/
│   ├── milestones/
│   │   └── UatHistoryPanel.tsx                    [M] add filmstrip + ScreenshotLightbox
│   └── dashboard/
│       └── MilestoneDigestRow.tsx                 [M] add hero thumbnail + useEffect fetch
```

No new files. All changes are targeted edits to two existing component files and the API client.

---

## ASCII Wireframes

### UatHistoryPanel — Run Card with Filmstrip

```
┌────────────────────────────────────────────────────────────────┐
│ [PASS]  Mar 3, 2026   12/12 passed                            │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ →              │
│ │img 1 │ │img 2 │ │img 3 │ │img 4 │ │img 5 │ scroll         │
│ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘                │
└────────────────────────────────────────────────────────────────┘
```

Each thumbnail: 64 px tall, auto width, border, hover border highlight, cursor pointer.

### ScreenshotLightbox Overlay

```
┌──────────────────────────────────────────────────────────────────┐
│                    dark backdrop (fixed)                         │
│                                                                  │
│    [◀]  ┌─────────────────────────────────┐  [▶]               │
│         │                                 │                      │
│         │        full-size image          │                      │
│         │                                 │                      │
│         └─────────────────────────────────┘                      │
│                  2 / 5                    [✕]                    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

- Click backdrop → close
- Click image → no-op (stopPropagation)
- Keyboard: Escape → close, ArrowLeft/ArrowRight → navigate

### MilestoneDigestRow — Hero Thumbnail

```
┌────────────────────────────────────────────────────────────────────┐
│ ● v22 Project Changelog     [active]  [img] ████░░ 3/5  ›       │
└────────────────────────────────────────────────────────────────────┘
                                          ↑
                              56×32 px hero thumbnail
                              (link to /milestones/{slug})
```

Thumbnail sits between the status badge and the progress bar. It only appears when `getLatestMilestoneUatRun` resolves with a run that has `screenshots[0]`.

---

## State Design

### `UatHistoryPanel` Local State

```ts
// Track which image is open in the lightbox
type LightboxState = { runId: string; index: number } | null
const [lightbox, setLightbox] = useState<LightboxState>(null)
```

No new server state — `runs` is already fetched; screenshots are filenames on each `UatRun`.

### `MilestoneDigestRow` Local State

```ts
const [latestRun, setLatestRun] = useState<UatRun | null>(null)

useEffect(() => {
  api.getLatestMilestoneUatRun(milestone.slug)
    .then(run => setLatestRun(run))
    .catch(() => {}) // graceful — missing thumbnail is not an error
}, [milestone.slug])
```

One fetch per milestone card on mount. No SSE subscription — dashboard milestone cards reload on SSE project events already; the thumbnail will update on next full reload.

---

## `ScreenshotLightbox` Component (local to `UatHistoryPanel.tsx`)

```tsx
interface ScreenshotLightboxProps {
  screenshots: string[]        // filenames
  milestoneSlug: string
  runId: string
  initialIndex: number
  onClose: () => void
}
```

Internal state: `index` (current position). The component registers a `keydown` listener on `document` via `useEffect` (cleaned up on unmount). It renders via `createPortal(…, document.body)` to escape stacking contexts.

**Implementation sketch**:

```tsx
function ScreenshotLightbox({ screenshots, milestoneSlug, runId, initialIndex, onClose }: ScreenshotLightboxProps) {
  const [index, setIndex] = useState(initialIndex)

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
      if (e.key === 'ArrowLeft') setIndex(i => Math.max(0, i - 1))
      if (e.key === 'ArrowRight') setIndex(i => Math.min(screenshots.length - 1, i + 1))
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [onClose, screenshots.length])

  return createPortal(
    <div
      className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center"
      onClick={onClose}
    >
      <div className="relative max-w-5xl max-h-[90vh] flex flex-col items-center" onClick={e => e.stopPropagation()}>
        <img
          src={api.uatArtifactUrl(milestoneSlug, runId, screenshots[index])}
          alt={`UAT screenshot ${index + 1} of ${screenshots.length}`}
          className="max-h-[80vh] max-w-full object-contain rounded"
        />
        <div className="flex items-center gap-4 mt-3">
          <button onClick={() => setIndex(i => Math.max(0, i - 1))} disabled={index === 0} ...>◀</button>
          <span className="text-sm text-white">{index + 1} / {screenshots.length}</span>
          <button onClick={() => setIndex(i => Math.min(screenshots.length - 1, i + 1))} disabled={index === screenshots.length - 1} ...>▶</button>
        </div>
        <button onClick={onClose} className="absolute top-0 right-0 ...">✕</button>
      </div>
    </div>,
    document.body
  )
}
```

---

## Design Decisions

### Why `createPortal` for the lightbox?

`UatHistoryPanel` is rendered inside nested flex containers with `overflow-hidden` ancestors (the card). A fixed-position overlay inside those would be clipped. `createPortal` renders directly into `document.body`, bypassing all stacking contexts.

### Why local state for `latestRun` in `MilestoneDigestRow`?

The dashboard `ProjectState` summary (`MilestoneSummary`) intentionally omits UAT run data to keep the state payload small. Adding UAT run data to the project state would bloat every SSE event. A separate per-card fetch is appropriate — it fires once on mount and the thumbnail is not on the critical rendering path.

### Why not a shared `ScreenshotLightbox` component?

The lightbox is very specific to UAT run screenshots. It is co-located with `UatHistoryPanel` where the screenshots array and run IDs already live. Sharing it would require a more complex prop API for no reuse benefit today.

### Why no loading spinner for images?

Browser-native `loading="lazy"` handles deferral. A broken image icon (if the artifact endpoint 404s) is acceptable — it signals a data integrity issue rather than a UI bug. A loading spinner per-image would add complexity without meaningful UX benefit for a history panel.

---

## Tailwind Classes Reference

| Element | Classes |
|---|---|
| Filmstrip container | `flex gap-2 overflow-x-auto py-1 mt-2` |
| Filmstrip thumbnail | `h-16 w-auto rounded cursor-pointer shrink-0 border border-border hover:border-primary transition-colors` |
| Lightbox backdrop | `fixed inset-0 z-50 bg-black/80 flex items-center justify-center` |
| Lightbox image | `max-h-[80vh] max-w-full object-contain rounded` |
| Hero thumbnail | `h-8 w-14 rounded object-cover border border-border` |

---

## Sequence: Dashboard Milestone Card Load

```
Browser mount MilestoneDigestRow
  → useEffect fires
  → GET /api/milestones/{slug}/uat-runs/latest
  → { id, screenshots: ["step-01.png", ...], ... } or null
  → setLatestRun(run)
  → conditional render of <img> with uatArtifactUrl(...)
  → GET /api/milestones/{slug}/uat-runs/{id}/artifacts/step-01.png (browser, image load)
```

---

## No Backend Changes

This feature makes zero changes to `crates/sdlc-core` or `crates/sdlc-server`. All backend work is in `uat-artifacts-storage`. This feature gate-depends on that feature being merged first.
