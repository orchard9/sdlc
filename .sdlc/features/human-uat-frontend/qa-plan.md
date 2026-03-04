# QA Plan: Human UAT Frontend — Submission Modal and Secondary Buttons

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` succeeds with no new warnings.
- [ ] `cd frontend && npx tsc --noEmit` exits with code 0.
- [ ] No `console.error` or unhandled Promise rejections at runtime when opening and submitting the modal.

## Surface 1: VerifyingMini (Milestone UAT)

### "Submit manually" button visibility
- [ ] "Submit manually" text link is present in `VerifyingMini` when all features in the milestone are released and no run is in progress.
- [ ] "Submit manually" is NOT shown when a UAT run is in progress (running state).
- [ ] "Submit manually" has visually lower weight than the primary "Run UAT" button.

### Modal lifecycle
- [ ] Clicking "Submit manually" opens `HumanUatModal` with title "Submit UAT Results".
- [ ] Clicking the × button closes the modal without submitting.
- [ ] Clicking the "Cancel" button closes the modal without submitting.
- [ ] Clicking outside the modal panel (on the overlay) closes the modal.

## Surface 2: Feature QA Action Card

### "Submit manually" button visibility
- [ ] "Submit manually" button/link is visible in the next-action card when `classification.action === 'run_qa'` and the feature is not currently running.
- [ ] "Submit manually" is NOT shown for any action other than `run_qa`.
- [ ] "Submit manually" has lower visual weight than the primary "Run" button.

### Modal lifecycle
- [ ] Clicking "Submit manually" opens `HumanUatModal` with title "Submit QA Results".
- [ ] Clicking × or Cancel closes the modal without submitting.
- [ ] Clicking the overlay closes the modal.

## HumanUatModal — Checklist Section

- [ ] Milestone mode: modal fetches acceptance test content on open; content renders read-only in scrollable block.
- [ ] Feature mode: modal fetches `qa_plan` artifact content on open; content renders read-only in scrollable block.
- [ ] When fetch fails or returns null/empty: placeholder message "No checklist available — proceed with manual assessment." is shown.
- [ ] Checklist text is not editable.

## HumanUatModal — Verdict Selection

- [ ] Three verdict options present: Pass, Pass with Tasks, Fail.
- [ ] Initially no verdict selected; Submit button disabled.
- [ ] Selecting a verdict enables the Submit button (given notes valid).
- [ ] Only one verdict can be selected at a time.

## HumanUatModal — Notes Validation

- [ ] When verdict is "Pass": notes textarea is optional; Submit is enabled regardless of notes content.
- [ ] When verdict is "Pass with Tasks": notes are required; Submit disabled if notes empty.
- [ ] When verdict is "Fail": notes are required; Submit disabled if notes empty.
- [ ] Notes label shows an appropriate required indicator when notes are required.
- [ ] Filling in notes when required unblocks the Submit button.

## HumanUatModal — Submission

- [ ] Milestone mode: clicking Submit calls `POST /api/milestones/{slug}/uat/human` with `{ verdict, notes }`.
- [ ] Feature mode: clicking Submit calls `POST /api/features/{slug}/qa/human` with `{ verdict, notes }`.
- [ ] During in-flight request: all controls (verdict, notes, Cancel, Submit) are disabled; Submit shows a spinner.
- [ ] On 2xx response: modal closes automatically.
- [ ] On error response: error message is shown inline in the modal; controls re-enabled; Cancel remains active.
- [ ] Error message is descriptive (uses the server error string if available).

## Post-Submission State

- [ ] After successful milestone submission: `UatHistoryPanel` refreshes (may require SSE from `MilestoneUatCompleted` — acceptable if SSE is not yet wired from backend; panel can be verified after `human-uat-backend` ships).
- [ ] After successful feature QA submission: feature detail refreshes (next classification reflects updated artifact state).

## TypeScript Compilation

- [ ] No `// @ts-ignore` or `any` types added without justification.
- [ ] All props passed to `HumanUatModal` are correctly typed.
- [ ] `npx tsc --noEmit` passes cleanly.
