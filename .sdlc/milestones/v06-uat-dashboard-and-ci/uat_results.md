# UAT Run — UAT run history dashboard, CI gate, and full guidance update
**Date:** 2026-03-01T08:15:00Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS WITH TASKS

---

- [x] `MilestoneDetail` page renders a `UatHistoryPanel` component showing past UAT runs _(2026-03-01T08:15:00Z)_
- [x] Each row shows: date, verdict badge (PASS / PASS+TASKS / FAILED), test count (passed/total), tasks created _(2026-03-01T08:15:00Z — fixed: `getByText('PASS', { exact: true })` to avoid matching "passed")_
- [x] `SseMessage::MilestoneUatCompleted { slug }` variant exists and is emitted when a UAT run finishes _(2026-03-01T08:15:00Z)_
- [ ] ~~`useSSE` hook dispatches `MilestoneUatCompleted` and the `UatHistoryPanel` refreshes without a page reload~~ _(✗ task milestone-uat-history-panel#T6 — useSSE has no milestone_uat event handler; UatHistoryPanel fetches once on mount only)_
- [x] `.github/workflows/uat.yml` exists _(2026-03-01T08:15:00Z)_
- [ ] ~~uat.yml triggers on `push` to `main` and `pull_request` touching `frontend/**` or `crates/**`~~ _(✗ task playwright-github-actions#T5 — workflow only triggers on PR to main; no push trigger and no path filters)_
- [x] The GitHub Actions workflow builds `sdlc`, installs Playwright Chromium, runs `npx playwright test`, uploads HTML report (30-day) on `always()`, uploads traces on `failure()` _(2026-03-01T08:15:00Z)_
- [ ] ~~`.sdlc/guidance.md` §5 references Playwright and the three-tier UAT pattern~~ _(✗ task guidance-playwright-update#T4 — guidance.md §5 has no Playwright content; agent made changes to a different section)_
- [x] `CLAUDE.md` links to `docs/uat-enterprise-strategy.md` in the Key Files table _(2026-03-01T08:15:00Z)_
- [x] `SDLC_MILESTONE_UAT_COMMAND` const in `init.rs` includes Playwright workflow (Mode A / Mode B) _(2026-03-01T08:15:00Z)_

---

**Tasks created:** milestone-uat-history-panel#T6, playwright-github-actions#T5, guidance-playwright-update#T4
**12/16 steps verified** (3 deferred to tasks, 1 confirmed via both code check and UI)
