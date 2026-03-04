# Spec: Human UAT Frontend — Submission Modal and Secondary Buttons

## Overview

This feature adds a "Submit manually" path to both UAT surfaces in the UI:

1. **Milestone UAT** — the `VerifyingMini` component inside `MilestonePreparePanel.tsx`
2. **Feature QA** — the `run_qa` action card inside `FeatureDetail.tsx`

When a user clicks "Submit manually", a shared modal opens. The modal shows the relevant checklist read-only, lets the user select a verdict and write notes, and submits results to the backend via a new REST endpoint (implemented in the companion `human-uat-backend` feature).

## Goals

1. Add a secondary "Submit manually" button alongside the existing "Run UAT" / "Run" primary buttons — lower visual weight, non-disruptive.
2. Build a shared `HumanUatModal` component with checklist display, verdict radio buttons, notes textarea, and submit/cancel actions.
3. Wire up two POST calls:
   - **Milestone**: `POST /api/milestones/{slug}/uat/human` → creates a `UatRun` with `mode = "human"`
   - **Feature QA**: `POST /api/features/{slug}/qa/human` → writes `qa-results.md` as a Draft artifact
4. Surface success feedback and update the relevant panel after submission.

## Non-Goals

- No changes to the AI "Run UAT" or "Run" primary buttons.
- No new pages or routes — the modal is rendered inline in the existing pages.
- No changes to `UatHistoryPanel` — it already auto-refreshes on `MilestoneUatCompleted` SSE.
- No approval step is added here — the approval of the Draft artifact is handled by the existing agent flow.
- Backend implementation is in `human-uat-backend` — this feature only covers the frontend.

## Deliverables

### 1. `HumanUatModal` Component

New file: `frontend/src/components/shared/HumanUatModal.tsx`

**Props:**
```typescript
interface HumanUatModalProps {
  open: boolean
  onClose: () => void
  mode: 'milestone' | 'feature'
  slug: string             // milestone slug OR feature slug
  checklistPath?: string   // optional: path hint for checklist content
}
```

**Modal sections (top to bottom):**

1. **Title row** — "Submit UAT Results" (milestone) or "Submit QA Results" (feature). Close button (×) top-right.

2. **Checklist** — fetched from:
   - Milestone: `GET /api/milestones/{slug}/acceptance-test` (if available) or `GET /api/milestones/{slug}/uat-plan`
   - Feature: `GET /api/artifacts/{slug}/qa_plan`
   Rendered read-only in a scrollable pre-formatted block. If the fetch fails or returns no content, display a placeholder: "No checklist available — proceed with manual assessment."

3. **Verdict** — radio buttons: `Pass`, `Pass with Tasks`, `Fail`.

4. **Notes** — textarea (required when verdict is not "Pass"; optional otherwise). Placeholder: "Describe what you tested and what you found…"

5. **Actions** row — `Cancel` (secondary) and `Submit Results` (primary, disabled while submitting).

**State:**
- `verdict: 'pass' | 'pass_with_tasks' | 'failed' | null` — null until selected
- `notes: string`
- `submitting: boolean`
- `submitError: string | null`

Submit is disabled when `verdict === null` or when verdict is not "pass" and `notes.trim() === ''`.

### 2. API Client Additions

Add to `frontend/src/api/client.ts`:

```typescript
submitHumanMilestoneUat: (
  slug: string,
  body: { verdict: string; notes: string; tests_total?: number; tests_passed?: number; tests_failed?: number }
) =>
  request<{ run_id: string }>(`/api/milestones/${encodeURIComponent(slug)}/uat/human`, {
    method: 'POST',
    body: JSON.stringify(body),
  }),

submitHumanFeatureQa: (
  slug: string,
  body: { verdict: string; notes: string }
) =>
  request<{ status: string }>(`/api/features/${encodeURIComponent(slug)}/qa/human`, {
    method: 'POST',
    body: JSON.stringify(body),
  }),

getMilestoneAcceptanceTest: (slug: string) =>
  request<{ content: string | null }>(`/api/milestones/${encodeURIComponent(slug)}/acceptance-test`),
```

### 3. `VerifyingMini` Update (`MilestonePreparePanel.tsx`)

Add a "Submit manually" button as a secondary action below/beside the primary "Run UAT" button. It should:
- Not be shown while a run is in progress (to avoid confusion).
- Open `HumanUatModal` with `mode="milestone"` and the milestone slug.

After successful submission, close the modal. `UatHistoryPanel` will auto-refresh via SSE.

### 4. Feature QA Secondary Button (`FeatureDetail.tsx`)

When `classification.action === 'run_qa'`, add a "Submit manually" link/button alongside the primary "Run" button.
- Secondary visual style (text link or ghost button).
- Opens `HumanUatModal` with `mode="feature"` and the feature slug.

After successful submission, the feature will refresh via the existing SSE/polling in `useFeature`.

## TypeScript Types

No new types required. The modal uses local state only; API call payloads are inline.

## Acceptance Criteria

- "Submit manually" button appears in `VerifyingMini` when all features are released and no run is active.
- "Submit manually" button/link appears in the `run_qa` action card in `FeatureDetail`.
- `HumanUatModal` opens and closes correctly from both surfaces.
- Checklist renders read-only; absence of checklist shows placeholder message.
- Verdict selection enables the Submit button; notes required for non-Pass verdicts.
- Successful milestone submission calls `POST /api/milestones/{slug}/uat/human` and closes the modal.
- Successful feature submission calls `POST /api/features/{slug}/qa/human` and closes the modal.
- Submit button is disabled while the request is in flight.
- API errors surface inline in the modal (not toast, not crash).
- TypeScript compiles cleanly (`npx tsc --noEmit` in `frontend/`).
- No `console.error` or unhandled promise rejections at runtime.

## Implementation Notes

- The checklist fetch is best-effort. A 404 from the acceptance-test endpoint returns the "no checklist" placeholder gracefully.
- The `HumanUatModal` component should be rendered at the bottom of both parent components (not portaled) to keep focus management simple.
- The `milestones/{slug}/acceptance-test` endpoint does not yet exist — the backend feature will ship it. If it 404s during development, the modal degrades gracefully via the placeholder message.
- Keep the modal implementation simple — a straightforward dialog overlay using the existing `cn()` utility and Tailwind classes consistent with other modals in the codebase.
- No external dialog library needed; the existing pattern (fixed inset overlay with stopPropagation) seen in `ScreenshotLightbox` is the reference.
