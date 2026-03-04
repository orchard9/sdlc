# Tasks: Human UAT Frontend — Submission Modal and Secondary Buttons

## T1: Add API client helpers for human UAT submission

Add three functions to `frontend/src/api/client.ts`:
- `submitHumanMilestoneUat(slug, body)` — `POST /api/milestones/{slug}/uat/human`
- `submitHumanFeatureQa(slug, body)` — `POST /api/features/{slug}/qa/human`
- `getMilestoneAcceptanceTest(slug)` — `GET /api/milestones/{slug}/acceptance-test`

The `submitHumanMilestoneUat` body: `{ verdict: string; notes: string; tests_total?: number; tests_passed?: number; tests_failed?: number }`.
The `submitHumanFeatureQa` body: `{ verdict: string; notes: string }`.

## T2: Implement `HumanUatModal` component

Create `frontend/src/components/shared/HumanUatModal.tsx`.

Props:
```ts
{ open: boolean; onClose: () => void; mode: 'milestone' | 'feature'; slug: string }
```

Sections (in order):
1. Header with title ("Submit UAT Results" or "Submit QA Results") and close button (×).
2. Checklist — fetched on open via `api.getArtifact(slug, 'qa_plan').content` (feature) or `api.getMilestoneAcceptanceTest(slug).content` (milestone). Rendered read-only in a scrollable pre-formatted block. Missing/failed → placeholder message.
3. Verdict radios: Pass / Pass with Tasks / Fail.
4. Notes textarea — required for non-Pass verdicts; label shows "required" in amber when applicable.
5. Inline error message when submission fails.
6. Footer: Cancel (secondary) + Submit Results (primary, with spinner when submitting).

Validation: Submit disabled when `verdict === null` or when notes required and empty.
On success: call `onClose()`.
Overlay: fixed inset-0 dark overlay; click outside closes modal.

## T3: Update `VerifyingMini` in `MilestonePreparePanel.tsx`

- Add `useState<boolean>` for modal open state.
- Render `<HumanUatModal>` when open.
- Add "Submit manually" text link below the "Run UAT" button row — hidden while `running` is true.
- Import `HumanUatModal` from `@/components/shared/HumanUatModal`.

## T4: Update feature `run_qa` action card in `FeatureDetail.tsx`

- Add `useState<boolean>` for modal open state.
- When `classification.action === 'run_qa'` and not running: render "Submit manually" as a ghost/text button to the left of the primary "Run" button.
- Render `<HumanUatModal mode="feature">` when open.
- Import `HumanUatModal`.
