# Human-Run UAT — Design

## Two Surfaces

### 1. Milestone UAT (`MilestonePreparePanel.tsx`)
- `VerifyingMini` component, shown when all features released
- Currently: "Run UAT" button → agent Playwright run
- Add: secondary "Submit manually" link (lower visual weight)

### 2. Feature QA (`FeatureDetail.tsx`, `run_qa` action)
- Currently: "Run" button → agent directive
- Add: secondary "Submit manually" button

## Form Design (shared modal)

```
[Checklist reference — qa_plan.md or acceptance_test.md rendered read-only]

Verdict:  ○ Pass  ○ Pass with Tasks  ○ Fail

Notes: [textarea — required for non-Pass]

[Cancel]  [Submit Results]
```

## Output

**Feature:** writes `qa-results.md` as Draft artifact
```markdown
## Verdict
Pass / Pass with Tasks / Fail

## Notes
{human notes}

Runner: human (manual)
Completed: {timestamp}
```

**Milestone:** creates UatRun with mode="human"
- `verdict`, `tests_total/passed/failed` from form (human provides counts)
- `summary.md` written to standard path
- `uat_results.md` updated

## Backend Changes

| Change | Location |
|--------|----------|
| Add `mode: Option<String>` to UatRun | `sdlc-core/src/milestone.rs` |
| `POST /api/features/{slug}/human-qa` | new route in `crates/sdlc-server/src/routes/` |
| `POST /api/milestone/{slug}/uat/human` | new route in milestones.rs |

## What Stays Unchanged

- Approval flow: agent still approves after human submits (Draft → Approved)
- AI "Run UAT" button: unchanged, primary CTA
- CLI path: humans can still write files directly