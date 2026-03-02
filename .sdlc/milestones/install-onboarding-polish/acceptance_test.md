# Acceptance Test: Installation and Onboarding Polish

This milestone delivers five changes: file-path error context in `sdlc init`, SSH + `make install` instructions in README, an `sdlc update` docs section in README, Vision/Architecture guidance on the `/setup` page and a dashboard banner, and SSE state consistency for `UatHistoryPanel` and `SettingsPage`.

## CLI / Documentation Checks

- [ ] README contains SSH URL variant for `cargo install --git`
- [ ] README contains `make install` as a from-source install path
- [ ] README contains an "Updating" section documenting `sdlc update`
- [ ] `sdlc init` completion message directs users to `sdlc ui` (not `sdlc feature create`)

## UI: Setup Page (/setup)

- [ ] Navigate to `/setup` — the page loads and shows a multi-step setup wizard
- [ ] Step 2 (Vision) shows guidance text explaining what Vision is and that agents use it
- [ ] Step 3 (Architecture) shows guidance text explaining what Architecture is and that agents use it

## UI: Dashboard Vision/Architecture Banner

- [ ] Dashboard loads and shows project state without errors
- [ ] When Vision and Architecture documents exist, no "Go to Setup" banner is shown

## UI: Milestone Detail — UatHistoryPanel

- [ ] Navigate to `/milestones/install-onboarding-polish` — the page loads
- [ ] The UAT History panel renders (element with data-testid="uat-history-panel" is present)

## UI: Settings / Config Page (/config)

- [ ] Navigate to `/config` — the config settings page loads successfully
- [ ] Config data (project settings) is displayed on the page
