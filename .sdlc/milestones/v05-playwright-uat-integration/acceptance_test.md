# Acceptance Test: Playwright UAT Integration

- [ ] `@microsoft/playwright-mcp` is registered in the project's `.mcp.json`
- [ ] `spawn_agent_run` for milestone-uat in `routes/runs.rs` includes Playwright MCP tools in `allowed_tools`
- [ ] The `sdlc-milestone-uat` skill has Mode A: detects existing `.spec.ts` for the milestone and runs `npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json`
- [ ] The `sdlc-milestone-uat` skill has Mode B: when no spec exists, generates one from `acceptance_test.md` via Playwright MCP browser tools, verifies it passes, then proceeds to synthesis
- [ ] After a UAT run, `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/results.json` is written (Playwright JSON report)
- [ ] After a UAT run, `.sdlc/milestones/<slug>/uat-runs/<date>-<id>/summary.md` is written with verdict (Pass / Pass With Tasks / Failed), test counts, and tasks created
- [ ] `UatRun` struct exists in `sdlc-core` with `id`, `verdict`, `tests_total`, `tests_passed`, `tests_failed`, `tasks_created` fields
- [ ] `Milestone` impl has `list_uat_runs()`, `latest_uat_run()`, `save_uat_run()` methods
- [ ] `GET /api/milestones/{slug}/uat-runs` returns a JSON array of UatRun records
- [ ] `GET /api/milestones/{slug}/uat-runs/latest` returns the most recent run record
