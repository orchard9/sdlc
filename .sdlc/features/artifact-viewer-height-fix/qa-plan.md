# QA Plan: Remove Artifact Height Cap

## Scope

Verify the single CSS change in `ArtifactViewer.tsx` produces the correct visual behavior and introduces no regressions.

## Test Cases

### TC-1: No inner scrollbar on long artifact

**Setup:** Open the feature detail page for any feature with a spec or design longer than 19 lines.
**Steps:**
1. Navigate to a feature detail page that has an approved spec.
2. Observe the spec artifact card.
**Expected:** The artifact card expands to show the full content. No inner vertical scrollbar appears inside the card.

### TC-2: No `max-h-96 overflow-y-auto` in source

**Steps:**
1. Grep `ArtifactViewer.tsx` for `max-h-96`.
**Expected:** Zero matches.

### TC-3: Short artifacts still render correctly

**Setup:** A feature with a one-line or two-line artifact.
**Steps:**
1. Navigate to a feature with a very short artifact.
2. Observe the artifact card.
**Expected:** The card height matches the content; no extra whitespace; no layout break.

### TC-4: Fullscreen button still functions

**Steps:**
1. Navigate to a feature with artifact content.
2. Click the `⤢` fullscreen button.
**Expected:** `FullscreenModal` opens. Artifact content renders inside the modal. Closing the modal returns to the feature detail page.

### TC-5: No horizontal overflow

**Steps:**
1. Open the feature detail page on a standard viewport (1280px width).
2. Inspect `ArtifactViewer` outer div.
**Expected:** No horizontal scrollbar on the card or the page.

### TC-6: TypeScript / build clean

**Steps:**
1. Run `cd frontend && npx tsc --noEmit`.
**Expected:** Zero TypeScript errors, zero build errors.

## Pass Criteria

All 6 test cases pass. The source contains no `max-h-96` in `ArtifactViewer.tsx`.
