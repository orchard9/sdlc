# QA Plan: uat-artifacts-ui

## Scope

Frontend-only changes to two React components and the API client:
- `frontend/src/api/client.ts` — `uatArtifactUrl` helper
- `frontend/src/components/milestones/UatHistoryPanel.tsx` — filmstrip + lightbox
- `frontend/src/components/dashboard/MilestoneDigestRow.tsx` — hero thumbnail

No Rust or server-side changes are in scope.

## Prerequisite

`uat-artifacts-storage` must be merged and the dev server must be running with a milestone that has at least one completed UAT run with `screenshots` populated. If no real data is available, use a mock/test fixture.

---

## Test Cases

### TC-1: Screenshot filmstrip renders when run has screenshots

**Given** a milestone with a completed UAT run that has `screenshots: ["step-01.png", "step-02.png"]`
**When** the user navigates to the milestone detail page
**Then** the `UatHistoryPanel` run card shows a horizontal filmstrip of two thumbnail images
- Images are 64 px tall, auto width, have rounded corners and a border
- Images are ordered in the same order as `screenshots`
- No broken image icons; images load from `/api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}`

### TC-2: No filmstrip when run has no screenshots

**Given** a milestone with a completed UAT run that has `screenshots: []`
**When** the user navigates to the milestone detail page
**Then** the `UatHistoryPanel` run card shows the existing metadata row (verdict, date, count)
- No filmstrip `<div>` is rendered
- No broken image icons appear

### TC-3: Lightbox opens on thumbnail click

**Given** a run card with a filmstrip of two thumbnails
**When** the user clicks the first thumbnail
**Then** a lightbox overlay appears:
- Dark backdrop covers the viewport
- The full-size image is displayed centered
- A page counter shows "1 / 2"
- A close button (✕) is visible
- Prev button is disabled (first image)
- Next button is enabled

### TC-4: Lightbox keyboard navigation

**Given** the lightbox is open at index 0 of 3 screenshots
**When** the user presses `ArrowRight`
**Then** the displayed image changes to index 1 and the counter shows "2 / 3"
**When** the user presses `ArrowLeft`
**Then** the image returns to index 0 and the counter shows "1 / 3"
**When** the user presses `Escape`
**Then** the lightbox closes and the filmstrip is visible again

### TC-5: Lightbox closes on backdrop click

**Given** the lightbox is open
**When** the user clicks outside the centered image (on the dark backdrop)
**Then** the lightbox closes

**When** the user clicks on the image itself
**Then** the lightbox remains open (event does not propagate to backdrop)

### TC-6: Hero thumbnail on dashboard

**Given** a milestone card in `MilestoneDigestRow` whose latest UAT run has `screenshots: ["step-01.png"]`
**When** the dashboard loads
**Then** the collapsed milestone card header shows a small thumbnail (32 × 56 px, `object-cover`)
- The thumbnail is positioned between the status badge and the progress bar
- Clicking the thumbnail navigates to `/milestones/{slug}`
- No lightbox opens from the dashboard thumbnail

### TC-7: No hero thumbnail when latest run has no screenshots

**Given** a milestone card whose latest UAT run has `screenshots: []`, or whose `getLatestMilestoneUatRun` returns `null`
**When** the dashboard loads
**Then** no thumbnail appears in the card header
- No broken image icon
- Layout of the card header is unchanged (status badge, progress bar remain aligned)

### TC-8: `uatArtifactUrl` URL encoding

**Given** `api.uatArtifactUrl("my milestone", "run-id-123", "file with spaces.png")`
**Then** the returned string is `/api/milestones/my%20milestone/uat-runs/run-id-123/artifacts/file%20with%20spaces.png`

Verify by calling the function directly in the browser console or a unit test.

### TC-9: TypeScript no-error compilation

**Given** the changes from T1–T4 applied
**When** running `npx tsc --noEmit` from `frontend/`
**Then** zero TypeScript errors are reported

### TC-10: No regression to existing `UatHistoryPanel` behavior

**Given** the existing `data-testid="uat-history-panel"` attribute is present
**And** a run with no screenshots
**Then** the panel renders the verdict badge, date, and test count exactly as before
- The `data-testid` attribute is preserved on the root element

---

## Browser Testing

Verify in Chromium (primary) and Firefox (secondary):
- Filmstrip horizontal scroll works on narrow viewports
- Lightbox `createPortal` renders above all other content (z-index correct)
- `loading="lazy"` does not cause layout shifts

---

## Regression Scope

- `UatHistoryPanel` loading state still shows spinner
- `UatHistoryPanel` empty state still shows "No UAT runs yet."
- `MilestoneDigestRow` expand/collapse still works
- `MilestoneDigestRow` progress bar and `/sdlc-run` command block still render
- Dashboard SSE-driven updates do not trigger extra `getLatestMilestoneUatRun` calls (only fires on mount)
