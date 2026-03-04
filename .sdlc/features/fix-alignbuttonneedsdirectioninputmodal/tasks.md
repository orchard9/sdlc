# Tasks: Align Button Direction Input Modal

## T1 — Create `AlignModal` shared component
**File:** `frontend/src/components/shared/AlignModal.tsx`
- Props: `open`, `onClose`, `onConfirm(direction: string)`, `title`
- Overlay + card layout matching `FixRightAwayModal` pattern
- Textarea (4 rows, auto-focus on open, `⌘↵` / `Ctrl↵` confirms)
- Cancel (ghost) and Align (primary) buttons
- Backdrop click and Escape both close/cancel

## T2 — Update API client to accept `direction`
**File:** `frontend/src/api/client.ts`
- `runVisionAlign(direction?: string)` — POST `/api/vision/run` with JSON body `{ direction }`
- `runArchitectureAlign(direction?: string)` — POST `/api/architecture/run` with JSON body `{ direction }`

## T3 — Wire modal into `VisionPage.tsx`
**File:** `frontend/src/pages/VisionPage.tsx`
- Add `modalOpen` state
- "Align" button opens modal instead of calling API
- `onConfirm(direction)` handler closes modal, sets `aligning`, calls `api.runVisionAlign(direction)`
- Render `<AlignModal>` with title `"Align Vision"`

## T4 — Wire modal into `ArchitecturePage.tsx`
**File:** `frontend/src/pages/ArchitecturePage.tsx`
- Same pattern as T3, title `"Align Architecture"`, calls `api.runArchitectureAlign(direction)`

## T5 — Wire modal into `SetupPage.tsx`
**File:** `frontend/src/pages/SetupPage.tsx`
- Replace `handleAlignVision` and `handleAlignArchitecture` direct calls with modal flow
- Single `alignModal: 'vision' | 'architecture' | null` state
- One `<AlignModal>` instance, title and onConfirm dispatch derived from state

## T6 — Update backend: accept optional `direction` body in `start_vision_align`
**File:** `crates/sdlc-server/src/routes/runs.rs`
- Add `AlignBody { direction: Option<String> }` extractor struct (derives `Default`)
- Accept `Json<AlignBody>` in `start_vision_align`
- Prepend `"User direction: <direction>\n\n"` to prompt when direction is non-empty

## T7 — Update backend: accept optional `direction` body in `start_architecture_align`
**File:** `crates/sdlc-server/src/routes/runs.rs`
- Same change as T6 for `start_architecture_align`
