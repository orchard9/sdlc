# Acceptance Test: UAT Dashboard and CI

- [ ] `MilestoneDetail` page renders a `UatHistoryPanel` component showing past UAT runs
- [ ] Each row in the history panel shows: date, verdict badge (PASS / PASS+TASKS / FAILED), test count (passed/total), tasks created
- [ ] `SseMessage::MilestoneUatCompleted { slug }` variant exists and is emitted when a UAT run finishes
- [ ] `useSSE` hook dispatches `MilestoneUatCompleted` and the `UatHistoryPanel` refreshes without a page reload
- [ ] `.github/workflows/uat.yml` exists and triggers on `push` to `main` and `pull_request` touching `frontend/**` or `crates/**`
- [ ] The GitHub Actions workflow: builds `sdlc-server`, installs Playwright Chromium, runs `npx playwright test`, uploads HTML report as artifact (30-day retention) on `always()`, uploads traces on `failure()`
- [ ] `.sdlc/guidance.md` §5 references Playwright and the three-tier UAT pattern
- [ ] `CLAUDE.md` links to `docs/uat-enterprise-strategy.md` in the Key Files table or a dedicated UAT section
- [ ] `SDLC_MILESTONE_UAT_COMMAND` const in `crates/sdlc-cli/src/cmd/init.rs` is updated to include Playwright workflow (Mode A / Mode B)
- [ ] Running `sdlc update` in a consumer project installs the updated `sdlc-milestone-uat.md` skill
