# QA Plan: Align Button Direction Input Modal

## Scope
Verify that clicking "Align" on VisionPage, ArchitecturePage, and SetupPage opens
a direction-input modal before firing any network request, and that the direction
is correctly forwarded to the backend agent prompt.

## Test Scenarios

### 1. VisionPage — modal opens on click
- Navigate to `/vision`
- Click the "Align" button
- **Expected:** AlignModal appears with title "Align Vision"; no network request has fired

### 2. VisionPage — cancel aborts the action
- Open AlignModal
- Click "Cancel" (or press Escape, or click backdrop)
- **Expected:** Modal closes; `aligning` spinner never appears; no POST to `/api/vision/run`

### 3. VisionPage — confirm with empty direction
- Open AlignModal, leave textarea blank
- Click "Align"
- **Expected:** Modal closes, spinner shown, POST `/api/vision/run` fires with body `{ "direction": "" }`

### 4. VisionPage — confirm with direction text
- Open AlignModal, type "Focus on the agent runtime model"
- Click "Align" (or ⌘↵)
- **Expected:** POST body contains `{ "direction": "Focus on the agent runtime model" }`

### 5. ArchitecturePage — same four scenarios
- Repeat scenarios 1–4 on `/architecture`
- Modal title should be "Align Architecture"; endpoint is `/api/architecture/run`

### 6. SetupPage — vision align modal
- Navigate to `/setup`, click the Vision "Align" button
- **Expected:** AlignModal opens with title "Align Vision"
- Confirm with a direction → POST fires to `/api/vision/run` with direction in body

### 7. SetupPage — architecture align modal
- On `/setup`, click the Architecture "Align" button  
- **Expected:** AlignModal opens with title "Align Architecture"
- Confirm → POST fires to `/api/architecture/run` with direction in body

### 8. SetupPage — both modals are independent
- Open vision modal, cancel; open architecture modal, confirm
- **Expected:** No cross-contamination of state between the two modals

### 9. Backend — direction included in agent prompt
- POST `/api/vision/run` with `{ "direction": "test direction" }`
- Inspect the spawned run's prompt (via `/api/runs` or logs)
- **Expected:** Prompt begins with `"User direction: test direction\n\n"`

### 10. Backend — empty direction omitted from prompt
- POST `/api/vision/run` with `{ "direction": "" }` or no body
- **Expected:** No "User direction:" prefix in the prompt

### 11. Build health
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
- `cd frontend && npm run build` succeeds with no TypeScript errors

## Pass Criteria
All 11 scenarios pass with no regressions to existing align functionality.
