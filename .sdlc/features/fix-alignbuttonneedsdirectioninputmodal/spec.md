# Spec: Align Button Direction Input Modal

## Problem

On the Vision and Architecture pages, clicking the "Align" button immediately fires
an API call (`runVisionAlign` / `runArchitectureAlign`) with no opportunity for the
user to provide directional input or context. The backend agent then runs with no
user-supplied direction, making alignment a black-box fire-and-forget operation with
no user control over the outcome.

Root cause: `handleAlign` in `VisionPage.tsx` and `ArchitecturePage.tsx` calls
`api.runVisionAlign()` / `api.runArchitectureAlign()` directly, without first
opening a modal to collect direction input.

Also affected: `SetupPage.tsx` has inline `handleAlignVision` / `handleAlignArchitecture`
handlers that exhibit the same problem (lines 129 and 147).

## Goal

When the user clicks "Align" on either the Vision page, Architecture page, or Setup
page, a small modal opens asking for optional directional input. The user types their
direction (or leaves it empty to run a general alignment), then confirms. Only after
confirmation does the API call fire — carrying the direction text as a `direction`
field in the POST body.

## Behaviour

1. **Click "Align"** → modal opens (no API call yet).
2. **Modal content:**
   - Heading: `"Align [Vision | Architecture]"` (context-appropriate)
   - Subtext: one sentence explaining what direction does (e.g. "Describe what you'd
     like the agent to focus on or emphasise. Leave blank for a general alignment.")
   - Textarea for direction input (placeholder: `"e.g. Emphasise the agent runtime
     model and remove stale milestones"`)
   - Two buttons: "Cancel" and "Align" (primary)
3. **Cancel** → modal closes, no API call, button state unchanged.
4. **Align (confirm)** → modal closes, `aligning` state set to `true`, API POST fires
   with `{ direction: "<user input>" }` (empty string if blank).
5. **API contract change:** both `/api/vision/run` and `/api/architecture/run` accept
   an optional JSON body `{ "direction": "..." }`. When `direction` is non-empty the
   backend agent prepends it to the prompt so the agent knows what to emphasise.

## Scope

### Frontend
- New shared component: `frontend/src/components/shared/AlignModal.tsx`
  - Props: `open`, `onClose`, `onConfirm(direction: string)`, `title`
  - Uses existing button / textarea styling patterns (inline Tailwind, no new deps)
- `VisionPage.tsx` — replace direct `api.runVisionAlign()` call with modal flow
- `ArchitecturePage.tsx` — replace direct `api.runArchitectureAlign()` call with modal flow
- `SetupPage.tsx` — replace both inline align calls with modal flow (one modal component,
  two separate open states or a single shared open+context state)
- `frontend/src/api/client.ts` — update `runVisionAlign` and `runArchitectureAlign`
  to accept optional `direction?: string` and POST it as JSON body

### Backend (Rust)
- `crates/sdlc-server/src/routes/runs.rs`
  - `start_vision_align`: accept optional JSON body `{ direction: Option<String> }`;
    if direction is present and non-empty, prepend `"User direction: <direction>\n\n"`
    to the existing prompt string
  - `start_architecture_align`: same treatment
- No changes to SSE events, state types, or other routes

## Out of Scope
- Direction history / saved presets
- Validation beyond trimming whitespace
- Changes to the backend agent logic beyond the direction prefix

## Acceptance Criteria
1. Clicking "Align" on VisionPage opens the AlignModal before any network call is made.
2. Clicking "Align" on ArchitecturePage opens the AlignModal before any network call is made.
3. Clicking "Align" on SetupPage (both vision and architecture variants) opens the AlignModal.
4. Cancelling the modal leaves the page state unchanged (no `aligning` spinner).
5. Confirming with empty input fires the API with `direction: ""` (general alignment works).
6. Confirming with text fires the API with that text, and the backend includes it in the prompt.
7. The AlignModal is a shared component used by all three call sites.
