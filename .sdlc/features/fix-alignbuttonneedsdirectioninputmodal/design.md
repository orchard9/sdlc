# Design: Align Button Direction Input Modal

## Overview

A lightweight shared modal intercepts the "Align" button click on VisionPage,
ArchitecturePage, and SetupPage. The modal collects an optional direction string,
then fires the API call with that string in the POST body. The backend prepends it
to the agent prompt when non-empty.

## Component: `AlignModal`

**File:** `frontend/src/components/shared/AlignModal.tsx`

Modelled after `FixRightAwayModal` (same overlay/card pattern, Escape-to-close,
backdrop click closes). Much simpler — single step, no async in the modal itself.

```
Props:
  open: boolean
  onClose: () => void
  onConfirm: (direction: string) => void
  title: string          // "Align Vision" | "Align Architecture"
```

### Wireframe

```
┌─────────────────────────────────────────────┐
│  ✨ Align Vision                             │
│  Describe what to focus on. Leave blank     │
│  for a general alignment.                   │
├─────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐    │
│  │ e.g. Emphasise the agent runtime    │    │
│  │ model and remove stale milestones   │    │
│  │                                     │    │
│  │                                     │    │
│  └─────────────────────────────────────┘    │
│                                             │
│           [Cancel]        [Align →]         │
└─────────────────────────────────────────────┘
```

- Textarea: 4 rows, monospace, auto-focused on open
- `⌘↵` / `Ctrl↵` fires confirm (hint shown in footer left)
- "Align →" is primary (bg-primary), always enabled (direction is optional)
- "Cancel" is ghost (text-muted-foreground)
- Backdrop click and Escape both cancel

## Frontend Call Sites

### `VisionPage.tsx`

```tsx
// Before: one-liner direct call
const handleAlign = () => {
  setAligning(true)
  api.runVisionAlign().catch(() => setAligning(false))
}

// After: open modal, API call deferred to onConfirm
const [modalOpen, setModalOpen] = useState(false)
const handleAlignConfirm = (direction: string) => {
  setModalOpen(false)
  setAligning(true)
  api.runVisionAlign(direction).catch(() => setAligning(false))
}
// Button onClick={()=>setModalOpen(true)}
// <AlignModal open={modalOpen} onClose={()=>setModalOpen(false)}
//             onConfirm={handleAlignConfirm} title="Align Vision" />
```

Same pattern in `ArchitecturePage.tsx` (title: "Align Architecture").

### `SetupPage.tsx`

Two separate align flows on the same page. Use a discriminated state:
```tsx
const [alignModal, setAlignModal] = useState<'vision'|'architecture'|null>(null)
```
- Vision "Align" button → `setAlignModal('vision')`
- Architecture "Align" button → `setAlignModal('architecture')`
- Single `<AlignModal>` instance, title derived from `alignModal` state
- `onConfirm` dispatches to the correct `api.runVisionAlign(d)` / `api.runArchitectureAlign(d)`

## API Client (`frontend/src/api/client.ts`)

```ts
runVisionAlign: (direction?: string) =>
  request<{ status: string; run_id: string }>('/api/vision/run', {
    method: 'POST',
    body: JSON.stringify({ direction: direction ?? '' }),
    headers: { 'Content-Type': 'application/json' },
  }),

runArchitectureAlign: (direction?: string) =>
  request<{ status: string; run_id: string }>('/api/architecture/run', {
    method: 'POST',
    body: JSON.stringify({ direction: direction ?? '' }),
    headers: { 'Content-Type': 'application/json' },
  }),
```

(The existing `request` helper should already pass headers through; verify.)

## Backend (`crates/sdlc-server/src/routes/runs.rs`)

Define a shared extractor struct:
```rust
#[derive(serde::Deserialize, Default)]
struct AlignBody {
    direction: Option<String>,
}
```

Update both handlers to accept `Json<AlignBody>`:
```rust
pub async fn start_vision_align(
    State(app): State<AppState>,
    Json(body): Json<AlignBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let direction_prefix = body.direction
        .filter(|d| !d.trim().is_empty())
        .map(|d| format!("User direction: {d}\n\n"))
        .unwrap_or_default();
    let prompt = format!("{direction_prefix}<existing prompt text>");
    ...
}
```

Same change for `start_architecture_align`.

Because `AlignBody` derives `Default`, `axum` will accept an empty body (no
`Content-Type`) as `AlignBody::default()` — backward compatible with any existing
caller that sends no body.

## No Other Changes

- SSE events unchanged
- Route paths unchanged (`/api/vision/run`, `/api/architecture/run`)
- No new routes, no database changes
