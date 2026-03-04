# Design: Human UAT Frontend — Submission Modal and Secondary Buttons

## Overview

Two existing UI surfaces get a secondary "Submit manually" path. A shared `HumanUatModal` component handles both. The design favors minimal disruption to the existing layout and visual hierarchy.

## Component Map

```
frontend/src/
  components/
    shared/
      HumanUatModal.tsx          ← NEW: shared modal used by both surfaces
  components/milestones/
    MilestonePreparePanel.tsx    ← MODIFIED: VerifyingMini gets "Submit manually" button
  pages/
    FeatureDetail.tsx            ← MODIFIED: run_qa action card gets "Submit manually" link
  api/
    client.ts                    ← MODIFIED: 3 new API helpers
```

## Mockup

[Mockup](mockup.html)

## `VerifyingMini` — Updated Layout

```
┌─────────────────────────────────────────────────────────┐
│  ✓ All features released                [Run UAT ▶]      │
│                                  Submit manually ↗        │
└─────────────────────────────────────────────────────────┘
```

- "Submit manually" rendered as a small text link (`text-[10px] text-muted-foreground underline`).
- Hidden while a UAT run is in progress.
- Rendered below the main row to avoid cluttering the compact banner.

## Feature QA Action Card — Updated Layout

```
┌────────────────────────────────────────────────────────────┐
│  Next: run qa                                   [Run ▶]    │
│  Verification step…                 Submit manually         │
│                                                            │
│  /sdlc-run slug  [copy]   /sdlc-next slug  [copy]          │
└────────────────────────────────────────────────────────────┘
```

- "Submit manually" appears as a ghost/text button to the left of the primary "Run" button inside the action row.
- Visible only when `classification.action === 'run_qa'`.

## `HumanUatModal` — Structure

```
┌──────────────────────────────────────────────────┐
│  Submit UAT Results                          [×]  │
├──────────────────────────────────────────────────┤
│  Checklist                                        │
│  ┌────────────────────────────────────────────┐  │
│  │  ## Acceptance Test                        │  │
│  │  - [ ] Feature X works in Safari          │  │
│  │  - [ ] Mobile layout is correct           │  │  ← read-only, scrollable (max-h-40)
│  │  ...                                      │  │
│  └────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────┤
│  Verdict                                          │
│  ● Pass   ○ Pass with Tasks   ○ Fail              │
├──────────────────────────────────────────────────┤
│  Notes                                            │
│  ┌────────────────────────────────────────────┐  │
│  │  Describe what you tested…                 │  │  ← required for non-Pass
│  └────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────┤
│  [error message if submit failed]                 │
│                         [Cancel]  [Submit Results]│
└──────────────────────────────────────────────────┘
```

### Overlay pattern

Full-screen dark overlay (`fixed inset-0 bg-black/60 z-50`), modal card centered (`flex items-center justify-center`). Click outside closes the modal. This is the same pattern as `ScreenshotLightbox`.

### Validation rules

| Verdict | Notes required? |
|---|---|
| pass | No |
| pass_with_tasks | Yes |
| failed | Yes |

Submit button is disabled when `verdict === null` or when notes are required and empty. Submit button shows a spinner while in-flight.

## Data Flow

### Milestone path

```
user clicks "Submit manually"
  → HumanUatModal opens (mode="milestone", slug=milestoneSlug)
  → fetches acceptance test: GET /api/milestones/{slug}/acceptance-test
  → user fills form, clicks "Submit Results"
  → POST /api/milestones/{slug}/uat/human { verdict, notes }
  → 200: close modal
  → UatHistoryPanel auto-refreshes via MilestoneUatCompleted SSE
```

### Feature path

```
user clicks "Submit manually"
  → HumanUatModal opens (mode="feature", slug=featureSlug)
  → fetches qa_plan: GET /api/artifacts/{slug}/qa_plan
  → user fills form, clicks "Submit Results"
  → POST /api/features/{slug}/qa/human { verdict, notes }
  → 200: close modal
  → feature auto-refreshes via existing useFeature SSE handling
```

## Styling Notes

- Use `bg-card border border-border rounded-2xl` for the modal panel.
- Use `bg-muted text-muted-foreground` for the checklist block.
- Verdict radios: custom styled with Tailwind, no additional library.
- Notes textarea: `resize-none h-20 w-full rounded-lg border border-border bg-background px-3 py-2 text-sm`.
- All button styles follow the existing patterns in `MilestonePreparePanel.tsx` and `FeatureDetail.tsx`.
- Error message: `text-xs text-destructive mt-1`.

## Implementation Order

1. Add `submitHumanMilestoneUat`, `submitHumanFeatureQa`, `getMilestoneAcceptanceTest` to `client.ts`.
2. Implement `HumanUatModal` component.
3. Update `VerifyingMini` in `MilestonePreparePanel.tsx`.
4. Update `run_qa` action card in `FeatureDetail.tsx`.
